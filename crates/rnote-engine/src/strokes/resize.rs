use na;

/// Enum that lists the different options for sizing the image
///
/// Either respect the original image size (in pixel or dimensions)
/// for svg, impose a size, or resize based on the viewport/page
#[derive(Debug)]
pub enum ImageSizeOption {
    /// respect the size of the original image (no resizing applied)
    RespectOriginalSize,
    /// Use the given size
    ImposeSize(na::Vector2<f64>),
    /// Resize the image to canvas/page view
    ResizeImage(Resize),
}

#[derive(Debug)]
pub struct Resize {
    /// width of a page
    pub width: f64,
    /// if the layout has a fixed size vertically
    pub isfixed_layout: bool,
    /// viewport
    pub max_viewpoint: na::OPoint<f64, na::Const<2>>,
}

/// Calculate the `ratio` by which to resize the image such that
/// - it stays fully in view
/// - it does not goes over a page border when the mode has a fixed
/// width size
///
/// `pos_left_top_canvas` is the position of the top-left corner of
/// the image in documents coordinates
pub fn calculate_resize_ratio(
    resize: Resize,
    initial_size_image: na::Vector2<f64>,
    pos_left_top_canvas: na::Vector2<f64>,
) -> f64 {
    let current_width = initial_size_image.x;
    let current_height = initial_size_image.y;

    // calculate the minimum ratio to stay in view
    let ratio_viewport = (1.0f64)
        .min(
            //check in the horizontal direction
            (resize.max_viewpoint.x - pos_left_top_canvas.index(0)) / (current_width),
        )
        .min(
            //check in the vertical direction
            (resize.max_viewpoint.y - pos_left_top_canvas.index(1)) / (current_height),
        );

    // check if we go out of the page on the right
    // we don't want to go out of the page for fixed layouts
    match resize.isfixed_layout {
        false => ratio_viewport,
        true => ratio_viewport.min((resize.width - pos_left_top_canvas.index(0)) / (current_width)),
    }
}
