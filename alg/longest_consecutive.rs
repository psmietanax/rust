use std::collections::HashSet;

// https://leetcode.com/problems/longest-consecutive-sequence/
pub fn longest_consecutive(nums: Vec<i32>) -> i32 {
    let set: HashSet<i32> = nums.iter().copied().collect();
    let mut max = 0;
    for x in set.iter() {
        // check if the number starts a sequence
        if !set.contains(&(x - 1)) {
            let mut counter = 1;
            while set.contains(&(x + counter)) {
                counter += 1;
            }
            if max < counter {
                max = counter;
            }
        }
    }
    max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_longest_consecutive() {
        assert_eq!(longest_consecutive(vec![]), 0);
        assert_eq!(longest_consecutive(vec![0]), 1);
        assert_eq!(longest_consecutive(vec![1, 9, 3, 10, 4, 20, 2]), 4);
        assert_eq!(longest_consecutive(vec![36, 41, 56, 35, 44, 33, 34, 92, 43, 32, 42]), 5);
        assert_eq!(longest_consecutive(vec![100, 4, 200, 1, 3, 2]), 4);
        assert_eq!(longest_consecutive(vec![0, 3, 7, 2, 5, 8, 4, 6, 0, 1]), 9);
        assert_eq!(longest_consecutive(vec![1, 2, 3, 4, 5]), 5);
        assert_eq!(longest_consecutive(vec![5, 4, 3, 2, 1]), 5);
    }

}
