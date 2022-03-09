// https://leetcode.com/problems/text-justification/
pub fn full_justify(words: Vec<String>, max_width: usize) -> Vec<String> {
    fn _build_line(words: &[String], len: usize, max_width: usize) -> String {
        let mut line = String::with_capacity(max_width);
        if words.len() > 1 {
            let separators = words.len() - 1;
            let filling = max_width - len;
            let separator_len = filling / separators;
            let mut separator_len_carrier = filling % separators;
            for idx in 0..words.len() - 1 {
                line.push_str(&words[idx]);
                for _ in 0..separator_len {
                    line.push(' ');
                }
                if separator_len_carrier > 0 {
                    line.push(' ');
                    separator_len_carrier -= 1;
                }
            }
            line.push_str(&words[words.len() - 1]);
        } else {
            line.push_str(&words[0]);
            for _ in words[0].len()..max_width {
                line.push(' ');
            }
        }
        line
    }

    let mut result = Vec::new();
    let mut len = words[0].len();
    let mut start = 0;
    for idx in 1..words.len() {
        if len + words[idx].len() + (idx - start) <= max_width {
            len += words[idx].len();
            continue;
        }
        result.push(_build_line(&words[start..idx], len, max_width));
        start = idx;
        len = words[start].len();
    }
    if start < words.len() {
        let mut len = 0;
        let mut last_line = String::new();
        for idx in start..words.len() - 1 {
            last_line.push_str(&words[idx]);
            last_line.push(' ');
            len += words[idx].len() + 1;
        }
        last_line.push_str(&words[words.len() - 1]);
        len += words[words.len() - 1].len();
        for _ in len..max_width {
            last_line.push(' ');
        }
        result.push(last_line);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        assert_eq!(full_justify(vec!["This".to_string(),
                                     "is".to_string(),
                                     "an".to_string(),
                                     "example".to_string(),
                                     "of".to_string(),
                                     "text".to_string(),
                                     "justification.".to_string()], 16),
        vec!["This    is    an",
             "example  of text",
             "justification.  "]);
    }

    #[test]
    fn test2() {
        assert_eq!(full_justify(vec!["What".to_string(),
                                     "must".to_string(),
                                     "be".to_string(),
                                     "acknowledgment".to_string(),
                                     "shall".to_string(),
                                     "be".to_string()], 16),
                   vec!["What   must   be",
                        "acknowledgment  ",
                        "shall be        "]);
    }

    #[test]
    fn test3() {
        assert_eq!(full_justify(vec!["Science".to_string(),
                                     "is".to_string(),
                                     "what".to_string(),
                                     "we".to_string(),
                                     "understand".to_string(),
                                     "well".to_string(),
                                     "enough".to_string(),
                                     "to".to_string(),
                                     "explain".to_string(),
                                     "to".to_string(),
                                     "a".to_string(),
                                     "computer.".to_string(),
                                     "Art".to_string(),
                                     "is".to_string(),
                                     "everything".to_string(),
                                     "else".to_string(),
                                     "we".to_string(),
                                     "do".to_string()], 20),
                   vec!["Science  is  what we",
                        "understand      well",
                        "enough to explain to",
                        "a  computer.  Art is",
                        "everything  else  we",
                        "do                  "]);
    }
}
