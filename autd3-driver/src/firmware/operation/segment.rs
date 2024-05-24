pub trait SwapSegmentOperation {
    fn new(segment: Segment, transition_mode: TransitionMode) -> Self;
    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError>;
    fn required_size(&self, device: &Device) -> usize;
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError>;
    fn is_done(&self, device: &Device) -> bool;
}

impl<T: SwapSegmentOperation> Operation for T {
    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.init(geometry)
    }

    fn required_size(&self, device: &Device) -> usize {
        self.required_size(device)
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        self.pack(device, tx)
    }

    fn is_done(&self, device: &Device) -> bool {
        self.is_done(device)
    }
}
