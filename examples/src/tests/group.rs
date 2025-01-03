use autd3::{driver::link::Link, prelude::*};

pub async fn group_by_device(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    let center = autd.center() + Vector3::new(0., 0., 150.0 * mm);

    autd.group(|dev| match dev.idx() {
        0 => Some("null"),
        1 => Some("focus"),
        _ => None,
    })
    .set("null", (Static::new(), Null::new()))?
    .set("focus", (Sine::new(150. * Hz), Focus::new(center)))?
    .send()
    .await?;

    Ok(true)
}

pub async fn group_by_transducer(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    let cx = autd.center().x;
    let g1 = Focus::new(autd[0].center() + Vector3::new(0., 0., 150.0 * mm));
    let g2 = Null::new();
    let g = Group::new(move |_dev| {
        move |tr| {
            if tr.position().x < cx {
                Some("focus")
            } else {
                Some("null")
            }
        }
    })
    .set("focus", g1)?
    .set("null", g2)?;

    let m = Sine::new(150. * Hz);
    autd.send((m, g)).await?;

    Ok(true)
}
