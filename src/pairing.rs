// Soft heaps based on pairing heaps.
// We do min-heaps by default.

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pool<T> {
    pub item: T,
    pub count: isize,
}

impl<T> Pool<T> {
    pub fn new(item: T) -> Self {
        Pool { item, count: 0 }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pairing<T> {
    pub key: Pool<T>,
    pub children: Vec<Pairing<T>>,
}

impl<T> Pairing<T> {
    pub fn new(key: Pool<T>) -> Self {
        Pairing {
            key,
            children: vec![],
        }
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
        self.meld(Pairing::new(Pool::new(item)))
    }

    pub fn pop_min(self) -> (Pool<T>, Option<Self>) {
        let Pairing { key, children } = self;
        (
            key,
            if children.is_empty() {
                None
            } else {
                todo!();
                // let mut children = children.into_iter();
                // let first = children.next().unwrap();
                // let rest = children.fold(Pairing::new(Pool::new(None)), |acc, x| acc.meld(x));
                // Some(rest)
                None
            },
        )
    }
}
