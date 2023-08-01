#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "clap-derive", derive(clap::ValueEnum))]
pub enum SelectionCollision {
    #[default]
    /// All strokes completely inside the area
    Contains,
    /// All Strokes intersecting with the area
    Intersects,
}

#[cfg(feature = "clap-derive")]
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
