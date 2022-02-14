use std::collections::HashMap;

fn fibonacci_cached(i: i32, cache: &mut HashMap<i32, i64>) -> i64 {
    if i == 0 || i == 1 {
        i as i64
    } else if let Some(&n) = cache.get(&i) {
        n
    } else {
        let result = fibonacci_cached(i - 1, cache) + fibonacci_cached(i - 2, cache);
        cache.insert(i, result);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fib() {
        let mut cache = HashMap::new();
        assert_eq!(fibonacci_cached(0, &mut cache), 0);
        assert_eq!(fibonacci_cached(1, &mut cache), 1);
        assert_eq!(fibonacci_cached(5, &mut cache), 5);
        assert_eq!(fibonacci_cached(10, &mut cache), 55);
        assert_eq!(fibonacci_cached(30, &mut cache), 832040);
    }

