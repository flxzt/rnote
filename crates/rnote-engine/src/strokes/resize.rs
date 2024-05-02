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
    /// Resize the image with various constraints
    ResizeImage(Resize),
}

#[derive(Debug)]
pub struct Resize {
    /// width of a page
    pub width: f64,
    /// height of a page
    pub height: f64,
    /// if the layout has a fixed size vertically
    pub layout_fixed_width: bool,
    /// viewport
    pub max_viewpoint: Option<na::OPoint<f64, na::Const<2>>>,
    /// resize to the viewport
    pub restrain_to_viewport: bool,
    /// To force elements to not go over borders
    /// maybe enabling that to be on only when borders are active
    /// would be a better idea
    pub respect_borders: bool,
}

/// helper functions for calculating resizing factors

/// Calculate where the next border of the page is
/// based on the current `position` and the `size` of
/// the page length
///
/// in conjunction with the the ratio min value, may
/// fail if the position is very close to a page border
fn helper_calculate_page_next_limit(position: &f64, size: &f64) -> f64 {
    ((position / size).floor() + 1.0f64) * size
}

/// Helper function to calculate ratios : min ratio for
/// the image to go from `current_position` to `current_size`
/// exactly
fn helper_calculate_fit_ratio(
    max_position: &f64,
    current_position: &f64,
    current_size: &f64,
) -> f64 {
    (max_position - current_position) / current_size
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
    let next_page_x = helper_calculate_page_next_limit(&pos_left_top_canvas.x, &resize.width);
    let next_page_y = helper_calculate_page_next_limit(&pos_left_top_canvas.y, &resize.height);

    // compile all ratio in a vec
    let ratios = [
        // check that we do not go out of the canvas view in the x direction
        helper_calculate_fit_ratio(
            &resize.max_viewpoint.unwrap_or(na::point![1.0, 1.0]).x,
            &pos_left_top_canvas.x,
            &initial_size_image.x,
        ),
        // check that we do not go out of view in the y direction
        helper_calculate_fit_ratio(
            &resize.max_viewpoint.unwrap_or(na::point![1.0, 1.0]).y,
            &pos_left_top_canvas.y,
            &initial_size_image.y,
        ),
        // check if we go out of the page on the right on fixed layout
        helper_calculate_fit_ratio(&resize.width, &pos_left_top_canvas.x, &initial_size_image.x),
        // check if we have to respect borders
        helper_calculate_fit_ratio(&next_page_y, &pos_left_top_canvas.y, &initial_size_image.y), // vertical border (cut in the y direction)
        helper_calculate_fit_ratio(&next_page_x, &pos_left_top_canvas.x, &initial_size_image.x), // horizontal border (cut in the x direction)
    ];

    let is_provided_viewport = resize.max_viewpoint.is_some();

    // apply rules
    let apply_ratios = vec![
        is_provided_viewport & resize.restrain_to_viewport, //canvas in the x direction
        is_provided_viewport & resize.restrain_to_viewport, //canvas in the y direction
        resize.layout_fixed_width, //do not go over the page on the right for fixed layout
        resize.respect_borders,    //do not go over the page on the bottom for all layouts
        resize.respect_borders,    //do not go over the page on the right for all layouts
    ];

    ratios
        .iter()
        .zip(apply_ratios)
        .filter(|x| x.1)
        .fold(1.0f64, |acc, x| acc.min(*x.0))
        .max(1e-15f64) //force the value to be positive as a zero would make transforms crash
}
