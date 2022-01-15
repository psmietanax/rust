fn main() {
    let m = merge(&[1, 3, 5], &[2, 4]);
    println!("{:?}", m);
}

struct Merger<'a> {
    arr1: &'a [i32],
    arr2: &'a [i32]
}

struct MergerIter<'a> {
    merger_ref: &'a Merger<'a>,
    counter_vec1: usize,
    counter_vec2: usize
}

impl <'a> Iterator for MergerIter<'a> {
    type Item = &'a i32;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.merger_ref.arr1.get(self.counter_vec1), self.merger_ref.arr2.get(self.counter_vec2)) {
            (None, None) => None,
            (x @ Some(_), None) => {
                self.counter_vec1 = self.counter_vec1 + 1;
                x
            },
            (None, x @ Some(_)) => {
                self.counter_vec2 = self.counter_vec2 + 1;
                x
            },
            (x @ Some(i1), Some(i2)) if i1 < i2 => {
                self.counter_vec1 = self.counter_vec1 + 1;
                x
            },
            (_, x @ Some(_)) => {
                self.counter_vec2 = self.counter_vec2 + 1;
                x
            }
        }
    }
}

impl <'a> IntoIterator for &'a Merger<'a> {
    type Item = &'a i32;
    type IntoIter = MergerIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        MergerIter { merger_ref: &self, counter_vec1: 0, counter_vec2: 0 }
    }
}


// Merge two sorted arrays
fn merge<'a>(arr1: &'a [i32], arr2: &'a [i32]) -> Vec<i32> {
    let merger = Merger { arr1, arr2 };
    merger.into_iter().map(|&x| x).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_empty() {
        assert_eq!(merge(&[], &[1, 2, 3]), &[1, 2, 3]);
        assert_eq!(merge(&[1, 2, 3], &[]), &[1, 2, 3]);
        assert_eq!(merge(&[], &[]), &[]);
    }

    #[test]
    fn test_merge() {
        assert_eq!(merge(&[1, 3, 5], &[2, 4, 6]), &[1, 2, 3, 4, 5, 6]);
        assert_eq!(merge(&[1, 3, 5], &[2, 4]), &[1, 2, 3, 4, 5]);
        assert_eq!(merge(&[1, 3], &[2, 4, 6]), &[1, 2, 3, 4, 6]);
        assert_eq!(merge(&[1, 2, 3], &[4, 5, 6]), &[1, 2, 3, 4, 5, 6]);
        assert_eq!(merge(&[4, 5, 6], &[1, 2, 3]), &[1, 2, 3, 4, 5, 6]);
        assert_eq!(merge(&[1, 2, 7, 8], &[3, 4, 5, 6]), &[1, 2, 3, 4, 5, 6, 7, 8]);
    }

}
