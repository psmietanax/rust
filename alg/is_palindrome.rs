fn main() {
    println!("{:?}", "abccba".to_string().is_palindrome());
}

trait PalindromeTest {
    fn is_palindrome(&self) -> bool;
}

impl PalindromeTest for String {
    fn is_palindrome(&self) -> bool {
        if self.is_empty() {
            return false;
        }
        let forward_it = self.chars().into_iter().take(self.len() / 2);
        let reverse_it = self.chars().rev().into_iter().take(self.len() / 2);
        forward_it.zip(reverse_it).all(|(x, y)| x == y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_palindrome() {
        assert_eq!("level".to_string().is_palindrome(), true);
        assert_eq!("helloolleh".to_string().is_palindrome(), true);
        assert_eq!("level refer level".to_string().is_palindrome(), true);
    }

    #[test]
    fn test_is_not_palindrome() {
        assert_eq!("hello".to_string().is_palindrome(), false);
        assert_eq!("".to_string().is_palindrome(), false);
        assert_eq!("not a palindrome".to_string().is_palindrome(), false);
    }
}
