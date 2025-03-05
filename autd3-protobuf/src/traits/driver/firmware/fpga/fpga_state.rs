use crate::{AUTDProtoBufError, pb::*, traits::FromMessage};

impl FromMessage<FpgaStateResponseLightweight>
    for Vec<Option<autd3_driver::firmware::fpga::FPGAState>>
{
    fn from_msg(msg: FpgaStateResponseLightweight) -> Result<Self, AUTDProtoBufError> {
        Ok(msg
            .fpga_state_list
            .iter()
            .map(|v| {
                v.state.map(|s| {
                    let state = s as u8;
                    unsafe { std::mem::transmute(state) }
                })
            })
            .collect())
    }
}
