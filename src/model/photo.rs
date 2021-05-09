use std::path::Path;
use std::fs::Metadata;
use std::fmt;
use std::fmt::Formatter;

pub struct Photo {
    pub(crate) name: String,
    pub(crate) path: Box<Path>,
    pub(crate) content: Vec<u8>,
    pub(crate) metadata: Metadata
}

impl fmt::Display for Photo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "(name={}, len={})", self.name, self.metadata.len())
    }
}