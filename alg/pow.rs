// An iterative function for pow(x, y)
// Complexity: O(log n)

fn main() {
    println!("{}", pow(5, 13));
}

fn pow(mut n: i32, mut k: u32) -> i64 {
    let mut result: i64 = 1;

    while k > 0 {
        if k.is_odd() {
            k = k - 1;
            result = result * n as i64;
        } else {
            k = k / 2;
            n = n * n;
        }
    }

    result
}

trait IsOdd {
    fn is_odd(&self) -> bool;
}

// enrich u32 by adding a check whether a number is odd 
impl IsOdd for u32 {
    fn is_odd(&self) -> bool {
        *self % 2 != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_numbers() {
        assert_eq!(pow(5, 1), 5);
        assert_eq!(pow(5, 2), 5_i64.pow(2));
        assert_eq!(pow(5, 7), 5_i64.pow(7));
        assert_eq!(pow(-5, 7), -5_i64.pow(7));
    }

    #[test]
    fn test_big_numbers() {
        assert_eq!(pow(111, 1), 111_i64.pow(1));
        assert_eq!(pow(111, 2), 111_i64.pow(2));
        assert_eq!(pow(111, 7), 111_i64.pow(7));
        assert_eq!(pow(-111, 7), -111_i64.pow(7));
    }
}
