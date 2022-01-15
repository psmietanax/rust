fn main() {
    println!("{:?}", reverse_words(&"hello   world"));
}

// reverse words in a given sentence
fn reverse_words(text: &str) -> String {
    text.split_whitespace()
        .rev()
        .collect::<Vec<&str>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse_words() {
        assert_eq!(reverse_words(&"Hello world"), "world Hello");
        assert_eq!(reverse_words(&"one two three four"), "four three two one");
        assert_eq!(reverse_words(&"Test"), "Test");
        assert_eq!(reverse_words(&"spaces      more   spaces"), "spaces more spaces");
    }
}
