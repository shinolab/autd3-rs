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
        let modulation = autd[0]
            .modulation()
            .take_while(|(t, _)| *t <= Duration::from_millis(5));
        println!("200Hz sine raw modulation buffer");
        Chart::new(180, 40, 0.0, 5.0)
            .lineplot(&Shape::Lines(
                &modulation
                    .map(|(t, v)| (t.as_micros() as f32 / 1000., v as f32))
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

        let pulse_width = record[0][0]
            .pulse_width()
            .into_iter()
            .skip_while(|(t, _)| *t < Duration::from_millis(5));
        println!("pulse width under 200Hz sine modulation with silencer");
        Chart::new(180, 40, 5.0, 10.0)
            .lineplot(&Shape::Lines(
                &pulse_width
                    .into_iter()
                    .map(|(t, v)| (t.as_micros() as f32 / 1000., v as f32))
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

        let pulse_width = record[0][0]
            .pulse_width()
            .into_iter()
            .skip_while(|(t, _)| *t < Duration::from_millis(5));
        println!("pulse width under 200Hz sine modulation without silencer");
        Chart::new(180, 40, 5.0, 10.0)
            .lineplot(&Shape::Lines(
                &pulse_width
                    .into_iter()
                    .map(|(t, v)| (t.as_micros() as f32 / 1000., v as f32))
                    .collect::<Vec<_>>(),
            ))
            .display();
    };

    autd.close().await?;

    Ok(())
}
