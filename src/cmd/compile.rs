use std::{
    fs::{self, File},
    io,
    path::PathBuf,
};

use argh::FromArgs;
use flate2::{Compression, bufread::GzEncoder};
use prost::Message;

use super::Executable;

/// Compile the proto files into a descriptor set file.
#[derive(FromArgs, Clone, Debug)]
#[argh(subcommand, name = "compile")]
pub struct CompileCommand {
    /// the proto file to compile
    #[argh(positional)]
    proto_file: PathBuf,

    /// more proto files to compile
    #[argh(positional)]
    more_proto_files: Vec<PathBuf>,

    /// the include DIR paths for the proto files, will be used in protobuf package resolution.
    /// Could be defined multiple times.
    #[argh(option, short = 'i')]
    includes: Vec<PathBuf>,

    /// the output file path for the compiled protobuf descriptor set file.
    /// If the path ends with `.gz`, the output will be compressed with gzip.
    /// The default is `output.desc`.
    #[argh(option, short = 'o', default = r#""output.desc".into()"#)]
    output: PathBuf,
}

impl Executable for CompileCommand {
    fn run(&self) -> anyhow::Result<()> {
        let desc = protox::compile(
            [&self.proto_file].into_iter().chain(&self.more_proto_files),
            &self.includes,
        )?
        .encode_to_vec();

        if self.output.extension() == Some("gz".as_ref()) {
            let mut compressed_stream = GzEncoder::new(desc.as_slice(), Compression::best());
            let mut f = File::create(&self.output)?;

            io::copy(&mut compressed_stream, &mut f)?;
        } else {
            fs::write(&self.output, desc)?;
        }

        Ok(())
    }
}
