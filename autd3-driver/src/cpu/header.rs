/*
 * File: header.rs
 * Project: cpu
 * Created Date: 02/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022-2023 Shun Suzuki. All rights reserved.
 *
 */

pub const MSG_ID_MAX: u8 = 0x7F;

#[repr(C)]
pub struct Header {
    pub msg_id: u8,
    _pad: u8,
    pub slot_2_offset: u16,
}
