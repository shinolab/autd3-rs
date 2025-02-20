use crate::{AUTDProtoBufError, pb::*, traits::FromMessage};

impl FromMessage<FirmwareVersionResponseLightweight>
    for Vec<autd3_driver::firmware::version::FirmwareVersion>
{
    fn from_msg(msg: FirmwareVersionResponseLightweight) -> Result<Self, AUTDProtoBufError> {
        Ok(msg
            .firmware_version_list
            .iter()
            .enumerate()
            .map(|(i, v)| autd3_driver::firmware::version::FirmwareVersion {
                idx: i as _,
                cpu: autd3_driver::firmware::version::CPUVersion {
                    major: autd3_driver::firmware::version::Major(v.cpu_major_version as _),
                    minor: autd3_driver::firmware::version::Minor(v.cpu_minor_version as _),
                },
                fpga: autd3_driver::firmware::version::FPGAVersion {
                    major: autd3_driver::firmware::version::Major(v.fpga_major_version as _),
                    minor: autd3_driver::firmware::version::Minor(v.fpga_minor_version as _),
                    function_bits: v.fpga_function_bits as _,
                },
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firmware_version() {
        let firmware_versions = vec![autd3_driver::firmware::version::FirmwareVersion {
            idx: 0,
            cpu: autd3_driver::firmware::version::CPUVersion {
                major: autd3_driver::firmware::version::Major(1),
                minor: autd3_driver::firmware::version::Minor(2),
            },
            fpga: autd3_driver::firmware::version::FPGAVersion {
                major: autd3_driver::firmware::version::Major(3),
                minor: autd3_driver::firmware::version::Minor(4),
                function_bits: 5,
            },
        }];
        let response = FirmwareVersionResponseLightweight {
            err: false,
            msg: String::new(),
            firmware_version_list: firmware_versions
                .iter()
                .map(|f| firmware_version_response_lightweight::FirmwareVersion {
                    cpu_major_version: f.cpu.major.0 as _,
                    cpu_minor_version: f.cpu.minor.0 as _,
                    fpga_major_version: f.fpga.major.0 as _,
                    fpga_minor_version: f.fpga.minor.0 as _,
                    fpga_function_bits: f.fpga.function_bits as _,
                })
                .collect(),
        };
        assert_eq!(
            firmware_versions,
            Vec::<autd3_driver::firmware::version::FirmwareVersion>::from_msg(response).unwrap()
        );
    }
}
