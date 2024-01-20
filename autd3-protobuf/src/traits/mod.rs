/*
 * File: mod.rs
 * Project: traits
 * Created Date: 19/01/2024
 * Author: Shun Suzuki
 * -----
 * Last Modified: 20/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2024 Shun Suzuki. All rights reserved.
 *
 */

mod driver;

pub trait ToMessage {
    type Message: prost::Message;

    fn to_msg(&self) -> Self::Message;
}

pub trait FromMessage<T: prost::Message>
where
    Self: Sized,
{
    fn from_msg(msg: &T) -> Option<Self>;
}
