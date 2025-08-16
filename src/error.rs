#[derive(Debug)]
pub enum Error {
    Parse(serde_json::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: This is a dummy implementation
        write!(f, "{self:?}")
    }
}

impl core::error::Error for Error {}
