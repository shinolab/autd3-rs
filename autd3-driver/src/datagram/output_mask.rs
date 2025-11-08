use std::convert::Infallible;

use autd3_core::{
    datagram::{Datagram, DatagramOption, DeviceMask},
    environment::Environment,
    firmware::Segment,
    geometry::{Device, Geometry, Transducer},
};

/// [`Datagram`] to mask output.
///
/// The transducers set to `false` in [`OutputMask`] will not output ultrasound regardless of the intensity settings by [`Gain`], [`FociSTM`], and [`GainSTM`].
///
/// # Example
///
/// ```
/// # use autd3_driver::datagram::OutputMask;
/// OutputMask::new(|_dev| |_tr| true);
/// ```
///
/// [`Datagram`]: autd3_core::datagram::Datagram
/// [`Gain`]: autd3_core::gain::Gain
/// [`FociSTM`]: crate::datagram::FociSTM
/// [`GainSTM`]: crate::datagram::GainSTM
#[derive(Debug)]
pub struct OutputMask<F, FT> {
    #[doc(hidden)]
    pub f: F,
    /// The segment to which this [`OutputMask`] applies.
    pub segment: Segment,
    _phantom: std::marker::PhantomData<FT>,
}

impl<'a, FT: Fn(&'a Transducer) -> bool, F: Fn(&'a Device) -> FT> OutputMask<F, FT> {
    /// Creates a new [`OutputMask`].
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self::with_segment(f, Segment::S0)
    }

    /// Creates a new [`OutputMask`] for a segment.
    pub const fn with_segment(f: F, segment: Segment) -> Self {
        Self {
            f,
            segment,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct OutputMaskOperationGenerator<F> {
    pub(crate) f: F,
    pub(crate) segment: Segment,
}

impl<'a, FT: Fn(&'a Transducer) -> bool, F: Fn(&'a Device) -> FT> Datagram<'a>
    for OutputMask<F, FT>
{
    type G = OutputMaskOperationGenerator<F>;
    type Error = Infallible;

    fn operation_generator(
        self,
        _: &'a Geometry,
        _: &Environment,
        _: &DeviceMask,
    ) -> Result<Self::G, Self::Error> {
        Ok(OutputMaskOperationGenerator {
            f: self.f,
            segment: self.segment,
        })
    }

    fn option(&self) -> DatagramOption {
        DatagramOption::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let datagram = OutputMask::new(|_dev| |_tr| true);
        assert_eq!(datagram.segment, Segment::S0);
    }
}
