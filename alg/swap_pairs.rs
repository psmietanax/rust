fn main() {
    let mut root = Box::new(ListNode::new(1));
    let mut next = Box::new(ListNode::new(2));
    let mut next_next = Box::new(ListNode::new(3));
    let mut next_next_next = Box::new(ListNode::new(4));
    next_next.next = Some(next_next_next);
    next.next = Some(next_next);
    root.next = Some(next);
    let x = swap_pairs(root);
    println!("{:?}", x);
}

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

fn swap_pairs(mut node: Box<ListNode>) -> Box<ListNode> {
    if node.next.is_some() {
        let mut next_node = node.next.take().unwrap();
        if next_node.next.is_some() {
            let next_next_node = swap_pairs(next_node.next.take().unwrap());
            node.next = Some(next_next_node);
        }
        next_node.next = Some(node);
        next_node
    } else {
        node
    }
}

fn to_values(mut node: &Box<ListNode>) -> Vec<i32> {
    let mut ret = Vec::new();
    ret.push(node.val);
    while node.next.is_some() {
        node = node.next.as_ref().unwrap();
        ret.push(node.val);
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_pairs_odd() {
        let list = build_list(5);
        let swapped = swap_pairs(list);
        assert_eq!(to_values(&swapped), vec![2, 1, 4, 3, 5])
    }

    #[test]
    fn test_swap_pairs_even() {
        let list = build_list(6);
        let swapped = swap_pairs(list);
        assert_eq!(to_values(&swapped), vec![2, 1, 4, 3, 6, 5])
    }

    fn build_list(mut n: usize) -> Box<ListNode> {
        let mut head = Box::new(ListNode::new(n as i32));
        while n > 1 {
            n -= 1;
            let mut prev = Box::new(ListNode::new(n as i32));
            prev.next = Some(head);
            head = prev;
        }
        head
    }

}
