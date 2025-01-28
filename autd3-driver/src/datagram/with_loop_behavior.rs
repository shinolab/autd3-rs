use autd3_core::datagram::{
    Datagram, DatagramL, DatagramOption, LoopBehavior, Segment, TransitionMode,
};

use derive_more::Deref;
use derive_new::new;

/// A wrapper of [`DatagramL`] to specify the loop behavior.
///
/// Note that the loop behavior only affects when switching segments.
#[derive(Deref, Debug, Clone, Copy, PartialEq, Eq, Hash, new)]
pub struct WithLoopBehavior<D: DatagramL> {
    #[deref]
    /// The original [`DatagramL`]
    pub inner: D,
    /// The loop behavior
    pub loop_behavior: LoopBehavior,
    /// The segment to write the data
    pub segment: Segment,
    /// The behavior when switching segments
    pub transition_mode: Option<TransitionMode>,
}

impl<D: DatagramL> Datagram for WithLoopBehavior<D> {
    type G = D::G;
    type Error = D::Error;

    fn operation_generator(
        self,
        geometry: &autd3_core::derive::Geometry,
        parallel: bool,
    ) -> Result<Self::G, Self::Error> {
        <D as DatagramL>::operation_generator_with_loop_behavior(
            self.inner,
            geometry,
            parallel,
            self.segment,
            self.transition_mode,
            self.loop_behavior,
        )
    }

    fn option(&self) -> DatagramOption {
        <D as DatagramL>::option(&self.inner)
    }
}
