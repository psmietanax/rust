// https://leetcode.com/problems/evaluate-reverse-polish-notation/
fn eval_rpn(tokens: Vec<&str>) -> i32 {
    let mut stack: Vec<i32> = Vec::new();
    for token in tokens.into_iter() {
        if is_operator(token) {
            if let (Some(right), Some(left)) = (stack.pop(), stack.pop()) {
                stack.push(eval(left, right, token));
            } else {
                panic!("Bad equation");
            }
        } else {
            stack.push(token.parse::<i32>().unwrap());
        }
    }
    stack.pop().unwrap()
}

fn is_operator(token: &str) -> bool {
    match token {
        "+" | "-" | "/" | "*" => true,
        _ => false
    }
}

fn eval(left: i32, right: i32, operator: &str) -> i32 {
    match operator {
        "+" => {
            left + right
        },
        "-" => {
            left - right
        },
        "/" => {
            left / right
        },
        "*" => {
            left * right
        },
        _ => panic!("Not a valid operator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpn_1() {
        let tokens = vec!["2", "1", "+", "3", "*"];
        assert_eq!(eval_rpn(tokens), 9);
    }

    #[test]
    fn test_rpn_2() {
        let tokens = vec!["4", "13", "5", "/", "+"];
        assert_eq!(eval_rpn(tokens), 6);
    }

    #[test]
    fn test_rpn_3() {
        let tokens = vec!["10", "6", "9", "3", "+", "-11", "*", "/", "*", "17", "+", "5", "+"];
        assert_eq!(eval_rpn(tokens), 22);
    }

    #[test]
    fn test_rpn_4() {
        let tokens = vec!["1", "2", "+", "3", "4", "+", "*"];
        assert_eq!(eval_rpn(tokens), 21);
    }
}
