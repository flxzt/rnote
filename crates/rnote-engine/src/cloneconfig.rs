/// Trait for types which hold and need to be able to clone only their configuration.
pub trait CloneConfig {
    fn clone_config(&self) -> Self;
}
