use std::fmt;

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct FociSTMControlFlags(u8);

bitflags::bitflags! {
    impl FociSTMControlFlags : u8 {
        const NONE       = 0;
        const BEGIN      = 1 << 0;
        const END        = 1 << 1;
        const TRANSITION = 1 << 2;
    }
}

impl fmt::Display for FociSTMControlFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = Vec::new();
        if self.contains(FociSTMControlFlags::BEGIN) {
            flags.push("BEGIN")
        }
        if self.contains(FociSTMControlFlags::END) {
            flags.push("END")
        }
        if self.contains(FociSTMControlFlags::TRANSITION) {
            flags.push("TRANSITION")
        }
        if self.is_empty() {
            flags.push("NONE")
        }
        write!(
            f,
            "{}",
            flags
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(" | ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_size() {
        assert_eq!(1, std::mem::size_of::<FociSTMControlFlags>());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_fmt() {
        assert_eq!("NONE", format!("{}", FociSTMControlFlags::NONE));
        assert_eq!("BEGIN", format!("{}", FociSTMControlFlags::BEGIN));
        assert_eq!("END", format!("{}", FociSTMControlFlags::END));
        assert_eq!("TRANSITION", format!("{}", FociSTMControlFlags::TRANSITION));
        assert_eq!(
            "BEGIN | END | TRANSITION",
            format!(
                "{}",
                FociSTMControlFlags::BEGIN
                    | FociSTMControlFlags::END
                    | FociSTMControlFlags::TRANSITION
            )
        );
    }
}
