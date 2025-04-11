// Soft heaps based on pairing heaps.
// We do min-heaps by default.

use std::collections::VecDeque;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Pool<T> {
    pub item: T,
    pub count: usize,
}

impl<T> Pool<T> {
    pub fn new(item: T) -> Self {
        Pool { item, count: 0 }
    }

    #[must_use]
    pub fn delete_one(self) -> Option<Self> {
        self.count
            .checked_sub(1)
            .map(|count| Self { count, ..self })
    }

    #[must_use]
    pub fn add_to_pool(mut self, count: usize) -> Self {
        self.count += count;
        self
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Pairing<const CHUNKS: usize, T> {
    pub key: Pool<T>,
    pub children: Vec<Pairing<CHUNKS, T>>,
}

impl<const CHUNKS: usize, T> From<Pool<T>> for Pairing<CHUNKS, T> {
    fn from(key: Pool<T>) -> Self {
        Self {
            key,
            children: vec![],
        }
    }
}

impl<const CHUNKS: usize, T> Pairing<CHUNKS, T> {
    pub fn new(item: T) -> Self {
        Self::from(Pool::new(item))
    }

    pub fn count_corrupted(&self) -> usize {
        self.key.count
            + self
                .children
                .iter()
                .map(Pairing::count_corrupted)
                .sum::<usize>()
    }

    pub fn count_uncorrupted(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(Pairing::count_uncorrupted)
            .sum::<usize>()
    }
}

impl<const CHUNKS: usize, T: Ord> Pairing<CHUNKS, T> {
    #[must_use]
    pub fn meld(self, other: Pairing<CHUNKS, T>) -> Pairing<CHUNKS, T> {
        let (mut a, b) = if self.key.item < other.key.item {
            (self, other)
        } else {
            (other, self)
        };
        a.children.push(b);
        a
    }

    #[must_use]
    pub fn insert(self, item: T) -> Self {
        self.meld(Self::new(item))
    }

    /// Corrupts the heap by pooling the top two elements.
    ///
    /// # Panics
    ///
    /// Panics if the heap property is violated (when the key's item is greater than
    /// the merged pairing's key item).
    #[must_use]
    pub fn corrupt(self, corrupted: &mut Vec<T>) -> Self {
        let Pairing { key, children } = self;
        match Self::merge_children(children, corrupted) {
            None => Pairing::from(key),
            Some(pairing) => {
                assert!(key.item <= pairing.key.item);
                corrupted.push(key.item);
                Pairing {
                    key: pairing.key.add_to_pool(key.count),
                    children: pairing.children,
                }
            }
        }
    }

    pub fn delete_min(self) -> (Option<Self>, Vec<T>) {
        let mut corrupted = vec![];
        let Pairing { key, children } = self;
        (
            match key.delete_one() {
                None => Self::merge_children(children, &mut corrupted),
                Some(key) => Some(Self { key, children }),
            },
            corrupted,
        )
    }

    /// Merges the list of children of a (former) node into one node.
    ///
    /// See 'A Nearly-Tight Analysis of Multipass Pairing Heaps"
    /// by Corwin Sinnamon and Robert E. Tarjan.
    /// <https://epubs.siam.org/doi/epdf/10.1137/1.9781611977554.ch23>
    ///
    /// The paper explains why multipass (like here) does give O(log n)
    /// delete-min.
    ///
    /// (Originally O(log n) delete-min was only proven for the two-pass
    /// variant.)
    #[must_use]
    pub fn merge_children(items: Vec<Self>, corrupted: &mut Vec<T>) -> Option<Self> {
        let mut d = VecDeque::from(items);

        for i in 1.. {
            match (d.pop_front(), d.pop_front()) {
                (None, _) => return None,
                (Some(a), None) => return Some(a),
                (Some(a), Some(b)) => {
                    let a = a.meld(b);
                    d.push_back(if i % CHUNKS == 0 {
                        a.corrupt(corrupted)
                    } else {
                        a
                    });
                }
            }
        }
        unreachable!();
    }

    pub fn check_heap_property(&self) -> bool {
        let Pairing { key, children } = self;
        children.iter().all(|child| key.item <= child.key.item)
            && children.iter().all(Self::check_heap_property)
    }
}

// Get all non-corrupted elements still in the heap.
impl<const CHUNKS: usize, T> From<Pairing<CHUNKS, T>> for Vec<T> {
    fn from(pairing: Pairing<CHUNKS, T>) -> Self {
        // Pre-order traversal.
        let mut items = vec![];
        let mut todo = VecDeque::from([pairing]);
        while let Some(pairing) = todo.pop_front() {
            let Pairing {
                key: Pool { item, count: _ },
                children,
            } = pairing;
            todo.extend(children);
            items.push(item);
        }
        items
    }
}

// Idea: look at my 'static visualisation' of sorting algorithms for various sequences of operations.
// Also: add tests etc.
// Also: actually use the soft pairing heap for my Schubert matroid.

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct SoftHeap<const CHUNKS: usize, T> {
    pub root: Option<Pairing<CHUNKS, T>>,
}

impl<const CHUNKS: usize, T> Default for SoftHeap<CHUNKS, T> {
    fn default() -> Self {
        Self { root: None }
    }
}

impl<const CHUNKS: usize, T: Ord> SoftHeap<CHUNKS, T> {
    #[must_use]
    pub fn insert(self, item: T) -> Self {
        match self.root {
            None => Self {
                root: Some(Pairing::new(item)),
            },
            Some(root) => Self {
                root: Some(root.insert(item)),
            },
        }
    }

    #[must_use]
    pub fn delete_min(self) -> (Self, Vec<T>) {
        // TODO: simplify.
        match self.root {
            None => (Self::default(), vec![]),
            Some(root) => {
                let (root, corrupted) = root.delete_min();
                (Self { root }, corrupted)
            }
        }
    }
    pub fn count_corrupted(&self) -> usize {
        self.root.as_ref().map_or(0, Pairing::count_corrupted)
    }
    pub fn count_uncorrupted(&self) -> usize {
        self.root.as_ref().map_or(0, Pairing::count_uncorrupted)
    }
}

impl<const CHUNKS: usize, T> From<SoftHeap<CHUNKS, T>> for Vec<T> {
    fn from(SoftHeap { root }: SoftHeap<CHUNKS, T>) -> Self {
        root.map(Vec::from).unwrap_or_default()
    }
}
