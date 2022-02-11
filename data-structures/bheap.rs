struct BHeap<T> {
    data: Vec<T>,
    ordering: Ordering
}

enum Kind {
    MAX, MIN
}

// A binary heap - either MAX or MIN - with O(log n) complexity for each operation.
impl <T> BHeap<T> where T: Ord + Debug + Copy {
    fn new(kind: Kind) -> Self {
        let ordering = match kind {
            Kind::MAX => Ordering::Greater,
            Kind::MIN => Ordering::Less
        };
        Self { data: Vec::new(), ordering }
    }

    fn push(&mut self, elem: T) {
        self.data.push(elem.clone());
        self.heapify_bottom_up(self.data.len() - 1);
    }

    fn pop(&mut self) -> Option<T> {
        self.del(0)
    }

    fn del(&mut self, idx: usize) -> Option<T> {
        let len = self.data.len();
        if len > 0 && idx < len {
            self.data.swap(idx, len - 1);
            let result = self.data.pop().unwrap();
            self.heapify_top_down(idx);
            Some(result)
        } else {
            None
        }
    }

    fn heapify_bottom_up(&mut self, mut position: usize) {
        if position > 0 {
            let parent_position = (position - 1) / 2;
            if self.data[position].cmp(&self.data[parent_position]) == self.ordering {
                self.data.swap(position, parent_position);
                self.heapify_bottom_up(parent_position);
            }
        }
    }

    fn heapify_top_down(&mut self, mut position: usize) {
        let left = position * 2 + 1;
        let right = position * 2 + 2;
        if right < self.data.len()
            && self.data[right].cmp(&self.data[left]) == self.ordering
            && self.data[right].cmp(&self.data[position]) == self.ordering {
                self.data.swap(position, right);
                self.heapify_top_down(right);
        } else if left < self.data.len()
            && self.data[left].cmp(&self.data[position]) == self.ordering{
            self.data.swap(position, left);
            self.heapify_top_down(left);
        }
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heap_sort_min_heap() {
        let mut heap = BHeap::new(Kind::MIN);
        heap.push(3);
        heap.push(5);
        heap.push(1);
        heap.push(4);
        heap.push(2);
        let result = into_values(heap);
        assert_eq!(result, Some(vec![1, 2, 3, 4, 5]));
    }

    #[test]
    fn test_heap_sort_max_heap() {
        let mut heap = BHeap::new(Kind::MAX);
        heap.push(3);
        heap.push(5);
        heap.push(1);
        heap.push(4);
        heap.push(2);
        let result = into_values(heap);
        assert_eq!(result, Some(vec![5, 4, 3, 2, 1]));
    }

    fn into_values<T>(mut heap: BHeap<T>) -> Option<Vec<T>> where T: Ord + Debug + Copy {
        (0..heap.len()).map(|idx| heap.pop()).collect::<Option<Vec<T>>>()
    }
}
