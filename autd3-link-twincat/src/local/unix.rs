use autd3_core::link::Link;

/// A [`Link`] using TwinCAT3.
///
/// To use this link, you need to install TwinCAT3 and run [`TwinCATAUTDServer`] before.
///
/// [`TwinCATAUTDServer`]: https://github.com/shinolab/autd3-server
pub struct TwinCAT {}

impl TwinCAT {
    /// Creates a new [`TwinCAT`].
    pub fn new() -> Result<Self, autd3_core::link::LinkError> {
        Err(autd3_core::link::LinkError::new(
            "TwinCAT is only supported on Windows",
        ))
    }
}

impl Link for TwinCAT {
    fn open(
        &mut self,
        _geometry: &autd3_core::geometry::Geometry,
    ) -> Result<(), autd3_core::link::LinkError> {
        Err(autd3_core::link::LinkError::new(
            "TwinCAT is only supported on Windows",
        ))
    }

    fn close(&mut self) -> Result<(), autd3_core::link::LinkError> {
        Err(autd3_core::link::LinkError::new(
            "TwinCAT is only supported on Windows",
        ))
    }

    fn send(
        &mut self,
        _tx: Vec<autd3_core::link::TxMessage>,
    ) -> Result<(), autd3_core::link::LinkError> {
        Err(autd3_core::link::LinkError::new(
            "TwinCAT is only supported on Windows",
        ))
    }

    fn update(
        &mut self,
        _geometry: &autd3_core::geometry::Geometry,
    ) -> Result<(), autd3_core::link::LinkError> {
        Err(autd3_core::link::LinkError::new(
            "TwinCAT is only supported on Windows",
        ))
    }

    fn alloc_tx_buffer(
        &mut self,
    ) -> Result<Vec<autd3_core::link::TxMessage>, autd3_core::link::LinkError> {
        Err(autd3_core::link::LinkError::new(
            "TwinCAT is only supported on Windows",
        ))
    }

    fn receive(
        &mut self,
        _rx: &mut [autd3_core::link::RxMessage],
    ) -> Result<(), autd3_core::link::LinkError> {
        Err(autd3_core::link::LinkError::new(
            "TwinCAT is only supported on Windows",
        ))
    }

    fn is_open(&self) -> bool {
        false
    }
}

impl autd3_core::link::AsyncLink for TwinCAT {}
