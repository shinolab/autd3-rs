use crate::{datagram::*, derive::*, firmware::operation::ReadsFPGAStateOp};

#[derive(Builder)]
pub struct ReadsFPGAState<F: Fn(&Device) -> bool> {
    #[get]
    f: F,
}

impl<F: Fn(&Device) -> bool> ReadsFPGAState<F> {
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

pub struct ReadsFPGAStateOpGenerator<F: Fn(&Device) -> bool> {
    f: F,
}

impl<F: Fn(&Device) -> bool> OperationGenerator for ReadsFPGAStateOpGenerator<F> {
    type O1 = ReadsFPGAStateOp;
    type O2 = NullOp;

    fn generate(&self, device: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new((self.f)(device)), Self::O2::default())
    }
}

impl<F: Fn(&Device) -> bool> Datagram for ReadsFPGAState<F> {
    type G = ReadsFPGAStateOpGenerator<F>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(ReadsFPGAStateOpGenerator { f: self.f })
    }

    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }

    #[tracing::instrument(level = "debug", skip(self, geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
        if tracing::enabled!(tracing::Level::DEBUG) {
            geometry
                .devices()
                .for_each(|dev| tracing::debug!("Device[{}]: {}", dev.idx(), (self.f)(dev)));
        }
    }
    // GRCOV_EXCL_STOP
}

#[cfg(feature = "capi")]
impl Default for ReadsFPGAState<Box<dyn Fn(&Device) -> bool>> {
    fn default() -> Self {
        Self::new(Box::new(|_| false))
    }
}
