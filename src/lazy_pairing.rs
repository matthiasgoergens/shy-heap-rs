use crate::pairing::Pairing;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct LazyHeap<const CHUNKS: usize, T> {
    pub roots: Vec<Pairing<CHUNKS, T>>,
}

impl<const CHUNKS: usize, T> Default for LazyHeap<CHUNKS, T> {
    fn default() -> Self {
        Self { roots: vec![] }
    }
}

impl<const CHUNKS: usize, T: Ord> LazyHeap<CHUNKS, T> {
    pub fn insert(mut self, item: T) -> Self {
        self.roots.push(Pairing::new(item));
        self
    }
    // Oh, this is actually not useful, because we will want to delete multiple times
    // from the minimum element, when it's a pool of more than zero corrupted elements.
    pub fn delete_min(self) -> Self {
        match Pairing::merge_children(self.roots) {
            None => Self::default(),
            Some(pairing) => Self {
                roots: pairing.children,
            },
        }
    }
}

impl<const CHUNKS: usize, T> From<LazyHeap<CHUNKS, T>> for Vec<T> {
    fn from(heap: LazyHeap<CHUNKS, T>) -> Self {
        heap.roots.into_iter().flat_map(Vec::from).collect()
    }
}
