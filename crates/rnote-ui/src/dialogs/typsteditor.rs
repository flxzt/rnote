// Imports
use crate::RnAppWindow;
use crate::canvas::RnCanvas;
use adw::prelude::*;
use glib::timeout_add_local_once;
use gtk4::{Builder, Button, Picture, TextView, gio, glib, glib::clone};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use tracing::{error, warn};

pub(crate) async fn dialog_typst_editor(
    appwindow: &RnAppWindow,
    canvas: &RnCanvas,
    initial_source: Option<String>,
    editing_stroke_key: Option<rnote_engine::store::StrokeKey>,
) {
    let builder = Builder::from_resource(
        (String::from(crate::config::APP_IDPATH) + "ui/dialogs/typsteditor.ui").as_str(),
    );

    let dialog: adw::Dialog = builder.object("dialog_typst_editor").unwrap();

    // Set dialog size to ~90% of app window size
    let window_width = appwindow.width();
    let window_height = appwindow.height();

    let dialog_width = (window_width as f64 * 0.9) as i32;
    let dialog_height = (window_height as f64 * 0.9) as i32;

    // Set minimum reasonable sizes
    let dialog_width = dialog_width.max(800);
    let dialog_height = dialog_height.max(600);

    dialog.set_content_width(dialog_width);
    dialog.set_content_height(dialog_height);

    let button_cancel: Button = builder.object("button_cancel").unwrap();
    let button_insert: Button = builder.object("button_insert").unwrap();
    let button_compile: Button = builder.object("button_compile").unwrap();
    let textview_source: TextView = builder.object("textview_source").unwrap();
    let picture_preview: Picture = builder.object("picture_preview").unwrap();
    let textview_error: TextView = builder.object("textview_error").unwrap();
    let paned: gtk4::Paned = builder.object("paned").unwrap();

    // Set paned position to half of dialog width for balanced layout
    paned.set_position(dialog_width / 2);

    let text_buffer = textview_source.buffer();
    let error_buffer = textview_error.buffer();

    // Set Typst content (either provided or default)
    if let Some(source) = initial_source {
        text_buffer.set_text(&source);
    } else {
        text_buffer.set_text("#set page(width: auto, height: auto, margin: 2pt)\n\n= Hello Typst!\n\nThis is a *bold* text and _italic_ text.\n\n$ sum_(i=1)^n i = (n(n+1))/2 $");
    }

    // Shared state for compiled SVG and debounce timer
    let compiled_svg: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let compile_timeout_id: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));

    // Helper function to compile and update preview
    let do_compile = clone!(
        #[weak]
        text_buffer,
        #[weak]
        picture_preview,
        #[weak]
        textview_error,
        #[weak]
        error_buffer,
        #[weak]
        button_insert,
        #[strong]
        compiled_svg,
        #[upgrade_or]
        (),
        move || {
            let source =
                text_buffer.text(&text_buffer.start_iter(), &text_buffer.end_iter(), false);

            // Compile Typst to SVG
            match rnote_engine::utils::typst::compile_to_svg(&source) {
                Ok(svg) => {
                    // Store the compiled SVG
                    *compiled_svg.borrow_mut() = Some(svg.clone());

                    // Convert SVG string to GdkTexture for preview
                    match svg_to_texture(&svg) {
                        Ok(texture) => {
                            picture_preview.set_paintable(Some(&texture));
                            button_insert.set_sensitive(true);
                            // Hide error textview on success
                            textview_error.set_visible(false);
                            error_buffer.set_text("");
                        }
                        Err(e) => {
                            error!("Failed to create texture from SVG: {e:?}");
                            // Show error in textview
                            error_buffer.set_text(&format!("Preview error: {e}"));
                            textview_error.set_visible(true);
                            button_insert.set_sensitive(false);
                        }
                    }
                }
                Err(e) => {
                    warn!("Typst compilation failed: {e:?}");
                    // Show error in textview at bottom
                    error_buffer.set_text(&format!("Compilation error:\n{e}"));
                    textview_error.set_visible(true);
                    button_insert.set_sensitive(false);
                    *compiled_svg.borrow_mut() = None;
                }
            }
        }
    );

    // Cancel button
    button_cancel.connect_clicked(clone!(
        #[weak]
        dialog,
        move |_| {
            dialog.close();
        }
    ));

    // Compile button - just trigger the compile function
    button_compile.connect_clicked(clone!(
        #[strong]
        do_compile,
        move |_| {
            do_compile();
        }
    ));

    // Auto-compile on text changes with debouncing
    text_buffer.connect_changed(clone!(
        #[strong]
        do_compile,
        #[strong]
        compile_timeout_id,
        move |_| {
            // Cancel any existing timeout
            // Note: We don't need to manually remove the old source ID
            // It will be automatically removed when it's dropped
            let _old_id = compile_timeout_id.borrow_mut().take();

            // Schedule a new compile after 500ms of no changes
            let new_id = timeout_add_local_once(
                Duration::from_millis(500),
                clone!(
                    #[strong]
                    do_compile,
                    #[strong]
                    compile_timeout_id,
                    move || {
                        do_compile();
                        // Clear the timeout ID after execution
                        *compile_timeout_id.borrow_mut() = None;
                    }
                ),
            );

            *compile_timeout_id.borrow_mut() = Some(new_id);
        }
    ));

    // Insert button
    button_insert.set_sensitive(false);
    button_insert.connect_clicked(clone!(
        #[weak]
        dialog,
        #[weak]
        canvas,
        #[weak]
        appwindow,
        #[weak]
        text_buffer,
        #[strong]
        compiled_svg,
        move |_| {
            if let Some(svg) = compiled_svg.borrow().as_ref() {
                // Get the Typst source code
                let source =
                    text_buffer.text(&text_buffer.start_iter(), &text_buffer.end_iter(), false);

                let widget_flags = if let Some(stroke_key) = editing_stroke_key {
                    // Update existing stroke
                    canvas.engine_mut().update_typst_stroke(
                        stroke_key,
                        svg.clone(),
                        source.to_string(),
                    )
                } else {
                    // Insert new stroke at center of viewport
                    let viewport = canvas.engine_ref().camera.viewport();
                    let pos = na::vector![viewport.center().x, viewport.center().y];
                    canvas
                        .engine_mut()
                        .insert_svg_image(svg.clone(), pos, Some(source.to_string()))
                };
                appwindow.handle_widget_flags(widget_flags, &canvas);

                dialog.close();
            }
        }
    ));

    // Auto-compile on dialog present
    button_compile.emit_clicked();

    dialog.present(Some(appwindow));
}

fn svg_to_texture(svg: &str) -> anyhow::Result<gdk4::Texture> {
    // Parse the SVG using librsvg
    let bytes = glib::Bytes::from(svg.as_bytes());
    let stream = gio::MemoryInputStream::from_bytes(&bytes);

    let handle = rsvg::Loader::new()
        .read_stream(&stream, None::<&gio::File>, None::<&gio::Cancellable>)
        .map_err(|e| anyhow::anyhow!("Failed to load SVG: {e}"))?;

    let renderer = rsvg::CairoRenderer::new(&handle);

    // Get the intrinsic dimensions to calculate aspect ratio
    let (intrinsic_width, intrinsic_height) = renderer
        .intrinsic_size_in_pixels()
        .unwrap_or((800.0, 600.0));

    let (width, height) = (intrinsic_width * 2.0, intrinsic_height * 2.0);
    let width = width.ceil() as i32;
    let height = height.ceil() as i32;

    // Create a surface and render the SVG at the higher resolution
    let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height)
        .map_err(|e| anyhow::anyhow!("Failed to create surface: {e}"))?;

    {
        let cr = cairo::Context::new(&surface)
            .map_err(|e| anyhow::anyhow!("Failed to create context: {e}"))?;

        renderer
            .render_document(
                &cr,
                &cairo::Rectangle::new(0.0, 0.0, width as f64, height as f64),
            )
            .map_err(|e| anyhow::anyhow!("Failed to render SVG: {e}"))?;
    } // Drop cr context here to release the borrow on surface

    // Convert cairo surface to GdkTexture using MemoryTexture
    let stride = surface.stride();

    let data = surface
        .data()
        .map_err(|e| anyhow::anyhow!("Failed to get surface data: {e}"))?;

    let bytes = glib::Bytes::from(&data[..]);

    let texture = gdk4::MemoryTexture::new(
        width,
        height,
        gdk4::MemoryFormat::B8g8r8a8,
        &bytes,
        stride as usize,
    );

    Ok(texture.upcast())
}
