// Use binary search to find first and last occurrence of a given number
// Return Option: Some(index) or None

fn main() {
    println!("{:?}", last(&[1, 2, 3, 4, 5], 5));
}

fn first(arr: &[i32], n: i32) -> Option<usize> {
    let mut start = 0 as usize;
    let mut end = arr.len() - 1;

    while start < end {
        let mid = (start + end) / 2;
        if arr[mid] < n {
            start = mid + 1;
        } else if arr[mid] > n {
            end = mid - 1;
        } else {
            end = mid;
        }
    }

    if arr[start] == n {
        Some(start)
    } else {
        None
    }
}

fn last(arr: &[i32], n: i32) -> Option<usize> {
    let mut start = 0 as usize;
    let mut end = arr.len() - 1;

    while start < end {
        let mid = (start + end + 1) / 2;
        if arr[mid] < n {
            start = mid + 1;
        } else if arr[mid] > n {
            end = mid - 1;
        } else {
            start = mid;
        }
    }

    if arr[end] == n {
        Some(end)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first() {
        assert_eq!(first(&[1, 2, 3, 4, 5], 2), Some(1));
        assert_eq!(first(&[1, 2, 3, 4, 5], 5), Some(4));
        assert_eq!(first(&[1, 2, 2, 2, 3], 2), Some(1));
        assert_eq!(first(&[1, 2, 3, 3, 3], 3), Some(2));
        assert_eq!(first(&[1, 1, 1, 2, 3], 1), Some(0));
        assert_eq!(first(&[1, 3, 4, 5, 6], 2), None);
    }

    #[test]
    fn test_last() {
        assert_eq!(last(&[1, 2, 3, 4, 5], 2), Some(1));
        assert_eq!(last(&[1, 2, 3, 4, 5], 5), Some(4));
        assert_eq!(last(&[1, 2, 2, 2, 3], 2), Some(3));
        assert_eq!(last(&[1, 2, 3, 3, 3], 3), Some(4));
        assert_eq!(last(&[1, 1, 1, 2, 3], 1), Some(2));
        assert_eq!(last(&[1, 3, 4, 5, 6], 2), None);
    }
}
