/*
 * File: focus.rs
 * Project: tests
 * Created Date: 28/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 26/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Shun Suzuki. All rights reserved.
 *
 */

use autd3::prelude::*;

pub async fn focus<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<bool> {
    // autd.send(ConfigureSilencer::with_fixed_completion_steps(10, 40)?)
    //     .await?;
    autd.send(ConfigureSilencer::disable()).await?;

    let center = autd.geometry.center() + Vector3::new(0., 0., 150.0 * MILLIMETER);

    let g = Focus::new(center);
    let m = Sine::new(150.);

    autd.send((m, g)).await?;

    Ok(true)
}
