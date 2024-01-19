/*
 * File: audit.rs
 * Project: link
 * Created Date: 17/01/2024
 * Author: Shun Suzuki
 * -----
 * Last Modified: 19/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2024 Shun Suzuki. All rights reserved.
 *
 */

use autd3::{link::Audit, prelude::*};
use autd3_driver::{cpu::RxMessage, fpga::FPGAState};

#[tokio::test]
async fn audit_test() {
    let mut autd = Controller::builder()
        .add_device(AUTD3::new(Vector3::zeros()))
        .open_with(Audit::builder().with_timeout(std::time::Duration::from_millis(100)))
        .await
        .unwrap();
    assert_eq!(autd.link.timeout(), std::time::Duration::from_millis(100));

    assert_eq!(autd.send(Null::new()).await, Ok(true));
    assert_eq!(
        autd.link.last_timeout(),
        std::time::Duration::from_millis(100)
    );
    assert_eq!(
        autd.send(Static::new().with_timeout(std::time::Duration::ZERO))
            .await,
        Ok(true)
    );
    assert_eq!(autd.link.last_timeout(), std::time::Duration::ZERO);

    assert_eq!(autd.link.emulators()[0].idx(), 0);
    assert_eq!(autd.link[0].idx(), 0);

    assert_eq!(autd.fpga_state().await, Ok(vec![None]));
    assert_eq!(
        autd.send(ConfigureReadsFPGAState::new(|_| true)).await,
        Ok(true)
    );
    autd.link[0].update();
    assert_eq!(
        autd.fpga_state().await,
        Ok(vec![Option::<FPGAState>::from(&RxMessage {
            data: 0x80,
            ack: 0x00
        })])
    );
    autd.link.emulators_mut()[0]
        .fpga_mut()
        .assert_thermal_sensor();
    autd.link[0].update();
    assert_eq!(
        autd.fpga_state().await,
        Ok(vec![Option::<FPGAState>::from(&RxMessage {
            data: 0x81,
            ack: 0x00
        })])
    );

    autd.link.down();
    assert_eq!(autd.send(Static::new()).await, Ok(false));
    assert_eq!(autd.fpga_state().await, Err(AUTDError::ReadFPGAStateFailed));
    autd.link.up();
    assert_eq!(autd.send(Static::new()).await, Ok(true));
    autd.link.break_down();
    assert_eq!(
        autd.send(Static::new()).await,
        Err(AUTDError::Internal(AUTDInternalError::LinkError(
            "broken".to_string()
        )))
    );
    assert_eq!(
        autd.fpga_state().await,
        Err(AUTDError::Internal(AUTDInternalError::LinkError(
            "broken".to_string()
        )))
    );
    autd.link.repair();
    assert_eq!(autd.send(Static::new()).await, Ok(true));

    assert_eq!(autd.close().await, Ok(true));
    assert_eq!(
        autd.send(Static::new()).await,
        Err(AUTDError::Internal(AUTDInternalError::LinkClosed))
    );
    assert_eq!(
        autd.fpga_state().await,
        Err(AUTDError::Internal(AUTDInternalError::LinkClosed))
    );
}
