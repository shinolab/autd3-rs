use autd3_core::datagram::{
    Datagram, DatagramL, DatagramOption, LoopBehavior, Segment, TransitionMode,
};

use derive_more::Deref;

#[derive(Deref, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WithLoopBehavior<D: DatagramL> {
    #[deref]
    pub inner: D,
    pub loop_behavior: LoopBehavior,
    pub segment: Segment,
    pub transition_mode: Option<TransitionMode>,
}

impl<D: DatagramL> Datagram for WithLoopBehavior<D> {
    type G = D::G;
    type Error = D::Error;

    fn operation_generator(
        self,
        geometry: &autd3_core::derive::Geometry,
        option: &autd3_core::derive::DatagramOption,
    ) -> Result<Self::G, Self::Error> {
        <D as DatagramL>::operation_generator_with_loop_behavior(
            self.inner,
            geometry,
            option,
            self.segment,
            self.transition_mode,
            self.loop_behavior,
        )
    }

    fn option(&self) -> DatagramOption {
        <D as DatagramL>::option(&self.inner)
    }
}
