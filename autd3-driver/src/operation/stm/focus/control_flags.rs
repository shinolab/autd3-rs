use std::fmt;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FocusSTMControlFlags(u8);

bitflags::bitflags! {
    impl FocusSTMControlFlags : u8 {
        const NONE            = 0;
        const BEGIN       = 1 << 0;
        const END         = 1 << 1;
        const UPDATE          = 1 << 2;
    }
}

impl fmt::Display for FocusSTMControlFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = Vec::new();
        if self.contains(FocusSTMControlFlags::BEGIN) {
            flags.push("BEGIN")
        }
        if self.contains(FocusSTMControlFlags::END) {
            flags.push("END")
        }
        if self.contains(FocusSTMControlFlags::UPDATE) {
            flags.push("UPDATE")
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
    fn test_size() {
        assert_eq!(1, std::mem::size_of::<FocusSTMControlFlags>());
    }

    #[test]
    fn test_fmt() {
        assert_eq!("NONE", format!("{}", FocusSTMControlFlags::NONE));
        assert_eq!("BEGIN", format!("{}", FocusSTMControlFlags::BEGIN));
        assert_eq!("END", format!("{}", FocusSTMControlFlags::END));
        assert_eq!("UPDATE", format!("{}", FocusSTMControlFlags::UPDATE));
        assert_eq!(
            "BEGIN | END | UPDATE",
            format!(
                "{}",
                FocusSTMControlFlags::BEGIN
                    | FocusSTMControlFlags::END
                    | FocusSTMControlFlags::UPDATE
            )
        );
    }
}
