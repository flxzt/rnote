// Imports
use crate::RnAppWindow;
use crate::canvas::RnCanvas;
use adw::prelude::*;
use gtk4::{Builder, Button, Picture, TextView, gio, glib, glib::clone};

use std::cell::RefCell;
use std::rc::Rc;
use tracing::error;

pub(crate) async fn dialog_typst_editor(appwindow: &RnAppWindow, canvas: &RnCanvas) {
    let builder = Builder::from_resource(
        (String::from(crate::config::APP_IDPATH) + "ui/dialogs/typsteditor.ui").as_str(),
    );

    let dialog: adw::Dialog = builder.object("dialog_typst_editor").unwrap();
    let button_cancel: Button = builder.object("button_cancel").unwrap();
    let button_insert: Button = builder.object("button_insert").unwrap();
    let button_compile: Button = builder.object("button_compile").unwrap();
    let textview_source: TextView = builder.object("textview_source").unwrap();
    let picture_preview: Picture = builder.object("picture_preview").unwrap();

    let text_buffer = textview_source.buffer();

    // Set default Typst content
    text_buffer.set_text("= Hello Typst!\n\nThis is a *bold* text and _italic_ text.\n\n$ sum_(i=1)^n i = (n(n+1))/2 $");

    // Shared state for compiled SVG
    let compiled_svg: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    // Cancel button
    button_cancel.connect_clicked(clone!(
        #[weak]
        dialog,
        move |_| {
            dialog.close();
        }
    ));

    // Compile button
    button_compile.connect_clicked(clone!(
        #[weak]
        text_buffer,
        #[weak]
        picture_preview,
        #[strong]
        compiled_svg,
        #[weak]
        button_insert,
        move |_| {
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
                        }
                        Err(e) => {
                            error!("Failed to create texture from SVG: {e:?}");
                            show_error_in_preview(&picture_preview, &format!("Preview error: {e}"));
                            button_insert.set_sensitive(false);
                        }
                    }
                }
                Err(e) => {
                    error!("Typst compilation failed: {e:?}");
                    show_error_in_preview(&picture_preview, &format!("Compilation error:\n{e}"));
                    button_insert.set_sensitive(false);
                    *compiled_svg.borrow_mut() = None;
                }
            }
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
        #[strong]
        compiled_svg,
        move |_| {
            if let Some(svg) = compiled_svg.borrow().as_ref() {
                // Get the center of the current viewport as insertion position
                let viewport = canvas.engine_ref().camera.viewport();
                let pos = na::vector![viewport.center().x, viewport.center().y];

                // Insert the SVG into the canvas
                let widget_flags = canvas.engine_mut().insert_svg_image(svg.clone(), pos);
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
    let (width, height) = renderer
        .intrinsic_size_in_pixels()
        .unwrap_or((800.0, 600.0));

    // Create a surface and render the SVG
    let mut surface = cairo::ImageSurface::create(
        cairo::Format::ARgb32,
        width.ceil() as i32,
        height.ceil() as i32,
    )
    .map_err(|e| anyhow::anyhow!("Failed to create surface: {e}"))?;

    {
        let cr = cairo::Context::new(&surface)
            .map_err(|e| anyhow::anyhow!("Failed to create context: {e}"))?;

        renderer
            .render_document(&cr, &cairo::Rectangle::new(0.0, 0.0, width, height))
            .map_err(|e| anyhow::anyhow!("Failed to render SVG: {e}"))?;
    } // Drop cr context here to release the borrow on surface

    // Convert cairo surface to GdkTexture using MemoryTexture
    let width = width as i32;
    let height = height;
    let stride = surface.stride();

    let data = surface
        .data()
        .map_err(|e| anyhow::anyhow!("Failed to get surface data: {e}"))?;

    let bytes = glib::Bytes::from(&data[..]);

    let texture = gdk4::MemoryTexture::new(
        width,
        height as i32,
        gdk4::MemoryFormat::B8g8r8a8,
        &bytes,
        stride as usize,
    );

    Ok(texture.upcast())
}

fn show_error_in_preview(picture: &Picture, error_msg: &str) {
    // Create a simple error image
    let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 400, 300)
        .expect("Failed to create surface");

    {
        let cr = cairo::Context::new(&surface).expect("Failed to create context");

        // White background
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint().expect("Failed to paint");

        // Red border
        cr.set_source_rgb(0.8, 0.2, 0.2);
        cr.set_line_width(2.0);
        cr.rectangle(10.0, 10.0, 380.0, 280.0);
        cr.stroke().expect("Failed to stroke");

        // Error text
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.select_font_face(
            "monospace",
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal,
        );
        cr.set_font_size(12.0);
        cr.move_to(20.0, 40.0);

        // Draw text line by line
        let mut y = 40.0;
        for line in error_msg.lines().take(15) {
            cr.move_to(20.0, y);
            let _ = cr.show_text(line);
            y += 15.0;
        }
    } // Drop cr context here to release the borrow on surface

    // Convert surface to texture
    let width = surface.width();
    let height = surface.height();
    let stride = surface.stride();
    let data = surface.data().expect("Failed to get surface data");

    let bytes = glib::Bytes::from(&data[..]);

    let texture = gdk4::MemoryTexture::new(
        width,
        height,
        gdk4::MemoryFormat::B8g8r8a8,
        &bytes,
        stride as usize,
    );

    picture.set_paintable(Some(&texture));
}
