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

// https://leetcode.com/problems/reverse-linked-list/
fn reverse_list(mut head: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
    let mut prev = None;
    while let Some(mut node) = head {
        let next = node.next.take();
        node.next = prev;
        prev = Some(node);
        head = next;
    }
    prev
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
    fn test_reverse_list_empty() {
        let list = build_list(0);
        let reversed = reverse_list(list);
        assert_eq!(to_values(&reversed).is_empty(), true)
    }

    #[test]
    fn test_reverse_list_single_element() {
        let list = build_list(1);
        let reversed = reverse_list(list);
        assert_eq!(to_values(&reversed), vec![1])
    }

    #[test]
    fn test_reverse_list() {
        let list = build_list(5);
        let reversed = reverse_list(list);
        assert_eq!(to_values(&reversed), vec![5, 4, 3, 2, 1])
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
