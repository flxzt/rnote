// Imports
use crate::RnAppWindow;
use crate::canvas::RnCanvas;
use adw::prelude::*;
use glib::timeout_add_local_once;
use gtk4::{Builder, Button, Picture, TextView, glib, glib::clone};
use p2d::bounding_volume::Aabb;
use rnote_engine::strokes::Content;
use rnote_engine::strokes::VectorImage;
use rnote_engine::strokes::content::GeneratedContentImages;
use rnote_engine::strokes::resize::ImageSizeOption;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use tracing::{error, info};

const DEFAULT_TYPST_TEMPLATE: &str = r#"#set page(width: auto, height: auto, margin: 2pt, fill: none)

= Hello Typst!

This is a *bold* text and _italic_ text.

$ sum_(i=1)^n i = (n(n+1))/2 $"#;

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

    // Prevent closing with Escape key by disabling all close attempts
    dialog.set_can_close(false);

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

    let button_close: Button = builder.object("button_close").unwrap();
    let button_insert: Button = builder.object("button_insert").unwrap();
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
        text_buffer.set_text(DEFAULT_TYPST_TEMPLATE);
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
            match rnote_engine::typst::compile_to_svg(&source) {
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
                    info!("Typst compilation failed: {e:?}");
                    // Show error in textview at bottom
                    error_buffer.set_text(&format!("Compilation error:\n{e}"));
                    textview_error.set_visible(true);
                    button_insert.set_sensitive(false);
                    *compiled_svg.borrow_mut() = None;
                }
            }
        }
    );

    // Close button
    button_close.connect_clicked(clone!(
        #[weak]
        dialog,
        move |_| {
            dialog.force_close();
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

                dialog.force_close();
            }
        }
    ));

    // Auto-compile on dialog present
    do_compile();

    dialog.present(Some(appwindow));
}

fn svg_to_texture(svg: &str) -> anyhow::Result<gtk4::gdk::Texture> {
    let image_scale = 2.0;
    let vectorimage = VectorImage::from_svg_str(
        svg,
        na::Vector2::zeros(),
        ImageSizeOption::RespectOriginalSize,
    )?;
    let viewport = Aabb::new(na::point![-1e10, -1e10], na::point![1e10, 1e10]);
    let images = vectorimage.gen_images(viewport, image_scale)?;
    let image = match images {
        GeneratedContentImages::Full(imgs)
        | GeneratedContentImages::Partial { images: imgs, .. } => imgs
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No images generated from SVG"))?,
    };
    let memtexture = image.to_memtexture()?;
    Ok(memtexture.upcast())
}
