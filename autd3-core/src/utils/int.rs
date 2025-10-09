fn is_zero<T>(x: T) -> bool
where
    T: From<u8> + PartialEq,
{
    x == T::from(0u8)
}

pub fn gcd<T>(mut a: T, mut b: T) -> T
where
    T: Copy + From<u8> + PartialEq + core::ops::Rem<Output = T>,
{
    while !is_zero(b) {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

pub fn lcm<T>(a: T, b: T) -> T
where
    T: Copy
        + From<u8>
        + PartialEq
        + core::ops::Rem<Output = T>
        + core::ops::Mul<Output = T>
        + core::ops::Div<Output = T>,
{
    let gcd = gcd(a, b);
    if is_zero(gcd) {
        return T::from(0u8);
    }
    a * (b / gcd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(12, 18), 6);
        assert_eq!(gcd(18, 12), 6);
        assert_eq!(gcd(7, 13), 1);
        assert_eq!(gcd(0, 5), 5);
        assert_eq!(gcd(5, 0), 5);
        assert_eq!(gcd(0, 0), 0);
    }

    #[test]
    fn test_lcm() {
        assert_eq!(lcm(12, 18), 36);
        assert_eq!(lcm(18, 12), 36);
        assert_eq!(lcm(7, 13), 91);
        assert_eq!(lcm(0, 5), 0);
        assert_eq!(lcm(5, 0), 0);
        assert_eq!(lcm(0, 0), 0);
    }
}
