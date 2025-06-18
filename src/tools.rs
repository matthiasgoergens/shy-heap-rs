#[must_use]
pub fn previous_full_multiple(n: usize, m: usize) -> usize {
    (n + 1).next_multiple_of(m) - m
}


use std::cell::Cell;
use std::cmp::Ordering;
use std::rc::Rc;

/* ---------- counted wrapper ---------- */

#[derive(Debug)]
pub struct Counted<T> {
    value: T,
    counter: Rc<Cell<usize>>,
}

impl<T> Counted<T> {
    fn new(value: T, counter: &Rc<Cell<usize>>) -> Self {
        Self {
            value,
            counter: counter.clone(),
        }
    }

    pub fn into_inner(self) -> T {
        self.value
    }
}

/* --- trait impls (unchanged in spirit) --- */

impl<T: Ord> Ord for Counted<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.counter.set(self.counter.get() + 1);
        self.value.cmp(&other.value)
    }
}
impl<T: Ord> PartialOrd for Counted<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T: Ord> PartialEq for Counted<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl<T: Ord> Eq for Counted<T> {}

/* ---------- helper: wrap a Vec and hand the counter back ---------- */

/// Consumes a vector, wraps every element, and returns
/// `(shared_counter, wrapped_vector)`.
pub fn with_counter<T: Ord>(v: Vec<T>) -> (Rc<Cell<usize>>, Vec<Counted<T>>) {
    let counter = Rc::new(Cell::new(0));
    let wrapped = v
        .into_iter()
        .map(|x| Counted::new(x, &counter))
        .collect();
    (counter, wrapped)
}

/* ---------- demo ---------- */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_comparisons() {
        let (counter, mut v) = with_counter(vec![3, 1, 4, 1, 5, 9, 2, 6]);
        v.sort();

        assert!(counter.get() > 0);
        let sorted: Vec<_> = v.into_iter().map(Counted::into_inner).collect();
        assert_eq!(sorted, vec![1, 1, 2, 3, 4, 5, 6, 9]);
        println!("comparisons: {}", counter.get());
    }
}
