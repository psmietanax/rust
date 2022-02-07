use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::ops::Deref;
use std::rc::{Rc, Weak};
#[derive(Debug)]
struct DoublyListNode<V> {
    value: V,
    next: Option<Rc<RefCell<DoublyListNode<V>>>>,
    // weak references are used to eliminate cycles (memory leaks)
    prev: Option<Weak<RefCell<DoublyListNode<V>>>>
}

impl <V> DoublyListNode<V> {
    fn of(value: V) -> Self {
        DoublyListNode { value, next: None, prev: None }
    }
}

#[derive(Debug)]
struct DoublyLinkedList<V> {
    head: Option<Rc<RefCell<DoublyListNode<V>>>>,
    tail: Option<Rc<RefCell<DoublyListNode<V>>>>,
    size: usize
}

impl <V> DoublyLinkedList<V> where V: Display + Debug {
    fn new() -> Self {
        DoublyLinkedList { head: None, tail: None, size: 0 }
    }

    fn push_front(&mut self, value: V) {
        let mut node = Rc::new(RefCell::new(DoublyListNode::of(value)));
        match self.head.take() {
            None => {
                self.tail = Some(Rc::clone(&node));
                self.head = Some(node);
            },
            Some(old_head) => {
                old_head.borrow_mut().prev = Some(Rc::downgrade(&node));
                node.borrow_mut().next = Some(old_head);
                self.head = Some(node);
            }
        }

        self.size += 1;
    }

    fn pop_front(&mut self) -> Option<V> {
        self.head.take().map(|old_head| {
            match old_head.borrow_mut().next.take() {
                Some(new_head) => {
                    new_head.borrow_mut().prev.take();
                    self.head = Some(new_head);
                }
                None => {
                    self.tail.take();
                }
            }
            self.size -= 1;
            Rc::try_unwrap(old_head).ok().unwrap().into_inner().value
        })
    }

    fn push_back(&mut self, value: V) {
        let mut node = Rc::new(RefCell::new(DoublyListNode::of(value)));
        match self.tail.take() {
            None => {
                self.tail = Some(Rc::clone(&node));
                self.head = Some(node);
            },
            Some(old_tail) => {
                node.borrow_mut().prev = Some(Rc::downgrade(&old_tail));
                old_tail.borrow_mut().next = Some(Rc::clone(&node));
                self.tail = Some(node);
            }
        }

        self.size += 1;
    }

    fn pop_back(&mut self) -> Option<V> {
        self.tail.take().map(|old_tail| {
            match old_tail.borrow_mut().prev.take() {
                Some(new_tail) => {
                    let upgraded = new_tail.upgrade().unwrap();
                    upgraded.borrow_mut().next.take();
                    self.tail = Some(upgraded);
                }
                None => {
                    self.head.take();
                }
            }
            self.size -= 1;
            Rc::try_unwrap(old_tail).ok().unwrap().into_inner().value
        })
    }

    fn size(&self) -> usize {
        self.size
    }

}

impl <V> Iterator for DoublyLinkedList<V> where V: Display + Debug {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_front() {
        let mut linkedlist = DoublyLinkedList::new();
        linkedlist.push_front(10);
        linkedlist.push_front(20);
        linkedlist.push_front(30);
        linkedlist.push_front(40);
        let values = linkedlist.into_iter().collect::<Vec<i32>>();
        assert_eq!(values, vec![40, 30, 20, 10]);

    }

    #[test]
    fn test_pop_front_many_elements() {
        let mut linkedlist = DoublyLinkedList::new();
        linkedlist.push_front(10);
        linkedlist.push_front(20);
        linkedlist.push_front(30);
        linkedlist.push_front(40);
        let popped = linkedlist.pop_front();
        assert_eq!(popped, Some(40));
        let values = linkedlist.into_iter().collect::<Vec<i32>>();
        assert_eq!(values, vec![30, 20, 10]);
    }

    #[test]
    fn test_pop_front_one_element() {
        let mut linkedlist = DoublyLinkedList::new();
        linkedlist.push_front(10);
        let popped = linkedlist.pop_front();
        assert_eq!(popped, Some(10));
        let popped = linkedlist.pop_front();
        assert_eq!(popped, None);
    }

    #[test]
    fn test_pop_front_zero_elements() {
        let mut linkedlist = DoublyLinkedList::<i32>::new();
        let popped = linkedlist.pop_front();
        assert_eq!(popped, None);
    }

    #[test]
    fn test_push_back() {
        let mut linkedlist = DoublyLinkedList::new();
        linkedlist.push_back(10);
        linkedlist.push_back(20);
        linkedlist.push_back(30);
        linkedlist.push_back(40);
        let values = linkedlist.into_iter().collect::<Vec<i32>>();
        assert_eq!(values, vec![10, 20, 30, 40]);
    }

    #[test]
    fn test_pop_back_many_elements() {
        let mut linkedlist = DoublyLinkedList::new();
        linkedlist.push_back(10);
        linkedlist.push_back(20);
        linkedlist.push_back(30);
        linkedlist.push_back(40);
        let popped = linkedlist.pop_back();
        assert_eq!(popped, Some(40));
        let values = linkedlist.into_iter().collect::<Vec<i32>>();
        assert_eq!(values, vec![10, 20, 30]);
    }

    #[test]
    fn test_pop_back_one_element() {
        let mut linkedlist = DoublyLinkedList::new();
        linkedlist.push_back(10);
        let popped = linkedlist.pop_back();
        assert_eq!(popped, Some(10));
        let popped = linkedlist.pop_back();
        assert_eq!(popped, None);
    }

    #[test]
    fn test_pop_back_zero_elements() {
        let mut linkedlist = DoublyLinkedList::<i32>::new();
        let popped = linkedlist.pop_back();
        assert_eq!(popped, None);
    }
}
