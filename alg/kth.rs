use std::cmp::Reverse;
use std::collections::BinaryHeap;

fn main() {
    println!("{:?}", kth_largest(&[3, 5, 1, 4, 2], 2));
}


// Find k largest elements in an array
fn kth_largest(arr: &[i32], k: usize) -> Vec<i32> {
    let mut min_heap = BinaryHeap::new();
    arr.into_iter()
        .take(k)
        .for_each(|&i| min_heap.push(Reverse(i)));

    arr.into_iter()
        .skip(k)
        .for_each(|&i| {
            if min_heap.peek().unwrap().0 < i {
                min_heap.pop();
                min_heap.push(Reverse(i));
            }
        });

    min_heap.into_iter().map(|r| r.0).collect()
}

// Find k-th largest element in an array
fn kth(arr: &[i32], k: usize) -> Option<i32> {
    if arr.len() < k {
        None
    } else {
        Some(kth_largest(arr, k)[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kth_largest() {
        assert_eq!(kth_largest(&[1, 2, 3, 4, 5], 2), &[4, 5]);
        assert_eq!(kth_largest(&[5, 4, 3, 2, 1], 2), &[4, 5]);
        assert_eq!(kth_largest(&[3, 1, 5, 4, 2], 2), &[4, 5]);
        assert_eq!(kth_largest(&[2, 2, 1, 1, 3, 3], 3), &[2, 3, 3]);
    }

    #[test]
    fn test_kth() {
        assert_eq!(kth(&[1, 2, 3, 4, 5], 2), Some(4));
        assert_eq!(kth(&[5, 4, 3, 2, 1], 2), Some(4));
        assert_eq!(kth(&[3, 1, 5, 4, 2], 2), Some(4));
        assert_eq!(kth(&[2, 2, 1, 1, 3, 3], 3), Some(2));
        assert_eq!(kth(&[1, 2, 3], 4), None);
    }

}
