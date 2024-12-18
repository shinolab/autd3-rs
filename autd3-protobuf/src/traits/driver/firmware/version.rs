use crate::{pb::*, traits::FromMessage, AUTDProtoBufError};

impl FromMessage<FirmwareVersionResponseLightweight>
    for Vec<autd3_driver::firmware::version::FirmwareVersion>
{
    fn from_msg(msg: &FirmwareVersionResponseLightweight) -> Result<Self, AUTDProtoBufError> {
        Ok(msg
            .firmware_version_list
            .iter()
            .enumerate()
            .map(|(i, v)| {
                autd3_driver::firmware::version::FirmwareVersion::new(
                    i as _,
                    autd3_driver::firmware::version::CPUVersion::new(
                        autd3_driver::firmware::version::Major(v.cpu_major_version as _),
                        autd3_driver::firmware::version::Minor(v.cpu_minor_version as _),
                    ),
                    autd3_driver::firmware::version::FPGAVersion::new(
                        autd3_driver::firmware::version::Major(v.fpga_major_version as _),
                        autd3_driver::firmware::version::Minor(v.fpga_minor_version as _),
                        v.fpga_function_bits as _,
                    ),
                )
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firmware_version() {
        let firmware_versions = vec![autd3_driver::firmware::version::FirmwareVersion::new(
            0,
            autd3_driver::firmware::version::CPUVersion::new(
                autd3_driver::firmware::version::Major(1),
                autd3_driver::firmware::version::Minor(2),
            ),
            autd3_driver::firmware::version::FPGAVersion::new(
                autd3_driver::firmware::version::Major(3),
                autd3_driver::firmware::version::Minor(4),
                5,
            ),
        )];
        let response = FirmwareVersionResponseLightweight {
            success: true,
            msg: String::new(),
            firmware_version_list: firmware_versions
                .iter()
                .map(|f| firmware_version_response_lightweight::FirmwareVersion {
                    cpu_major_version: f.cpu().major().0 as _,
                    cpu_minor_version: f.cpu().minor().0 as _,
                    fpga_major_version: f.fpga().major().0 as _,
                    fpga_minor_version: f.fpga().minor().0 as _,
                    fpga_function_bits: f.fpga().function_bits() as _,
                })
                .collect(),
        };
        assert_eq!(
            firmware_versions,
            Vec::<autd3_driver::firmware::version::FirmwareVersion>::from_msg(&response).unwrap()
        );
    }
}
