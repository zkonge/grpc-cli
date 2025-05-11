mod client;
mod compile;
mod inspect;
mod server;
mod version;
mod json;

use argh::FromArgs;

/// Command line arguments for the application.
#[derive(FromArgs, Clone, Debug)]
#[argh(subcommand)]
pub enum Command {
    Compile(compile::CompileCommand),
    Inspect(inspect::InspectCommand),
    Server(server::ServerCommand),
    Client(client::ClientCommand),
    Json(json::JsonCommand),
    Version(version::VersionCommand),
}

pub trait Executable {
    fn run(&self) -> anyhow::Result<()>;
}
