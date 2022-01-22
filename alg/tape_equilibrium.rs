use std::cmp::{max, min};

// https://app.codility.com/programmers/lessons/3-time_complexity/tape_equilibrium/
// Tape Equilibrium problem - min difference between two split parts
// Array: [1 3 2 8 1]
// Parts: [[1], [3 2 8 1]], [[1 3], [2 8 1]], [[1 3 2], [8 1]], [[1 3 2 8], [1]]
fn tape_equilibrium(arr: &[i32]) -> i32 {
    let mut left = arr[0];
    let mut right = arr.iter().skip(1).fold(0, |x, y| x + y);
    let mut diff = (right - left).abs();
    for v in arr.iter().skip(1) {
        left += v;
        right -= v;
        diff = min(diff, (right - left).abs());
    }
    diff
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use super::*;

    #[test]
    fn test() {
        assert_eq!(tape_equilibrium(&[3, 1, 2, 4, 3]), 1);
        assert_eq!(tape_equilibrium(&[1, 3, 2, 8, 1]), 3);
    }

}
