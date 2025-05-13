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
Useful functions for interacting with gRPC, including:
  * dummy server (only one method each running)
  * client
  * protobuf compiler
  * protobuf descriptor inspector
  * protobuf binary-json converter

Before running the program, you need to precompile the protobuf files into protobuf file descriptors.
you can either use the `protoc` command or the grpc-cli builtin command `compile` to do this.

```bash
$ grpc_cli compile -i ./proto ./proto/hello.proto
# output to "./output.desc", for more command line options, run see the help of `compile`
```
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
