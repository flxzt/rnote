//! rnote-cli
//!
//! The cli interface is not stable (yet) and could change at any time.

pub mod cli;

fn main() -> anyhow::Result<()> {
    println!("Entering rnote-cli");

    smol::block_on(async { cli::run().await })
}
