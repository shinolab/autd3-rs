use autd3_driver::{derive::*, firmware::fpga::Drive};
use derive_new::new;

#[derive(Gain, Default, Clone, PartialEq, Eq, Debug, new)]
pub struct Null {}

impl GainContext for Null {
    fn calc(&self, _: &Transducer) -> Drive {
        Drive::null()
    }
}

impl GainContextGenerator for Null {
    type Context = Null;

    fn generate(&mut self, _: &Device) -> Self::Context {
        Null {}
    }
}

impl Gain for Null {
    type G = Null;

    fn init_with_filter(
        self,
        _geometry: &Geometry,
        _filter: Option<HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::G, AUTDInternalError> {
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::create_geometry;

    use super::*;

    #[test]
    fn test_null() -> anyhow::Result<()> {
        let geometry = create_geometry(1);

        let g = Null::new();
        let mut b = g.init(&geometry)?;
        geometry.iter().for_each(|dev| {
            let d = b.generate(dev);
            dev.iter().for_each(|tr| {
                assert_eq!(Drive::null(), d.calc(tr));
            });
        });

        Ok(())
    }
}
