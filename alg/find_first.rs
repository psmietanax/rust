use std::ops::AddAssign;
use linked_hash_map::LinkedHashMap;

fn main() {
    println!("{:?}", first_unique(&"aabbc".to_string()));
}

// Find first unique character in a text
fn first_unique(text: &str) -> Option<char> {
    let mut map = LinkedHashMap::<char, usize>::new();

    text.chars()
        .into_iter()
        .for_each(|c| { 
            map.entry(c)
                .or_insert(0)
                .add_assign(1);
    });

    map.iter()
        .filter(|&(_, &counter)| counter == 1)
        .map(|(&c, _)| c)
        .next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique() {
        assert_eq!(first_unique(&"abcd".to_string()), Some('a'));
        assert_eq!(first_unique(&"aabcd".to_string()), Some('b'));
        assert_eq!(first_unique(&"aabbcdc".to_string()), Some('d'));
        assert_eq!(first_unique(&"abcdabcde".to_string()), Some('e'));
    }

    #[test]
    fn test_no_unique() {
        assert_eq!(first_unique(&"".to_string()), None);
        assert_eq!(first_unique(&"aabbcc".to_string()), None);
    }
}
