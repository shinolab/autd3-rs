/*
 * File: remote_twincat.rs
 * Project: src
 * Created Date: 22/05/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 26/11/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

mod tests;

use anyhow::Result;

use autd3::prelude::*;
use autd3_link_twincat::RemoteTwinCAT;

#[tokio::main]
async fn main() -> Result<()> {
    let autd = Controller::builder()
        .add_device(AUTD3::new(Vector3::zeros()))
        .open_with(RemoteTwinCAT::builder("0.0.0.0.0.0"))
        .await?;

    tests::run(autd).await
}
