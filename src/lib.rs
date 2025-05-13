mod cmd;
mod codec;
mod descriptor_set;
mod json;
mod static_server;
mod tls;
mod util;

use argh::FromArgs;
use cmd::Executable;

use self::cmd::Command;

/**
Everything you need for interacting with gRPC, including:
  * dummy server
  * client
  * protobuf compiler
  * protobuf descriptor inspector
*/
#[derive(FromArgs, Clone, Debug)]
pub struct App {
    #[argh(subcommand)]
    /// command to run
    pub command: Command,
}

impl App {
    pub fn run() -> anyhow::Result<()> {
        let app: Self = argh::from_env();

        let cmd: &dyn Executable = match &app.command {
            Command::Compile(x) => x,
            Command::Inspect(x) => x,
            Command::Json(x) => x,
            Command::Client(x) => x,
            Command::Server(x) => x,
            Command::Version(x) => x,
        };

        cmd.run()
    }
}
