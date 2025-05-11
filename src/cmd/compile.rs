use std::{iter::once, path::PathBuf};

use argh::FromArgs;

use super::Executable;
use crate::descriptor_set::DescriptorSet;

/// Compile the protobuf files into a descriptor set file.
#[derive(FromArgs, Clone, Debug)]
#[argh(subcommand, name = "compile")]
pub struct CompileCommand {
    /// the proto file to compile
    #[argh(positional)]
    file: PathBuf,

    /// more proto files to compile
    #[argh(positional)]
    more_files: Vec<PathBuf>,

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
        let ds = DescriptorSet::compile(once(&self.file).chain(&self.more_files), &self.includes)?;

        ds.to_file(&self.output).map_err(Into::into)
    }
}
