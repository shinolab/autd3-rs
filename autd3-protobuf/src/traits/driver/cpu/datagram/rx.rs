/*
 * File: rx.rs
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

impl ToMessage for Vec<autd3_driver::cpu::RxMessage> {
    type Message = RxMessage;

    fn to_msg(&self) -> Self::Message {
        let mut data = vec![0; std::mem::size_of::<autd3_driver::cpu::RxMessage>() * self.len()];
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.as_ptr() as *const u8,
                data.as_mut_ptr(),
                data.len(),
            );
        }
        Self::Message { data }
    }
}

impl FromMessage<RxMessage> for Vec<autd3_driver::cpu::RxMessage> {
    fn from_msg(msg: &RxMessage) -> Option<Self> {
        let mut rx = vec![
            autd3_driver::cpu::RxMessage { ack: 0, data: 0 };
            msg.data.len() / std::mem::size_of::<autd3_driver::cpu::RxMessage>()
        ];
        unsafe {
            std::ptr::copy_nonoverlapping(msg.data.as_ptr(), rx.as_mut_ptr() as _, msg.data.len());
        }
        Some(rx)
    }
}
