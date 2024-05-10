use crate::{pb::*, traits::FromMessage};

impl FromMessage<FirmwareVersionResponseLightweight>
    for Vec<autd3_driver::firmware::version::FirmwareVersion>
{
    fn from_msg(msg: &FirmwareVersionResponseLightweight) -> Option<Self> {
        Some(
            msg.firmware_version_list
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    autd3_driver::firmware::version::FirmwareVersion::new(
                        i as _,
                        v.cpu_major_version as _,
                        v.cpu_minor_version as _,
                        v.fpga_major_version as _,
                        v.fpga_minor_version as _,
                        v.fpga_function_bits as _,
                    )
                })
                .collect(),
        )
    }
}
