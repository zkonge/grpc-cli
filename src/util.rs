use std::{
    fs::{self, File},
    io,
    path::Path,
};

use flate2::read::GzDecoder;
use prost::Message;
use prost_reflect::prost_types::FileDescriptorSet;

pub fn load_descriptor_set_from_path(p: &Path) -> anyhow::Result<FileDescriptorSet> {
    let desc = if p.extension() == Some("gz".as_ref()) {
        let mut buf = Vec::new();
        let f = File::open(p)?;

        io::copy(&mut GzDecoder::new(f), &mut buf)?;

        buf
    } else {
        fs::read(p)?
    };

    let file_descriptor_set = FileDescriptorSet::decode(desc.as_slice())?;

    Ok(file_descriptor_set)
}

pub fn tokio_rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
