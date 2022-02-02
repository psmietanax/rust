#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ListNode {
  pub val: i32,
  pub next: Option<Box<ListNode>>
}

impl ListNode {
  #[inline]
  fn new(val: i32) -> Self {
    ListNode {
      next: None,
      val
    }
  }
}

// https://leetcode.com/problems/rotate-list/
fn rotate_right(mut head: Option<Box<ListNode>>, n: usize) -> Option<Box<ListNode>> {
    let (head, tail) = split(head, n);
    concat(tail, head)
}

fn rotate_left(mut head: Option<Box<ListNode>>, n: usize) -> Option<Box<ListNode>> {
    let list_len = len(&head);
    let n = list_len - (n % list_len);
    rotate_right(head, n)
}

fn len(head: &Option<Box<ListNode>>) -> usize {
    match head {
        None => 0,
        Some(node) => {
            1 + len(&node.next)
        }
    }
}

fn last(head: &mut Option<Box<ListNode>>) -> Option<&mut Box<ListNode>> {
    match head.as_mut() {
        None => None,
        Some(next) if next.next.is_some() => last(&mut next.next),
        node @ Some(_) => node
    }
}

fn get(head: Option<&mut Box<ListNode>>, n: usize) -> Option<&mut Box<ListNode>> {
    if n == 0 {
        return head;
    }
    match head {
        None => None,
        Some(next) => get(next.next.as_mut(), n - 1),
    }
}

// concat two lists
fn concat(mut head: Option<Box<ListNode>>, tail: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
    match last(&mut head) {
        Some(x) => x.next = tail,
        None => head = tail
    }
    head
}

// another version of concat
fn concat2(mut head: Option<Box<ListNode>>, tail: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
    if let Some(ref mut inner_node) = head {
        let mut node = inner_node;
        while let Some(ref mut next) = node.next {
            node = next;
        }
        node.next = tail;
    }
    head
}

// another version of concat
fn concat3(mut head: Option<Box<ListNode>>, tail: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
    if tail.is_some() {
        if head.is_some() {
            let mut node = head.as_deref_mut().unwrap();
            while node.next.is_some() {
                node = node.next.as_deref_mut().unwrap();
            }
            node.next = tail;
        } else {
            head = tail;
        }
    }
    head
}

fn split(mut head: Option<Box<ListNode>>, mut n: usize) -> (Option<Box<ListNode>>, Option<Box<ListNode>>) {
    let list_len = len(&head);
    let split_idx = list_len - (n % list_len) - 1;

    match get(head.as_mut().take(), split_idx) {
        Some(split_node) => {
            let tail = split_node.next.take();
            (head, tail)
        },
        None => (head, None)
    }
}

fn to_values(mut node: &Option<Box<ListNode>>) -> Vec<i32> {
    let mut ret = Vec::new();
    while let Some(n) = node {
        ret.push(n.val);
        node = &n.next;
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotate_right_0() {
        let list = build_list(3);
        let rotated = rotate_right(list, 0);
        let values = to_values(&rotated);
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn test_rotate_right_1() {
        let list = build_list(3);
        let rotated = rotate_right(list, 1);
        let values = to_values(&rotated);
        assert_eq!(values, vec![3, 1, 2]);
    }

    #[test]
    fn test_rotate_right_2() {
        let list = build_list(3);
        let rotated = rotate_right(list, 2);
        let values = to_values(&rotated);
        assert_eq!(values, vec![2, 3, 1]);
    }

    #[test]
    fn test_rotate_right_3() {
        let list = build_list(3);
        let rotated = rotate_right(list, 3);
        let values = to_values(&rotated);
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn test_rotate_right_4() {
        let list = build_list(3);
        let rotated = rotate_right(list, 4);
        let values = to_values(&rotated);
        assert_eq!(values, vec![3, 1, 2]);
    }

    #[test]
    fn test_rotate_left_0() {
        let list = build_list(3);
        let rotated = rotate_left(list, 0);
        let values = to_values(&rotated);
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn test_rotate_left_1() {
        let list = build_list(3);
        let rotated = rotate_left(list, 1);
        let values = to_values(&rotated);
        assert_eq!(values, vec![2, 3, 1]);
    }

    #[test]
    fn test_rotate_left_2() {
        let list = build_list(3);
        let rotated = rotate_left(list, 2);
        let values = to_values(&rotated);
        assert_eq!(values, vec![3, 1, 2]);
    }

    #[test]
    fn test_rotate_left_3() {
        let list = build_list(3);
        let rotated = rotate_left(list, 3);
        let values = to_values(&rotated);
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn test_rotate_left_4() {
        let list = build_list(3);
        let rotated = rotate_left(list, 4);
        let values = to_values(&rotated);
        assert_eq!(values, vec![2, 3, 1]);
    }

    fn build_list(mut n: usize) -> Option<Box<ListNode>> {
        let mut list = None;
        if n > 0 {
            let mut head = Box::new(ListNode::new(n as i32));
            while n > 1 {
                n -= 1;
                let mut prev = Box::new(ListNode::new(n as i32));
                prev.next = Some(head);
                head = prev;
            }
            list = Some(head);
        }
        list
    }

}
