use std::{path::PathBuf, str::FromStr};

use argh::FromArgs;
use prost_reflect::Kind;

use super::Executable;
use crate::descriptor_set::DescriptorSet;

#[derive(Clone, Copy, Debug)]
enum Descriptor {
    Service,
    Message,
    Enum,
    Extension,
}

impl FromStr for Descriptor {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "service" => Ok(Descriptor::Service),
            "message" => Ok(Descriptor::Message),
            "enum" => Ok(Descriptor::Enum),
            "extension" => Ok(Descriptor::Extension),
            _ => Err(anyhow::anyhow!("unknown descriptor type")),
        }
    }
}

/// print detailed protobuf type info from the descriptor set
#[derive(FromArgs, Clone, Debug)]
#[argh(subcommand, name = "inspect")]
pub struct InspectCommand {
    /// the descriptor set file to inspect
    #[argh(positional)]
    descriptor_set: PathBuf,

    /// print only the descriptor type matching the regex rule
    /// could be `service`, `message`, `enum`, `extension`
    #[argh(option, short = 't', default = "Descriptor::Service")]
    descriptor_type_filter: Descriptor,

    /// print only the type name matching the regex rule
    #[argh(option, short = 'f')]
    name_fileter: Option<String>,
    // /// print only the type name
    // #[argh(switch, short = 'n')]
    // name_only: bool,
}

impl Executable for InspectCommand {
    fn run(&self) -> anyhow::Result<()> {
        let ds = DescriptorSet::from_file(&self.descriptor_set)?;
        let pool = ds.pool();

        let name_filter: Box<dyn Fn(&str) -> bool> = if let Some(pat) = &self.name_fileter {
            let pat = regex_lite::Regex::new(pat)?;

            Box::new(move |name: &str| pat.is_match(name))
        } else {
            Box::new(|_| true)
        };

        match self.descriptor_type_filter {
            Descriptor::Service => {
                for s in pool.services() {
                    for m in s.methods() {
                        if name_filter(m.full_name()) {
                            let cs_tag = if m.is_client_streaming() {
                                "stream "
                            } else {
                                ""
                            };

                            let ss_tag = if m.is_server_streaming() {
                                "stream "
                            } else {
                                ""
                            };

                            println!(
                                "{}({}{}) ({}{})",
                                m.full_name(),
                                cs_tag,
                                m.input().full_name(),
                                ss_tag,
                                m.output().full_name()
                            );
                        }
                    }
                }
            }
            Descriptor::Message => {
                for m in pool.all_messages() {
                    if name_filter(m.full_name()) {
                        let mut fields = Vec::new();
                        for f in m.fields() {
                            let kind = match f.kind() {
                                Kind::Message(m) => m.full_name().to_string(),
                                Kind::Enum(e) => e.full_name().to_string(),
                                k => format!("{k:?}"),
                            };
                            fields.push(format!("\t{} {} = {};\n", kind, f.name(), f.number()));
                        }
                        println!("{}{{\n{}}}", m.full_name(), fields.concat());
                    }
                }
            }
            Descriptor::Enum => {
                for e in pool.all_enums() {
                    if name_filter(e.full_name()) {
                        println!("{}", e.full_name());
                    }
                }
            }
            Descriptor::Extension => {
                for e in pool.all_extensions() {
                    if name_filter(e.full_name()) {
                        println!("{}", e.full_name());
                    }
                }
            }
        }

        Ok(())
    }
}
