use autd3::prelude::*;

pub async fn plane(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::default()).await?;

    let dir = Vector3::z();

    let m = Sine::new(150. * Hz);
    let g = Plane::new(dir);

    autd.send((m, g)).await?;

    Ok(true)
}
