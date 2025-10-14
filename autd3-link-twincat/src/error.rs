use autd3_core::link::LinkError;

#[derive(Debug)]
#[non_exhaustive]
pub enum AdsError {
    DllNotFound,
    FunctionNotFound(String),
    OpenPort,
    ClosePort,
    DeviceInvalidSize,
    GetLocalAddress(i32),
    AmsNetIdParse,
    AmsAddRoute(i32),
    SendData(i32),
    ReadData(i32),
    InvalidIp(String),
    #[cfg(feature = "remote")]
    Ads(ads::Error),
}

impl std::fmt::Display for AdsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdsError::DllNotFound => write!(f, "TcAdsDll not found. Please install TwinCAT3."),
            AdsError::FunctionNotFound(name) => {
                write!(f, "Function {} not found in TcAdsDll", name)
            }
            AdsError::OpenPort => write!(f, "Failed to open port"),
            AdsError::ClosePort => write!(f, "Failed to close port"),
            AdsError::DeviceInvalidSize => write!(f, "The number of devices is invalid"),
            AdsError::GetLocalAddress(code) => write!(f, "Failed to get local address: {}", code),
            AdsError::AmsNetIdParse => write!(f, "Ams net id must have 6 octets"),
            AdsError::AmsAddRoute(code) => write!(f, "Failed to add route: {}", code),
            AdsError::SendData(code) => write!(f, "Failed to send data: {}", code),
            AdsError::ReadData(code) => write!(f, "Failed to read data: {}", code),
            AdsError::InvalidIp(ip) => write!(f, "Invalid IP address: {}", ip),
            #[cfg(feature = "remote")]
            AdsError::Ads(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for AdsError {}

impl From<AdsError> for LinkError {
    fn from(err: AdsError) -> Self {
        LinkError::new(err)
    }
}

#[cfg(feature = "remote")]
impl From<ads::Error> for AdsError {
    fn from(err: ads::Error) -> Self {
        AdsError::Ads(err)
    }
}
