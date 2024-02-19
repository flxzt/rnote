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

/// Helper function to calculate ratios
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
    let current_width = initial_size_image.x;
    let current_height = initial_size_image.y;

    // compile all ratio in a vec
    let ratios = vec![
        // check that we do not go out of the canvas view in the x direction
        helper_calculate_fit_ratio(
            &resize.max_viewpoint.x,
            &pos_left_top_canvas.index(0),
            &current_width,
        ),
        // check that we do not go out of view in the y direction
        helper_calculate_fit_ratio(
            &resize.max_viewpoint.y,
            &pos_left_top_canvas.index(1),
            &current_height,
        ),
        // check if we go out of the page on the right on fixed layout
        helper_calculate_fit_ratio(&resize.width, &pos_left_top_canvas.index(0), &current_width),
        // check if we have to respect borders
        calculate_resize_ratio_respect_borders(&resize, &initial_size_image, &pos_left_top_canvas),
    ];

    // apply rules
    let apply_ratios = vec![
        true,                   //canvas in the x direction
        true,                   //canvas in the y direction
        resize.isfixed_layout,  //do not go over the page on the right for fixed layout
        resize.respect_borders, //do not go over the page on the right for all layouts (slightly redundant)
    ];

    ratios
        .iter()
        .zip(apply_ratios)
        .filter(|x| x.1)
        .fold(1.0f64, |acc, x| acc.min(*x.0))
        .max(1e-8f64) //force the value to be positive as a zero would incurr crashes when applying the transforms
}

/// calculate the ratio to not go over borders
pub fn calculate_resize_ratio_respect_borders(
    resize: &Resize,
    initial_size_image: &na::Vector2<f64>,
    pos_left_top_canvas: &na::Vector2<f64>,
) -> f64 {
    // closure to calculate the ceil
    // We take the `floor (ratio + eps) + 1``
    // This allows for element that would fall exactly
    // on a border to be on the next page and disallow
    // too small ratios
    let f_ratio = |position: &f64, size: &f64| -> f64 {
        (((position / size) + 1e-8f64).floor() + 1.0f64) * size
    };

    let next_page_vertical_border = f_ratio(&pos_left_top_canvas.index(0), &resize.width);
    let next_page_horizontal_border = f_ratio(&pos_left_top_canvas.index(1), &resize.height);

    let ratios = vec![
        helper_calculate_fit_ratio(
            &next_page_vertical_border,
            &pos_left_top_canvas.index(0),
            &initial_size_image.x,
        ),
        helper_calculate_fit_ratio(
            &next_page_horizontal_border,
            &pos_left_top_canvas.index(1),
            &initial_size_image.y,
        ),
    ];

    let rule_apply = vec![
        true, // vertical rule
        true, // horizontal rule
    ];

    ratios
        .iter()
        .zip(rule_apply)
        .fold(1.0f64, |acc, x| acc.min(*x.0))
        .max(1e-8f64)
}
