#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Segment {
    #[default]
    S0 = 0,
    S1 = 1,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment() {
        let d = Segment::S0;

        let dc = Clone::clone(&d);
        assert_eq!(d, dc);

        assert_eq!(format!("{:?}", d), "S0");
    }
}
