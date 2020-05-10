#[derive(Debug)]
pub enum ClientError {
    NoApiKey,
    RegistrationError,
    Io(std::io::Error),
    TonicStatus(tonic::Status),
    TonicTransport(tonic::transport::Error),
    BadArgument,
}

impl std::error::Error for ClientError {}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl From<std::io::Error> for ClientError {
    fn from(e: std::io::Error) -> Self {
        ClientError::Io(e)
    }
}

impl From<tonic::Status> for ClientError {
    fn from(e: tonic::Status) -> Self {
        ClientError::TonicStatus(e)
    }
}

impl From<tonic::transport::Error> for ClientError {
    fn from(e: tonic::transport::Error) -> Self {
        ClientError::TonicTransport(e)
    }
}
