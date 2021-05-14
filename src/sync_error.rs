use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub(crate) struct SyncError(
    pub(crate) String
);
impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for SyncError {}