use autd3::prelude::*;

pub async fn group_by_device(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    let center = autd.geometry.center() + Vector3::new(0., 0., 150.0 * MILLIMETER);

    autd.group(|dev| match dev.idx() {
        0 => Some("null"),
        1 => Some("focus"),
        _ => None,
    })
    .set("null", (Static::new(), Null::new()))?
    .set("focus", (Sine::new(150.), Focus::new(center)))?
    .send()
    .await?;

    Ok(true)
}

pub async fn group_by_transducer(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    let cx = autd.geometry.center().x;
    let g1 = Focus::new(autd.geometry[0].center() + Vector3::new(0., 0., 150.0 * MILLIMETER));
    let g2 = Null::new();
    let g = Group::new(move |_dev, tr: &Transducer| {
        if tr.position().x < cx {
            Some("focus")
        } else {
            Some("null")
        }
    })
    .set("focus", g1)
    .set("null", g2);

    let m = Sine::new(150.);
    autd.send((m, g)).await?;

    Ok(true)
}
