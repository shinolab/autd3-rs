use crate::{pb::*, traits::FromMessage};

impl FromMessage<FirmwareInfoResponseLightweight>
    for Vec<autd3_driver::firmware::firmware_version::FirmwareInfo>
{
    fn from_msg(msg: &FirmwareInfoResponseLightweight) -> Option<Self> {
        Some(
            msg.firmware_info_list
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    autd3_driver::firmware::firmware_version::FirmwareInfo::new(
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
