use std::cmp::max;

fn main() {
    println!("{:?}", longest_palindrome("aa".to_string()));
}

fn longest_palindrome(s: String) -> String {
    if s.is_empty() {
        return s;
    }
    let chars: Vec<char> = s.chars().collect();
    let mut max_count = 1;
    let mut max_idx = 0;
    for idx in 1..chars.len() {
        let odd_palindrome_count = palindrome_count(&chars, idx - 1, idx + 1);
        let even_palindrome_count = palindrome_count(&chars, idx - 1, idx);
        let local_max = max(odd_palindrome_count, even_palindrome_count);
        if local_max > max_count {
            max_count = local_max;
            max_idx = idx;
        }
    }

    let start = max_idx - max_count / 2;
    let end = if max_count % 2 == 0 {
        max_idx + max_count / 2
    } else {
        max_idx + max_count / 2 + 1
    };

    chars[start..end].iter().collect()
}

fn palindrome_count(chars: &Vec<char>, mut left: usize, mut right: usize) -> usize {
    let mut count = 0;
    while right < chars.len() && chars[left] == chars[right] {
        count = right - left + 1;
        if left == 0 || right == chars.len() - 1 {
            break;
        }
        left = left - 1;
        right = right + 1;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_longest_palindrome() {
        assert_eq!(longest_palindrome("".to_string()), "");
        assert_eq!(longest_palindrome("a".to_string()), "a");
        assert_eq!(longest_palindrome("ab".to_string()), "a");
        assert_eq!(longest_palindrome("aa".to_string()), "aa");
        assert_eq!(longest_palindrome("aba".to_string()), "aba");
        assert_eq!(longest_palindrome("xa".to_string()), "x");
        assert_eq!(longest_palindrome("xab".to_string()), "x");
        assert_eq!(longest_palindrome("xaa".to_string()), "aa");
        assert_eq!(longest_palindrome("xaba".to_string()), "aba");
        assert_eq!(longest_palindrome("ax".to_string()), "a");
        assert_eq!(longest_palindrome("abx".to_string()), "a");
        assert_eq!(longest_palindrome("aax".to_string()), "aa");
        assert_eq!(longest_palindrome("abax".to_string()), "aba");
        assert_eq!(longest_palindrome("best level".to_string()), "level");
    }

}
