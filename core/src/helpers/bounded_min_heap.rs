use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub struct BoundedMinHeap<T: Ord> {
  heap: BinaryHeap<Reverse<T>>,
  capacity: usize,
}

impl<T: Ord> BoundedMinHeap<T> {
  pub fn new(capacity: usize) -> Self {
    Self {
      heap: BinaryHeap::new(),
      capacity,
    }
  }

  pub fn push(&mut self, item: T) {
    if self.heap.len() < self.capacity {
      self.heap.push(Reverse(item));
    } else if let Some(Reverse(min)) = self.heap.pop() {
      if item > min {
        self.heap.push(Reverse(item));
      } else {
        self.heap.push(Reverse(min));
      }
    }
  }

  pub fn pop(&mut self) -> Option<T> {
    self.heap.pop().map(|x| x.0)
  }

  pub fn peek(&self) -> Option<&T> {
    self.heap.peek().map(|x| &x.0)
  }

  pub fn drain(&mut self) -> Vec<T> {
    self.heap.drain().map(|x| x.0).collect()
  }
}
