use argh::FromArgs;

use super::Executable;

/// Print the version of the application.
#[derive(FromArgs, Clone, Debug)]
#[argh(subcommand, name = "json")]
pub struct JsonCommand {
    // /// the directory of the protobuf package.
    // #[argh(option, short = 'p')]
    // protobuf_include_dirs: Vec<PathBuf>,

    // /// the name of the protobuf message.
    // #[argh(option, short = 'm')]
    // protobuf_message: Option<String>,

    // /// the path to the protobuf file.
    // #[argh(positional)]
    // protobuf_files: Vec<PathBuf>,

    // /// reverse the conversion. input is JSON, output is protobuf.
    // #[argh(switch, short = 'j')]
    // json_to_protobuf: bool,

    // /// the path to the output file. leave empty to write to stdout.
    // #[argh(option, short = 'o')]
    // output: Option<PathBuf>,

    // /// the path to the input file. leave empty to read from stdin.
    // #[argh(option, short = 'i')]
    // input: Option<PathBuf>,
}


impl Executable for JsonCommand {
    fn run(&self) -> anyhow::Result<()> {
        println!("v{}", env!("CARGO_PKG_VERSION"));
        Ok(())
    }
}


// fn json_to_protobuf(json_msg: &[u8], md: MessageDescriptor) -> miette::Result<Vec<u8>> {
//     let mut de = serde_json::de::Deserializer::from_slice(json_msg);
//     let proto_msg =
//         DynamicMessage::deserialize(md, &mut de).expect("Failed to deserialize JSON to protobuf");
//     de.end().expect("Failed to deserialize JSON to protobuf");

//     Ok(proto_msg.encode_to_vec())
// }

// fn protobuf_to_json(protobuf_msg: &[u8], md: MessageDescriptor) -> miette::Result<String> {
//     let proto_msg = DynamicMessage::decode(md, protobuf_msg).expect("Failed to decode protobuf");
//     let json_msg = serde_json::to_string(&proto_msg).expect("Failed to serialize protobuf to JSON");

//     Ok(json_msg)
// }