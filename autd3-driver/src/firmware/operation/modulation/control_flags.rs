use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct ModulationControlFlags(u8);

bitflags::bitflags! {
    impl ModulationControlFlags : u8 {
        const NONE           = 0;
        const BEGIN          = 1 << 0;
        const END            = 1 << 1;
        const TRANSITION     = 1 << 2;
        const SEGMENT        = 1 << 3;
    }
}

impl fmt::Display for ModulationControlFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = Vec::new();
        if self.contains(ModulationControlFlags::BEGIN) {
            flags.push("BEGIN")
        }
        if self.contains(ModulationControlFlags::END) {
            flags.push("END")
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
        assert_eq!(std::mem::size_of::<ModulationControlFlags>(), 1);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_fmt() {
        assert_eq!(format!("{}", ModulationControlFlags::NONE), "NONE");
        assert_eq!(format!("{}", ModulationControlFlags::BEGIN), "BEGIN");
        assert_eq!(format!("{}", ModulationControlFlags::END), "END");
        assert_eq!(
            format!(
                "{}",
                ModulationControlFlags::BEGIN | ModulationControlFlags::END
            ),
            "BEGIN | END"
        );
    }
}
