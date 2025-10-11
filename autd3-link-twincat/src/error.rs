use autd3_core::link::LinkError;

#[derive(Debug)]
#[non_exhaustive]
pub enum AdsError {
    DllNotFound,
    FunctionNotFound(String),
    OpenPort,
    GetLocalAddress(i32),
    SendData(i32),
    ReadData(i32),
}

impl std::fmt::Display for AdsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdsError::DllNotFound => write!(f, "TcAdsDll not found. Please install TwinCAT3."),
            AdsError::FunctionNotFound(name) => {
                write!(f, "Function {} not found in TcAdsDll", name)
            }
            AdsError::OpenPort => write!(f, "Failed to open port"),
            AdsError::GetLocalAddress(code) => write!(f, "Failed to get local address: {}", code),
            AdsError::SendData(code) => write!(f, "Failed to send data: {}", code),
            AdsError::ReadData(code) => write!(f, "Failed to read data: {}", code),
        }
    }
}

impl std::error::Error for AdsError {}

impl From<AdsError> for LinkError {
    fn from(err: AdsError) -> Self {
        LinkError::new(err)
    }
}
