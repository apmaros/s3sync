type GenError = Box<dyn std::error::Error>;

#[derive(Debug)]
struct SyncError(String);
impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for SyncError {}