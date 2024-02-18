use na;
use p2d::query::gjk::eps_tol;

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
    /// height of a page
    pub height: f64,
    /// if the layout has a fixed size vertically
    pub isfixed_layout: bool,
    /// viewport
    pub max_viewpoint: na::OPoint<f64, na::Const<2>>,
    /// To force elements to not go over borders
    /// maybe enabling that to be on only when borders are active
    /// would be a better idea
    pub respect_borders: bool,
}

/// Calculate the `ratio` by which to resize the image such that
/// - it stays fully in view
/// - it does not goes over a page border when the mode has a fixed
/// width size
///
/// There is an additional constraint when the `respect_border`
/// bool of the `Resize` struct is true. In this case we disallow
/// images to go over to the next page on the right
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

    let mut ratio = 1.0f64; //start the ratio at 1

    // check that we do not go out of the canvas view in the x direction
    ratio = ratio.min((resize.max_viewpoint.x - pos_left_top_canvas.index(0)) / (current_width));
    // check that we do not go out of view in the y direction
    ratio = ratio.min((resize.max_viewpoint.y - pos_left_top_canvas.index(1)) / (current_height));

    // check if we go out of the page on the right on fixed layout
    if resize.isfixed_layout {
        ratio = ratio.min((resize.width - pos_left_top_canvas.index(0)) / (current_width));
    }

    // check if we have to respect borders
    if resize.respect_borders {
        ratio = ratio.min(calculate_resize_ratio_respect_borders(
            resize,
            initial_size_image,
            pos_left_top_canvas,
        ));
    }
    ratio
}

/// calculate the ratio to not go over borders
pub fn calculate_resize_ratio_respect_borders(
    resize: Resize,
    initial_size_image: na::Vector2<f64>,
    pos_left_top_canvas: na::Vector2<f64>,
) -> f64 {
    // beware : we MIGHT have zero as the top position, so we start at at minimum at eps
    let next_page_vertical_border =
        (pos_left_top_canvas.index(0).max(eps_tol()) / resize.width).ceil() * resize.width;
    let next_page_horizontal_border =
        (pos_left_top_canvas.index(1).max(eps_tol()) / resize.height).ceil() * resize.height;

    ((next_page_vertical_border - pos_left_top_canvas.index(0)) / initial_size_image.x)
        .min((next_page_horizontal_border - pos_left_top_canvas.index(1)) / initial_size_image.y)
        .max(eps_tol())
}
