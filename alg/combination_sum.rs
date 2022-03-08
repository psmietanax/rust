// https://leetcode.com/problems/combination-sum/
pub fn combination_sum(candidates: Vec<i32>, target: i32) -> Vec<Vec<i32>> {
    fn _combination_sum(sub: &[i32], candidates: &[i32], target: i32, res: &mut Vec<Vec<i32>>) {
        if target == 0 {
            res.push(sub.to_vec());
        } else if target > 0 {
            for (idx, num) in candidates.iter().enumerate() {
                let mut sub_vec = sub.to_vec();
                sub_vec.push(*num);
                _combination_sum(&sub_vec, &candidates[idx..], target - *num, res);
            }
        }
    }

    let mut res: Vec<Vec<i32>> = vec![];
    _combination_sum(&vec![], &candidates, target, &mut res);
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bt() {
        assert_eq!(combination_sum(vec![2, 3, 6, 7], 7), vec![vec![2, 2, 3], vec![7]]);
    }

    #[test]
    fn test_bt2() {
        assert_eq!(combination_sum(vec![2, 3, 5], 8), vec![vec![2, 2, 2, 2], vec![2, 3, 3], vec![3, 5]]);
    }
}
