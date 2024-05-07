use crate::{
    error::AUTDInternalError,
    firmware::operation::{cast, Operation, TypeTag},
    geometry::{Device, Geometry},
};

use super::Remains;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct GPIOInFlags(u8);

bitflags::bitflags! {
    impl GPIOInFlags : u8 {
        const NONE      = 0;
        const GPIO_IN_0 = 1 << 0;
        const GPIO_IN_1 = 1 << 1;
        const GPIO_IN_2 = 1 << 2;
        const GPIO_IN_3 = 1 << 3;
    }
}

#[repr(C, align(2))]
struct EmulateGPIOIn {
    tag: TypeTag,
    flag: GPIOInFlags,
}

pub struct EmulateGPIOInOp<F: Fn(&Device) -> [bool; 4]> {
    remains: Remains,
    f: F,
}

impl<F: Fn(&Device) -> [bool; 4]> EmulateGPIOInOp<F> {
    pub fn new(f: F) -> Self {
        Self {
            remains: Default::default(),
            f,
        }
    }
}

impl<F: Fn(&Device) -> [bool; 4]> Operation for EmulateGPIOInOp<F> {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        let gpio_in = (self.f)(device);
        let mut flag = GPIOInFlags::NONE;
        flag.set(GPIOInFlags::GPIO_IN_0, gpio_in[0]);
        flag.set(GPIOInFlags::GPIO_IN_1, gpio_in[1]);
        flag.set(GPIOInFlags::GPIO_IN_2, gpio_in[2]);
        flag.set(GPIOInFlags::GPIO_IN_3, gpio_in[3]);

        *cast::<EmulateGPIOIn>(tx) = EmulateGPIOIn {
            tag: TypeTag::EmulateGPIOIn,
            flag,
        };

        self.remains[device] -= 1;
        Ok(std::mem::size_of::<EmulateGPIOIn>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<EmulateGPIOIn>()
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.remains.init(geometry, |_| 1);
        Ok(())
    }

    fn is_done(&self, device: &Device) -> bool {
        self.remains.is_done(device)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{defined::FREQ_40K, geometry::tests::create_geometry};

    const NUM_TRANS_IN_UNIT: usize = 249;
    const NUM_DEVICE: usize = 10;

    #[test]
    fn test() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT, FREQ_40K);

        let mut tx = [0x00u8; 2 * NUM_DEVICE];

        let mut op = EmulateGPIOInOp::new(|dev| match dev.idx() {
            0 => [true, false, false, true],
            _ => [false, true, true, false],
        });

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
            assert_eq!(tx[dev.idx() * 2], TypeTag::EmulateGPIOIn as u8);
            assert_eq!(
                tx[dev.idx() * 2 + 1],
                if dev.idx() == 0 { 0b1001 } else { 0b0110 }
            );
        });
    }
}
