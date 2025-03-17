// Soft heaps based on pairing heaps.
// We do min-heaps by default.

use itertools::Itertools;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pool<T> {
    pub item: T,
    pub count: isize,
}

impl<T> Pool<T> {
    pub fn new(item: T) -> Self {
        Pool { item, count: 0 }
    }

    pub fn pop(self) -> (Option<T>, Option<Self>) {
        assert!(self.count >= 0);
        if self.count <= 0 {
            (Some(self.item), None)
        } else {
            (
                None,
                Some(Self {
                    count: self.count - 1,
                    ..self
                }),
            )
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pairing<T> {
    pub key: Pool<T>,
    pub children: Vec<Pairing<T>>,
}
impl<T> From<Pool<T>> for Pairing<T> {
    fn from(key: Pool<T>) -> Self {
        Self {
            key,
            children: vec![],
        }
    }
}

impl<T> Pairing<T> {
    pub fn new(item: T) -> Self {
        Self::from(Pool::new(item))
    }
}

// pub fn sift(a: T) -> Pool<T> {
//     todo!();
// }

impl<T: Ord> Pairing<T> {
    pub fn meld(self, other: Pairing<T>) -> Pairing<T> {
        let (mut a, b) = if self.key.item < other.key.item {
            (self, other)
        } else {
            (other, self)
        };
        a.children.push(b);
        a
    }

    pub fn insert(self, item: T) -> Self {
        self.meld(Self::new(item))
    }

    pub fn pop_min(self) -> (Option<T>, Option<Self>) {
        let Pairing { key, children } = self;
        let (popped, remainder) = key.pop();
        (
            popped,
            match remainder {
                None => Self::merge_pairs(children),
                Some(key) => Some(Self { key, children }),
            },
        )
    }

    pub fn delete_min(self) -> Option<Self> {
        let (_, children) = self.pop_min();
        children
    }

    pub fn merge_pairs(items: Vec<Self>) -> Option<Self> {
        items
            .into_iter()
            .chunks(2)
            .into_iter()
            .filter_map(|pair| pair.reduce(Self::meld))
            .reduce(Self::meld)
    }
}
