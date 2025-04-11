//! rnote-cli
//!
//! The cli interface is not (yet) stable and could change at any time.

// Modules
pub(crate) mod cli;
pub(crate) mod export;
pub(crate) mod import;
pub(crate) mod mutate;
pub(crate) mod test;
pub(crate) mod thumbnail;
pub(crate) mod validators;

// Renames
extern crate nalgebra as na;
extern crate parry2d_f64 as p2d;

fn main() -> anyhow::Result<()> {
    smol::block_on(async { cli::run().await })
}
