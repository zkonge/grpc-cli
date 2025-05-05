mod client;
mod compile;
mod inspect;
mod server;
mod version;

use argh::FromArgs;

/// Command line arguments for the application.
#[derive(FromArgs, Clone, Debug)]
#[argh(subcommand)]
pub enum Command {
    Compile(compile::CompileCommand),
    Inspect(inspect::InspectCommand),
    // Server(ServerCommand),
    Client(client::ClientCommand),
    Version(version::VersionCommand),
}

pub trait Executable {
    fn run(&self) -> anyhow::Result<()>;
}
