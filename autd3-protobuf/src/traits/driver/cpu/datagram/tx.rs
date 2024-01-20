/*
 * File: tx.rs
 * Project: datagram
 * Created Date: 19/01/2024
 * Author: Shun Suzuki
 * -----
 * Last Modified: 20/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2024 Shun Suzuki. All rights reserved.
 *
 */

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::cpu::TxDatagram {
    type Message = TxRawData;

    fn to_msg(&self) -> Self::Message {
        Self::Message {
            data: self.all_data().to_vec(),
            num_devices: self.num_devices() as _,
        }
    }
}

impl FromMessage<TxRawData> for autd3_driver::cpu::TxDatagram {
    fn from_msg(msg: &TxRawData) -> Option<Self> {
        let mut tx = autd3_driver::cpu::TxDatagram::new(msg.num_devices as usize);
        unsafe {
            std::ptr::copy_nonoverlapping(
                msg.data.as_ptr(),
                tx.all_data_mut().as_mut_ptr(),
                msg.data.len(),
            );
        }
        Some(tx)
    }
}
