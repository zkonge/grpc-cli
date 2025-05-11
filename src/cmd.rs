mod client;
mod compile;
mod inspect;
mod json;
mod server;
mod version;

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
