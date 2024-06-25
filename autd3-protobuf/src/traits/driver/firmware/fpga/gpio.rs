use crate::pb::*;

impl From<GpioIn> for autd3_driver::firmware::fpga::GPIOIn {
    fn from(value: GpioIn) -> Self {
        match value {
            GpioIn::I0 => Self::I0,
            GpioIn::I1 => Self::I1,
            GpioIn::I2 => Self::I2,
            GpioIn::I3 => Self::I3,
        }
    }
}

impl From<autd3_driver::firmware::fpga::GPIOIn> for GpioIn {
    fn from(value: autd3_driver::firmware::fpga::GPIOIn) -> Self {
        match value {
            autd3_driver::firmware::fpga::GPIOIn::I0 => Self::I0,
            autd3_driver::firmware::fpga::GPIOIn::I1 => Self::I1,
            autd3_driver::firmware::fpga::GPIOIn::I2 => Self::I2,
            autd3_driver::firmware::fpga::GPIOIn::I3 => Self::I3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpioin() {
        {
            let v = autd3_driver::firmware::fpga::GPIOIn::I0;
            let msg: GpioIn = v.into();
            let v2: autd3_driver::firmware::fpga::GPIOIn = msg.into();
            assert_eq!(v, v2);
        }

        {
            let v = autd3_driver::firmware::fpga::GPIOIn::I1;
            let msg: GpioIn = v.into();
            let v2: autd3_driver::firmware::fpga::GPIOIn = msg.into();
            assert_eq!(v, v2);
        }

        {
            let v = autd3_driver::firmware::fpga::GPIOIn::I2;
            let msg: GpioIn = v.into();
            let v2: autd3_driver::firmware::fpga::GPIOIn = msg.into();
            assert_eq!(v, v2);
        }

        {
            let v = autd3_driver::firmware::fpga::GPIOIn::I3;
            let msg: GpioIn = v.into();
            let v2: autd3_driver::firmware::fpga::GPIOIn = msg.into();
            assert_eq!(v, v2);
        }
    }
}
