use std::cmp;

fn main() {
    println!("{}", max_sub_array(vec![1, -3, 2, 3, -10, 1, 1, 2]));
}

fn max_sub_array(nums: Vec<i32>) -> i32 {
    let mut max = 0;
    let mut max_so_far = 0;
    for i in 0..nums.len() {
        max_so_far = cmp::max(max_so_far + nums[i], 0);
        max = cmp::max(max, max_so_far);
    }
    max
}

fn max_sub_array_indexed(nums: Vec<i32>) -> (i32, i32, i32) {
    let mut max = 0;
    let mut max_so_far = 0;
    let mut start = 0;
    let mut end = -1;
    let mut max_start = -1;
    let mut max_end = -1;
    for i in 0..nums.len() {
        if max_so_far + nums[i] > 0 {
            end = i as i32;
            max_so_far += nums[i];
        } else {
            start = (i + 1) as i32;
            max_so_far = 0;
        }
        if max_so_far > max {
            max = max_so_far;
            max_start = start;
            max_end = end;
        }
    }
    (max, max_start, max_end)
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
        assert_eq!(max_sub_array(vec![5, 4, -1, 7, 8]), 23);
    }

    #[test]
    fn test_max_sub_array_indexed() {
        assert_eq!(max_sub_array_indexed(vec![1, -3, 2, 3, -10, 1, 1, 2]), (5, 2, 3));
        assert_eq!(max_sub_array_indexed(vec![-2, 1, -3, 4, -1, 2, 1, -5, 4]), (6, 3, 6));
        assert_eq!(max_sub_array_indexed(vec![1]), (1, 0, 0));
        assert_eq!(max_sub_array_indexed(vec![-3]), (0, -1, -1));
        assert_eq!(max_sub_array_indexed(vec![5, 4, -1, 7, 8]), (23, 0, 4));
    }

}
