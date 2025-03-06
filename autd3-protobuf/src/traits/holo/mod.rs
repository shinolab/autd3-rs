mod amp;
mod combinatorial;
mod constraint;
mod linear_synthesis;
mod nls;

#[macro_export]
macro_rules! to_holo {
    ($self:expr) => {
        $self
            .foci
            .into_iter()
            .map(|(p, a)| Holo {
                pos: Some(p.into()),
                amp: Some(a.into()),
            })
            .collect()
    };
}
