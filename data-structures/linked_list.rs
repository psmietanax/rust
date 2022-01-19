use std::fmt::{Debug, Display};

#[derive(Debug)]
struct ListNode<V> {
    value: V,
    next: Option<Box<ListNode<V>>>
}

impl <V> ListNode<V> {
    fn of(value: V) -> Self {
        ListNode { value, next: None }
    }
}

struct LinkedList<V> {
    head: Option<Box<ListNode<V>>>,
    size: usize
}

impl <V> LinkedList<V> where V: Display + Debug {
    fn new() -> Self {
        LinkedList { head: None, size: 0 }
    }

    fn push_front(&mut self, value: V) {
        let node = Some(Box::new(ListNode::of(value)));
        if self.head.is_none() {
            self.head = node;
        } else {
            let current_head = self.head.replace(node.unwrap());
            self.head.as_mut().unwrap().next = current_head;
        }
        self.size += 1;
    }

    fn pop_front(&mut self) -> Option<V> {
        if self.head.is_some() {
            self.size -= 1;
            if self.head.as_ref().unwrap().next.is_some() {
                let next_node = self.head.as_mut().unwrap().next.take().unwrap();
                self.head.replace(next_node).map(|b| b.value)
            } else {
                self.head.take().map(|b| b.value)
            }
        } else {
            None
        }
    }

    fn push_back(&mut self, value: V) {
        let node = Some(Box::new(ListNode::of(value)));
        match self.head.as_mut() {
            None => {
                self.head = node;
            },
            Some(head) => {
                let mut tail = head;
                while tail.next.is_some() {
                    tail = tail.next.as_mut().unwrap();
                }
                tail.next = node;
            }
        }
        self.size += 1;
    }

    fn pop_back(&mut self) -> Option<V> {
        if self.head.is_some() {
            self.size -= 1;
            if self.head.as_ref().unwrap().next.is_some() {
                let mut tail = self.head.as_mut().unwrap();
                while tail.next.as_ref().unwrap().next.is_some() {
                    tail = tail.next.as_mut().unwrap();
                }
                tail.next.take().map(|b| b.value)
            } else {
                self.head.take().map(|b| b.value)
            }
        } else {
            None
        }
    }

    fn size(&self) -> usize {
        self.size
    }

    fn print(&self) {
        if let Some(head) = self.head.as_ref() {
            let mut tail = head;
            while tail.next.is_some() {
                print!("{:?} -> ", tail.value);
                tail = tail.next.as_ref().unwrap();
            }
            println!("{:?}", tail.value);
        }
    }
}

struct LinkedListIter<'a, V> {
    linked_list: Option<&'a LinkedList<V>>,
    node: Option<&'a Box<ListNode<V>>>
}

impl <'a, V> Iterator for LinkedListIter<'a, V> where V: Debug {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        if self.node.is_none() && self.linked_list.is_some() {
            self.node = self.linked_list.unwrap().head.as_ref();
            self.linked_list = None;
        } else if self.node.is_some() {
            self.node = self.node.unwrap().next.as_ref();
        }
        self.node.map(|b| &b.value)
    }
}

impl <'a, V> IntoIterator for &'a LinkedList<V> where V: Debug {
    type Item = &'a V;
    type IntoIter = LinkedListIter<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        LinkedListIter { linked_list: Some(&self), node: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_front() {
        let mut linkedlist = LinkedList::new();
        linkedlist.push_front(10);
        linkedlist.push_front(20);
        linkedlist.push_front(30);
        linkedlist.push_front(40);
        let values = linkedlist.into_iter().collect::<Vec<&i32>>();
        assert_eq!(values, vec![&40, &30, &20, &10]);

    }

    #[test]
    fn test_pop_front_many_elements() {
        let mut linkedlist = LinkedList::new();
        linkedlist.push_front(10);
        linkedlist.push_front(20);
        linkedlist.push_front(30);
        linkedlist.push_front(40);
        let popped = linkedlist.pop_front();
        assert_eq!(popped, Some(40));
        let values = linkedlist.into_iter().collect::<Vec<&i32>>();
        assert_eq!(values, vec![&30, &20, &10]);
    }

    #[test]
    fn test_pop_front_one_element() {
        let mut linkedlist = LinkedList::new();
        linkedlist.push_front(10);
        let popped = linkedlist.pop_front();
        assert_eq!(popped, Some(10));
        let popped = linkedlist.pop_front();
        assert_eq!(popped, None);
    }

    #[test]
    fn test_pop_front_zero_elements() {
        let mut linkedlist = LinkedList::<i32>::new();
        let popped = linkedlist.pop_front();
        assert_eq!(popped, None);
    }

    #[test]
    fn test_push_back() {
        let mut linkedlist = LinkedList::new();
        linkedlist.push_back(10);
        linkedlist.push_back(20);
        linkedlist.push_back(30);
        linkedlist.push_back(40);
        let values = linkedlist.into_iter().collect::<Vec<&i32>>();
        assert_eq!(values, vec![&10, &20, &30, &40]);
    }

    #[test]
    fn test_pop_back_many_elements() {
        let mut linkedlist = LinkedList::new();
        linkedlist.push_back(10);
        linkedlist.push_back(20);
        linkedlist.push_back(30);
        linkedlist.push_back(40);
        let popped = linkedlist.pop_back();
        assert_eq!(popped, Some(40));
        let values = linkedlist.into_iter().collect::<Vec<&i32>>();
        assert_eq!(values, vec![&10, &20, &30]);
    }

    #[test]
    fn test_pop_back_one_element() {
        let mut linkedlist = LinkedList::new();
        linkedlist.push_back(10);
        let popped = linkedlist.pop_back();
        assert_eq!(popped, Some(10));
        let popped = linkedlist.pop_back();
        assert_eq!(popped, None);
    }

    #[test]
    fn test_pop_back_zero_elements() {
        let mut linkedlist = LinkedList::<i32>::new();
        let popped = linkedlist.pop_back();
        assert_eq!(popped, None);
    }
}
