use std::cmp;

fn main() {
    println!("{}", max_profit(vec![7, 1, 5, 3, 6, 4]));
}

// https://leetcode.com/problems/best-time-to-buy-and-sell-stock/
fn max_profit(nums: Vec<i32>) -> i32 {
    let mut max = 0;
    let mut max_element_so_far = 0;
    for i in (0..nums.len()).rev() {
        max_element_so_far = cmp::max(max_element_so_far, nums[i]);
        max = cmp::max(max, max_element_so_far - nums[i]);
    }
    max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_profit() {
        assert_eq!(max_profit(vec![7, 1, 5, 3, 6, 4]), 5);
        assert_eq!(max_profit(vec![6, 5, 4, 3, 2, 1]), 0);
        assert_eq!(max_profit(vec![1, 6, 6, 6, 6, 7]), 6);

    }

}
