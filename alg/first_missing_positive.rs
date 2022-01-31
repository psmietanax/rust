// https://leetcode.com/problems/first-missing-positive/
fn first_missing_positive(mut nums: Vec<i32>) -> Option<usize> {
    // partition vec into two groups: positive and non-positive numbers
    let mut positive_end = nums.len() - 1;
    for idx in (0..nums.len()).rev() {
        if !nums[idx].is_positive() {
            nums.swap(positive_end, idx);
            positive_end -= 1;
        }
    }
    // run the algorithm for positive nums only (up to positive_end)
    // mark positive positions as negative by number's index
    for idx in 0..=positive_end {
        let num_idx = nums[idx].abs() as usize - 1;
        if num_idx <= positive_end {
            nums[num_idx] = -nums[num_idx].abs();
        }
    }
    // find first positive number and return result
    for idx in 0..=positive_end {
        if nums[idx].is_positive() {
            return Some(idx + 1);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_exist() {
        assert_eq!(first_missing_positive(vec![1, 2, 3]), None);
        assert_eq!(first_missing_positive(vec![0, 1, 2]), None);
        assert_eq!(first_missing_positive(vec![-100, 0, 1, 2]), None);
        assert_eq!(first_missing_positive(vec![2, -100, 0, 1]), None);
        assert_eq!(first_missing_positive(vec![-1, 2, -2, 1, -3]), None);
    }

    #[test]
    fn test_exist() {
        assert_eq!(first_missing_positive(vec![2, 3]), Some(1));
        assert_eq!(first_missing_positive(vec![2, 4, 3, 1, 10]), Some(5));
        assert_eq!(first_missing_positive(vec![100, 101, 102, 1, 103]), Some(2));
    }

    #[test]
    fn test_duplicate() {
        assert_eq!(first_missing_positive(vec![2, 3, 2]), Some(1));
        assert_eq!(first_missing_positive(vec![1, 3, 1]), Some(2));
        assert_eq!(first_missing_positive(vec![1, 2, 3, 3, 2, 1]), Some(4));
    }

}
