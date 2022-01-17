fn main() {
    println!("{}", sum("999".to_string(), "99999".to_string()));
}

fn sum(num1: String, num2: String) -> String {
    let mut result = Vec::new();
    let mut it1 = num1.chars().rev();
    let mut it2 = num2.chars().rev();
    let mut rem = 0;

    loop {
        match (it1.next(), it2.next()) {
            (Some(x), Some(y)) => {
                let val = x.to_digit(10).unwrap() + y.to_digit(10).unwrap() + rem;
                rem = val / 10;
                result.push(char::from_digit(val % 10, 10).unwrap());
            },
            (Some(x), None) | (None, Some(x)) => {
                let val = x.to_digit(10).unwrap() + rem;
                rem = val / 10;
                result.push(char::from_digit(val % 10, 10).unwrap());
            },
            (None, None) => {
                break;
            }
        }
    }
    while rem > 0 {
        result.push(char::from_digit(rem % 10, 10).unwrap());
        rem = rem / 10;
    }

    result.iter().rev().collect()
}

fn mul(num1: String, num2: String) -> String {
    let mut results = Vec::new();
    let (mut it, num) = if num1.len() < num2.len() {
        (num1.chars().rev().enumerate(), &num2)
    } else {
        (num2.chars().rev().enumerate(), &num1)
    };

    while let Some((idx, x)) = it.next() {
        results.push(_mul(num, x, idx));
    }

    results.into_iter().reduce(|x, y| sum(x, y)).unwrap()
}

fn _mul(num: &String, y: char, appender: usize) -> String {
    let mut result = Vec::new();
    let mut it = num.chars().rev();
    let mut rem = 0;

    for _ in 0..appender {
        result.push('0');
    }

    let mut all_zeroes = true;

    while let Some(x) = it.next() {
        let val = x.to_digit(10).unwrap() * y.to_digit(10).unwrap() + rem;
        rem = val / 10;
        result.push(char::from_digit(val % 10, 10).unwrap());

        if all_zeroes && val != 0 {
            all_zeroes = false;
        }
    }
    while rem > 0 {
        result.push(char::from_digit(rem % 10, 10).unwrap());
        rem = rem / 10;
    }

    if all_zeroes {
        "0".to_string()
    } else {
        result.iter().rev().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum() {
        assert_eq!(sum("1234".to_string(), "1234".to_string()), "2468".to_string());
        assert_eq!(sum("123456789".to_string(), "1234".to_string()), "123458023".to_string());
        assert_eq!(sum("0".to_string(), "0".to_string()), "0".to_string());
        assert_eq!(sum("9999999999".to_string(), "1".to_string()), "10000000000".to_string());
    }

    #[test]
    fn test_mul() {
        assert_eq!(mul("1234".to_string(), "1234".to_string()), "1522756".to_string());
        assert_eq!(mul("123456789".to_string(), "1234".to_string()), "152345677626".to_string());
        assert_eq!(mul("9999".to_string(), "0".to_string()), "0".to_string());
        assert_eq!(mul("9999".to_string(), "1".to_string()), "9999".to_string());
    }

}
