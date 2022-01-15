fn main() {
    println!("{:?}", reverse(1230));
}

// reverse integer
fn reverse(mut x: i32) -> i32 {
    let mut result = 0;
    while x > 0 {
        result = result * 10 + x % 10;
        x = x / 10;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse() {
        assert_eq!(reverse(1), 1);
        assert_eq!(reverse(12), 21);
        assert_eq!(reverse(123), 321);
        assert_eq!(reverse(100), 1);
        assert_eq!(reverse(120), 21);
        assert_eq!(reverse(0), 0);
    }

}
