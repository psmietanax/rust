use std::collections::HashSet;

fn main() {
    println!("{:?}", three_sum(&[1, 2, 3, 4, 5, 6, 7], 10));
}

// find all pairs of integers whose sum is equal to a given number
fn two_sum(arr: &[i32], sum: i32) -> Vec<(i32, i32)> {
    let mut set = HashSet::new();
    let mut result = Vec::new();

    for i in arr.iter() {
        if set.contains(&(sum - i)) {
            result.push((*i, sum - i))
        }
        set.insert(*i);
    }

    result
}

// find all triplets of integers whose sum is equal to a given number
fn three_sum(arr: &[i32], sum: i32) -> Vec<(i32, i32, i32)> {
    let mut result = Vec::new();

    for (idx, i) in arr.iter().enumerate() {
        for (x, y) in two_sum(&arr[idx+1..], sum - i).into_iter() {
            result.push((*i, x, y));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_sum_empty() {
        assert_eq!(two_sum(&[], 10), vec![]);
        assert_eq!(two_sum(&[1, 2, 3, 4], 10), vec![]);
        assert_eq!(two_sum(&[1, 2, 3, 4], 1), vec![]);
    }

    #[test]
    fn test_two_sum() {
        assert_eq!(two_sum(&[1, 2, 3, 4, 5, 6, 7], 10), vec![(6, 4), (7, 3)]);
        assert_eq!(two_sum(&[4, 1, 7, 3, 6, 5, 2], 10), vec![(3, 7), (6, 4)]);
        assert_eq!(two_sum(&[1, 2, 3, 3, 4], 6), vec![(3, 3), (4, 2)]);
    }

    #[test]
    fn test_two_sum_negative() {
        assert_eq!(two_sum(&[-10, 10, 0, 5, -5], 0), vec![(10, -10), (-5, 5)]);
    }

    #[test]
    fn test_three_sum_empty() {
        assert_eq!(three_sum(&[], 10), vec![]);
        assert_eq!(three_sum(&[1, 2, 3, 4], 10), vec![]);
        assert_eq!(three_sum(&[1, 2, 3, 4], 1), vec![]);
    }

    #[test]
    fn test_three_sum() {
        assert_eq!(three_sum(&[1, 2, 3, 4, 5, 6, 7], 10), vec![(1, 5, 4), (1, 6, 3), (1, 7, 2), (2, 5, 3)]);
        assert_eq!(three_sum(&[4, 1, 7, 3, 6, 5, 2], 10), vec![(4, 5, 1), (1, 6, 3), (1, 2, 7), (3, 2, 5)]);
        assert_eq!(three_sum(&[1, 2, 3, 3, 3, 4], 9), vec![(2, 4, 3), (3, 3, 3)]);
    }

    #[test]
    fn test_three_sum_negative() {
        assert_eq!(three_sum(&[-10, 10, 0, 5, -5], 0), vec![(-10, 0, 10), (0, -5, 5)]);
        assert_eq!(three_sum(&[1, -4, 5, -6, 12], 2), vec![(1, 5, -4), (-4, 12, -6)]);
    }

}
