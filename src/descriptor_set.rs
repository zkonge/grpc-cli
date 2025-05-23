use std::{fs, io, path::Path};

use prost::Message;
use prost_reflect::{DescriptorPool, prost_types::FileDescriptorSet};

pub struct DescriptorSet {
    file_descriptor_set: FileDescriptorSet,
    descriptor_pool: DescriptorPool,
}

impl DescriptorSet {
    pub fn from_file(p: &Path) -> io::Result<Self> {
        let raw_file_descriptor_set = fs::read(p)?;

        let file_descriptor_set = FileDescriptorSet::decode(raw_file_descriptor_set.as_slice())?;
        let descriptor_pool = DescriptorPool::from_file_descriptor_set(file_descriptor_set.clone())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(Self {
            file_descriptor_set,
            descriptor_pool,
        })
    }

    pub fn to_file(&self, p: &Path) -> io::Result<()> {
        let desc = self.file_descriptor_set.encode_to_vec();

        fs::write(p, desc)?;

        Ok(())
    }

    pub fn compile(
        files: impl IntoIterator<Item = impl AsRef<Path>>,
        includes: impl IntoIterator<Item = impl AsRef<Path>>,
    ) -> anyhow::Result<Self> {
        let file_descriptor_set = protox::compile(files, includes)?;
        let descriptor_pool = DescriptorPool::from_file_descriptor_set(file_descriptor_set.clone())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(Self {
            file_descriptor_set,
            descriptor_pool,
        })
    }

    pub fn pool(&self) -> DescriptorPool {
        self.descriptor_pool.clone()
    }
}
