/*
 * File: nop.rs
 * Project: link
 * Created Date: 17/01/2024
 * Author: Shun Suzuki
 * -----
 * Last Modified: 17/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2024 Shun Suzuki. All rights reserved.
 *
 */

use autd3::prelude::*;

#[tokio::test]
async fn nop_test() {
    let mut autd = Controller::builder()
        .add_device(AUTD3::new(Vector3::zeros()))
        .open_with(Nop::builder().with_timeout(std::time::Duration::from_millis(100)))
        .await
        .unwrap();

    assert_eq!(autd.link.timeout(), std::time::Duration::from_millis(100));

    assert_eq!(autd.send(Static::new()).await, Ok(true));

    assert_eq!(autd.close().await, Ok(true));

    assert_eq!(
        autd.send(Static::new()).await,
        Err(AUTDError::Internal(AUTDInternalError::LinkClosed))
    );
    assert_eq!(
        autd.fpga_info().await,
        Err(AUTDError::Internal(AUTDInternalError::LinkClosed))
    );
}
