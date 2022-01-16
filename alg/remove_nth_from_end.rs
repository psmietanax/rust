fn main() {
    let mut node1 = Box::new(ListNode::new(1));
    let mut node2 = Box::new(ListNode::new(2));
    let mut node3 = Box::new(ListNode::new(3));
    let mut node4 = Box::new(ListNode::new(4));
    let mut node5 = Box::new(ListNode::new(5));
    node4.next = Some(node5);
    node3.next = Some(node4);
    node2.next = Some(node3);
    node1.next = Some(node2);

    println!("{:?}", remove_nth_from_end(Some(node1), 5));

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

// Given the head of a linked list, remove the nth node from the end of the list and return its head.
// https://leetcode.com/problems/remove-nth-node-from-end-of-list/
fn remove_nth_from_end(head: Option<Box<ListNode>>, n: usize) -> Option<Box<ListNode>> {
    // get the list size to count how many nodes to move ahead before removing a node
    let list_size = list_size(&head);

    // add aux node so we can also remove the first node
    let mut aux = Box::new(ListNode::new(0));
    aux.next = head;

    // we're going to mutate the list so need the &mut reference
    let mut node = &mut aux;

    // move forward by the computed count
    for _ in 0..list_size - n {
        node = node.next.as_mut().unwrap();
    }
    // assign the new node
    // use Option#take() so we can move out from a shared reference
    node.next = node.next.take().unwrap().next;

    aux.next
}

fn list_size(head: &Option<Box<ListNode>>) -> usize {
    if head.is_none() {
        0
    } else {
        1 + list_size(&head.as_ref().unwrap().next)
    }
}

fn values(mut head: &Option<Box<ListNode>>) -> Vec<i32> {
    let mut result = Vec::new();
    while head.is_some() {
        result.push(head.as_ref().unwrap().val);
        head = &head.as_ref().unwrap().next;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_values() {
        let node = create_list(5);
        assert_eq!(values(&node), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_list_size() {
        let node = create_list(10);
        assert_eq!(list_size(&node), 10);
    }

    #[test]
    fn test_remove_first_from_end() {
        let node = create_list(5);
        let result = remove_nth_from_end(node, 1);
        assert_eq!(values(&result), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_remove_last_from_end() {
        let node = create_list(5);
        let result = remove_nth_from_end(node, 5);
        assert_eq!(values(&result), vec![2, 3, 4, 5]);
    }

    #[test]
    fn test_remove_middle_from_end() {
        let node = create_list(5);
        let result = remove_nth_from_end(node, 3);
        assert_eq!(values(&result), vec![1, 2, 4, 5]);
    }

    fn create_list(mut n: i32) -> Option<Box<ListNode>> {
        let mut node = Some(Box::new(ListNode::new(n)));
        n -= 1;
        while n > 0 {
            let mut prev = Some(Box::new(ListNode::new(n)));
            prev.as_mut().unwrap().next = node;
            node = prev;
            n -= 1;
        }
        node
    }

}
