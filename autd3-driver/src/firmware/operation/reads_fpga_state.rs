use crate::{
    error::AUTDInternalError,
    firmware::operation::{cast, Operation, TypeTag},
    geometry::{Device, Geometry},
};

use super::Remains;

#[repr(C, align(2))]
struct ConfigureReadsFPGAState {
    tag: TypeTag,
    value: bool,
}

pub struct ConfigureReadsFPGAStateOp<F: Fn(&Device) -> bool> {
    remains: Remains,
    f: F,
}

impl<F: Fn(&Device) -> bool> ConfigureReadsFPGAStateOp<F> {
    pub fn new(f: F) -> Self {
        Self {
            remains: Default::default(),
            f,
        }
    }
}

impl<F: Fn(&Device) -> bool> Operation for ConfigureReadsFPGAStateOp<F> {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        *cast::<ConfigureReadsFPGAState>(tx) = ConfigureReadsFPGAState {
            tag: TypeTag::ReadsFPGAState,
            value: (self.f)(device),
        };

        self.remains[device] -= 1;
        Ok(std::mem::size_of::<ConfigureReadsFPGAState>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<ConfigureReadsFPGAState>()
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.remains.init(geometry, 1);
        Ok(())
    }

    fn is_done(&self, device: &Device) -> bool {
        self.remains.is_done(device)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::tests::create_geometry;

    const NUM_TRANS_IN_UNIT: usize = 249;
    const NUM_DEVICE: usize = 10;

    #[test]
    fn test() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; 2 * NUM_DEVICE];

        let mut op = ConfigureReadsFPGAStateOp::new(|dev| dev.idx() == 0);

        assert!(op.init(&geometry).is_ok());

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.required_size(dev), 2));

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains[dev], 1));

        geometry.devices().for_each(|dev| {
            assert!(op.pack(dev, &mut tx[dev.idx() * 2..]).is_ok());
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains[dev], 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * 2], TypeTag::ReadsFPGAState as u8);
            assert_eq!(
                tx[dev.idx() * 2 + 1],
                if dev.idx() == 0 { 0x01 } else { 0x00 }
            );
        });
    }
}
