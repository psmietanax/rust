use std::cmp;

fn main() {
    println!("{}", binary_gap(0b0));
}

// https://leetcode.com/problems/binary-gap/
fn binary_gap(mut n: usize) -> usize {
    while n > 0 && n % 2 == 0 {
        n /= 2;
    }

    let mut local_max = 0;
    let mut max = 0;

    while n > 0 {
        if n % 2 == 1 {
            max = cmp::max(max, local_max);
            local_max = 0;
        } else {
            local_max += 1;
        }
        n /= 2;
    }
    max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_gap() {
        assert_eq!(binary_gap(0b0), 0);
        assert_eq!(binary_gap(0b0), 0);
        assert_eq!(binary_gap(0b1001), 2);
        assert_eq!(binary_gap(0b10010000), 2);
        assert_eq!(binary_gap(0b100100001), 4);
        assert_eq!(binary_gap(0b100100001001000010001), 4);
    }

}
