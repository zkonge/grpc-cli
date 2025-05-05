use argh::FromArgs;

use super::Executable;

/// Print the version of the application.
#[derive(FromArgs, Clone, Debug)]
#[argh(subcommand, name = "version")]
pub struct VersionCommand {}

impl Executable for VersionCommand {
    fn run(&self) -> anyhow::Result<()> {
        println!("v{}", env!("CARGO_PKG_VERSION"));
        Ok(())
    }
}
