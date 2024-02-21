mod amp;
mod combinatorial;
mod constraint;
mod linear_synthesis;
mod matrix;
mod nls;

#[macro_export]
macro_rules! to_holo {
    ($self:expr) => {
        $self
            .foci()
            .map(|(p, a)| Holo {
                pos: Some(p.to_msg(None)),
                amp: Some(a.to_msg(None)),
            })
            .collect()
    };
}
