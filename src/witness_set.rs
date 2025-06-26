#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Witnessed<T> {
    pub item: T,
    pub count: usize,
    pub children: WitnessedSet<T>,
}

impl<T> Witnessed<T> {
    pub fn singleton(item: T) -> Self {
        Self {
            item,
            count: 1,
            children: WitnessedSet::default(),
        }
    }

    pub fn add_child(&mut self, child: Witnessed<T>) {
        self.count += child.count;
        self.children.add_child(child);
    }

    pub fn in_order(self, result: &mut Vec<T>) {
        result.push(self.item);
        for child in self.children.items {
            child.in_order(result);
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct WitnessedSet<T> {
    pub count: usize,
    pub items: Vec<Witnessed<T>>,
}

impl<T> Default for WitnessedSet<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            count: 0,
        }
    }
}

impl<T> WitnessedSet<T> {
    pub fn add_child(&mut self, child: Witnessed<T>) {
        self.count += child.count;
        self.items.push(child);
    }

    pub fn extend(&mut self, other: WitnessedSet<T>) {
        self.count += other.count;
        self.items.extend(other.items);
    }
    pub fn in_order(self, result: &mut Vec<T>) {
        for witnessed in self.items {
            witnessed.in_order(result);
        }
    }
}

impl<T> From<WitnessedSet<T>> for Vec<T> {
    fn from(set: WitnessedSet<T>) -> Self {
        let mut result = Vec::new();
        set.in_order(&mut result);
        result
    }
}

impl<T> From<Witnessed<T>> for Vec<T> {
    fn from(witnessed: Witnessed<T>) -> Self {
        let mut result = Vec::new();
        witnessed.in_order(&mut result);
        result
    }
}
