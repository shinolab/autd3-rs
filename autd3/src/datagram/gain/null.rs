use autd3_driver::derive::*;

#[derive(Gain, Default, Clone, PartialEq, Eq, Debug)]
pub struct Null {}

impl Null {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Gain for Null {
    fn calc(&self, _geometry: &Geometry) -> GainCalcResult {
        Ok(Self::transform(|_| |_| Drive::null()))
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
        let b = g.calc(&geometry)?;
        geometry.iter().for_each(|dev| {
            let d = b(dev);
            dev.iter().for_each(|tr| {
                assert_eq!(Drive::null(), d(tr));
            });
        });

        Ok(())
    }
}
