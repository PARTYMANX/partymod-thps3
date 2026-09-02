enum ArenaEntry<T> {
    Occupied(T),
    Free(Option<usize>),
}

pub struct GenArena<T> {
    // TODO: add count
    list: Vec<ArenaEntry<T>>,
    generations: Vec<u64>,
    first_open: Option<usize>,
}

#[derive(Copy, Clone)]
pub struct GenArenaKey {
    index: usize,
    generation: u64,
}

impl<T> GenArena<T> {
    pub fn new() -> Self {
        Self {
            list: Vec::new(),
            generations: Vec::new(),
            first_open: None,
        }
    }

    pub fn push(&mut self, val: T) -> GenArenaKey {
        // find open index, if it exists
        if let Some(idx) = self.first_open {
            match self.list[idx] {
                ArenaEntry::Occupied(_) => panic!("First open index was occupied!"),
                ArenaEntry::Free(v) => self.first_open = v,
            }

            self.list[idx] = ArenaEntry::Occupied(val);

            self.generations[idx] += 1;

            GenArenaKey {
                generation: self.generations[idx],
                index: idx,
            }
        } else {
            let idx = self.list.len();

            self.list.push(ArenaEntry::Occupied(val));
            self.generations.push(0);

            GenArenaKey {
                generation: 0,
                index: idx,
            }
        }
    }

    #[allow(unused)]
    pub fn free(&mut self, key: GenArenaKey) -> bool {
        if key.index >= self.list.len() {
            return false;
        }

        if key.generation == self.generations[key.index] {
            match self.list[key.index] {
                ArenaEntry::Occupied(_) => {
                    self.list[key.index] = ArenaEntry::Free(self.first_open);
                    return true;
                }
                ArenaEntry::Free(_) => return false,
            }
        }

        return false;
    }

    pub fn get(&self, key: GenArenaKey) -> Option<&T> {
        if key.index >= self.list.len() {
            return None;
        }

        if key.generation != self.generations[key.index] {
            return None;
        }

        if let ArenaEntry::Occupied(data) = &self.list[key.index] {
            return Some(data);
        } else {
            return None;
        }
    }

    pub fn get_mut(&mut self, key: GenArenaKey) -> Option<&mut T> {
        if key.index >= self.list.len() {
            return None;
        }

        if key.generation != self.generations[key.index] {
            return None;
        }

        if let ArenaEntry::Occupied(data) = &mut self.list[key.index] {
            return Some(data);
        } else {
            return None;
        }
    }

    #[allow(unused)]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            iter: self.list.iter(),
        }
    }

    #[allow(unused)]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            iter: self.list.iter_mut(),
        }
    }
}

pub struct Iter<'a, T> {
    iter: core::slice::Iter<'a, ArenaEntry<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(slot) = self.iter.next() {
            if let ArenaEntry::Occupied(item) = slot {
                return Some(item);
            }
        }

        None
    }
}

pub struct IterMut<'a, T> {
    iter: core::slice::IterMut<'a, ArenaEntry<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(slot) = self.iter.next() {
            if let ArenaEntry::Occupied(item) = slot {
                return Some(item);
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use std::assert_eq;

    use super::GenArena;

    #[test]
    fn test() {
        let mut arena = GenArena::new();

        let a = arena.push(1);
        let b = arena.push(2);

        assert_eq!(*arena.get(a).unwrap(), 1);
        assert_eq!(*arena.get(b).unwrap(), 2);

        let mut_a = arena.get_mut(a).unwrap();
        *mut_a = 4;

        assert_eq!(*arena.get(a).unwrap(), 4);

        arena.free(a);

        assert!(matches!(arena.get(a), None));
    }

    #[test]
    fn test_iter() {
        let mut arena = GenArena::new();

        let _ = arena.push(1);
        let b = arena.push(2);
        let _ = arena.push(3);

        let mut i1 = arena.iter();
        assert_eq!(i1.next(), Some(&1));
        assert_eq!(i1.next(), Some(&2));
        assert_eq!(i1.next(), Some(&3));
        assert_eq!(i1.next(), None);

        arena.free(b);

        let mut i2 = arena.iter();
        assert_eq!(i2.next(), Some(&1));
        assert_eq!(i2.next(), Some(&3));
        assert_eq!(i2.next(), None);
    }
}
