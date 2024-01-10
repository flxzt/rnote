#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum SelectionCollision {
    #[default]
    /// All strokes completely inside the area
    Contains,
    /// All Strokes intersecting with the area
    Intersects,
}

impl std::fmt::Display for SelectionCollision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SelectionCollision::Contains => "contains",
                SelectionCollision::Intersects => "intersects",
            }
        )
    }
}
