use std::cmp;

fn main() {
    println!("{}", max_sub_array(vec![1, -3, 2, 3, -10, 1, 1, 2]));
}

// https://leetcode.com/problems/maximum-subarray/
fn max_sub_array(nums: Vec<i32>) -> i32 {
    let mut max = 0;
    let mut max_so_far = 0;
    for i in 0..nums.len() {
        max_so_far = cmp::max(max_so_far + nums[i], 0);
        max = cmp::max(max, max_so_far);
    }
    max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_sub_array() {
        assert_eq!(max_sub_array(vec![1, -3, 2, 3, -10, 1, 1, 2]), 5);
        assert_eq!(max_sub_array(vec![-2, 1, -3, 4, -1, 2, 1, -5, 4]), 6);
        assert_eq!(max_sub_array(vec![1]), 1);
        assert_eq!(max_sub_array(vec![-3]), 0);
        assert_eq!(max_sub_array(vec![5,4,-1,7,8]), 23);
    }

}
