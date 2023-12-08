/*
 * File: helper.rs
 * Project: benches
 * Created Date: 30/07/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 26/11/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use autd3::prelude::*;
use autd3_driver::geometry::IntoDevice;

pub fn generate_geometry(size: usize) -> Geometry {
    Geometry::new(
        (0..size)
            .flat_map(|i| {
                (0..size).map(move |j| {
                    AUTD3::new(Vector3::new(
                        i as float * AUTD3::DEVICE_WIDTH,
                        j as float * AUTD3::DEVICE_HEIGHT,
                        0.,
                    ))
                    .into_device(j + i * size)
                })
            })
            .collect(),
    )
}
