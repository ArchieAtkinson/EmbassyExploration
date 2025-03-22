#![cfg_attr(not(test), no_std)]

pub fn add_two(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(2, add_two(1, 1));
    }
}
