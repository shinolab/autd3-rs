use std::time::Duration;

use anyhow::Result;

use autd3::prelude::*;
use autd3_link_calc::Calc;

use textplots::{Chart, Plot, Shape};

#[tokio::main]
async fn main() -> Result<()> {
    let mut autd =
        Controller::builder([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())])
            .open(Calc::builder())
            .await?;

    // raw modulation buffer
    {
        autd.send(Sine::new(200. * Hz)).await?;

        let df = autd[0].modulation();
        let t = df["time[s]"].f32()?;
        let modulation = df["modulation"].u8()?;
        println!("200Hz sine raw modulation buffer");
        Chart::new(180, 40, 0.0, 5.0)
            .lineplot(&Shape::Lines(
                &t.into_no_null_iter()
                    .zip(modulation.into_no_null_iter())
                    .map(|(t, v)| (t * 1000., v as f32))
                    .collect::<Vec<_>>(),
            ))
            .display();
    }

    // pulse width under 200Hz sine modulation with silencer
    {
        autd.send(Silencer::default()).await?;
        autd.start_recording()?;
        autd.send((Sine::new(200. * Hz), Uniform::new(EmitIntensity::new(0xFF))))
            .await?;
        autd.tick(Duration::from_millis(10))?;
        let record = autd.finish_recording()?;

        let df = record[0][0].pulse_width();
        let t = df["time[s]"].f32()?;
        let pulse_width = df["pulsewidth"].u8()?;
        println!("pulse width under 200Hz sine modulation with silencer");
        Chart::new(180, 40, 5.0, 10.0)
            .lineplot(&Shape::Lines(
                &t.into_no_null_iter()
                    .zip(pulse_width.into_no_null_iter())
                    .map(|(t, v)| (t * 1000., v as f32))
                    .collect::<Vec<_>>(),
            ))
            .display();
    };

    // pulse width under 200Hz sine modulation without silencer
    {
        autd.send(Silencer::disable()).await?;
        autd.start_recording()?;
        autd.send((Sine::new(200. * Hz), Uniform::new(EmitIntensity::new(0xFF))))
            .await?;
        autd.tick(Duration::from_millis(10))?;
        let record = autd.finish_recording()?;

        let df = record[0][0].pulse_width();
        let t = df["time[s]"].f32()?;
        let pulse_width = df["pulsewidth"].u8()?;
        println!("pulse width under 200Hz sine modulation without silencer");
        Chart::new(180, 40, 5.0, 10.0)
            .lineplot(&Shape::Lines(
                &t.into_no_null_iter()
                    .zip(pulse_width.into_no_null_iter())
                    .map(|(t, v)| (t * 1000., v as f32))
                    .collect::<Vec<_>>(),
            ))
            .display();
    };

    // output voltage
    {
        autd.send(Silencer::disable()).await?;
        autd.start_recording()?;
        autd.send((
            Static::with_intensity(0xFF),
            Uniform::new((Phase::new(0x40), EmitIntensity::new(0xFF))),
        ))
        .await?;
        autd.tick(Duration::from_millis(1))?;
        let record = autd.finish_recording()?;

        let df = record[0][0].output_voltage();
        let t = df["time[s]"].f32()?;
        let v = df["voltage[V]"].f32()?;
        println!("output voltage");
        Chart::new(360, 40, 0.0, 1.0)
            .lineplot(&Shape::Lines(
                &t.into_no_null_iter()
                    .zip(v.into_no_null_iter())
                    .map(|(t, v)| (t * 1000., v))
                    .collect::<Vec<_>>(),
            ))
            .display();
    };

    // output ultrasound
    {
        autd.send(Silencer::disable()).await?;
        autd.start_recording()?;
        autd.send((
            Static::with_intensity(0xFF),
            Uniform::new((Phase::new(0x40), EmitIntensity::new(0xFF))),
        ))
        .await?;
        autd.tick(Duration::from_millis(1))?;
        let record = autd.finish_recording()?;

        let df = record[0][0].output_ultrasound();
        let t = df["time[s]"].f32()?;
        let v = df["p[a.u.]"].f32()?;
        println!("output ultrasound");
        Chart::new(360, 40, 0.0, 1.0)
            .lineplot(&Shape::Lines(
                &t.into_no_null_iter()
                    .zip(v.into_no_null_iter())
                    .map(|(t, v)| (t * 1000., v))
                    .collect::<Vec<_>>(),
            ))
            .display();
    };

    autd.close().await?;

    Ok(())
}
