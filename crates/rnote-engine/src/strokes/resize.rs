use na;

#[derive(Debug)]
pub struct Resize {
    // size of a page
    pub width: f64,
    // if the layout has a fixed size vertically
    pub isfixed_layout: bool,
    // viewport
    pub max_viewpoint: na::OPoint<f64, na::Const<2>>,
}

pub fn calculate_resize(
    resize: Resize,
    initial_size: na::Vector2<f64>,
    pos: na::Vector2<f64>,
) -> f64 {
    let current_width = initial_size.x;
    let current_height = initial_size.y;

    // calculate the minimum ratio to stay in the viewport
    let ratio_viewport = (1.0f64)
        .min(
            //check in the horizontal direction
            (resize.max_viewpoint.x - pos.index(0)) / (current_width),
        )
        .min(
            //check in the vertical direction
            (resize.max_viewpoint.y - pos.index(1)) / (current_height),
        );

    // check if we go out of the viewport in the two directions
    match resize.isfixed_layout {
        false => ratio_viewport,
        true => ratio_viewport.min((resize.width - pos.index(0)) / (current_width)),
    }
}
