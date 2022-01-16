use std::cmp::{max, min};

fn main() {
    println!("{:?}", max_area(&[1,8,6,2,5,4,8,3,7]));
}

// Container With Most Water
// https://leetcode.com/problems/container-with-most-water/
fn max_area(arr: &[usize]) -> usize {
    let mut max_area = 0;
    let mut left = 0;
    let mut right = arr.len() - 1;

    while left < right {
        max_area = max(max_area, min(arr[left], arr[right]) * (right - left));
        if arr[left] < arr[right] {
            left += 1;
        } else if arr[left] > arr[right] {
            right -= 1;
        } else {
            left += 1;
            right -= 1;
        }
    }

    max_area
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_area() {
        assert_eq!(max_area(&[1, 8, 6, 2, 5, 4, 8, 3, 7]), 49);
        assert_eq!(max_area(&[3, 9, 3, 4, 7, 2, 12, 6]), 45);
        assert_eq!(max_area(&[1, 1]), 1);
        assert_eq!(max_area(&[6, 1, 6, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]), 13);
        assert_eq!(max_area(&[1, 2, 3, 4, 3, 2, 1]), 8);
    }

}
