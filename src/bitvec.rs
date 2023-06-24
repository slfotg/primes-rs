use bitvec::vec::BitVec;

#[derive(Debug, Clone)]
pub struct MappedBitVec {
    vec: BitVec,
    offset: usize,
    modulus: usize,
}

impl MappedBitVec {
    pub fn new(vec: BitVec, modulus: usize, offset: usize) -> MappedBitVec {
        MappedBitVec {
            vec,
            offset,
            modulus,
        }
    }

    pub fn max_len(&self) -> usize {
        self.vec.len()
    }

    pub fn len(&self) -> usize {
        self.vec.count_ones()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn set(&mut self, index: usize, val: bool) {
        self.vec.set(index, val);
    }

    pub fn get(&self, index: usize) -> usize {
        self.apply(index)
    }

    fn apply(&self, i: usize) -> usize {
        i * self.modulus + self.offset
    }

    pub fn first_one(&self, next_index: usize) -> Option<(usize, usize)> {
        self.vec[next_index..].first_one().map(|x| {
            let next = x + next_index;
            (next, self.get(next))
        })
    }

    pub fn to_vec(&self) -> Vec<usize> {
        let mut vec = Vec::with_capacity(self.len());
        vec.extend(self.vec.iter_ones().map(|i| self.apply(i)));
        vec
    }
}

impl IntoIterator for MappedBitVec {
    type Item = usize;

    type IntoIter = MappedBitVecIterator;

    fn into_iter(self) -> Self::IntoIter {
        MappedBitVecIterator {
            index: 0,
            vec: self,
        }
    }
}

pub struct MappedBitVecIterator {
    index: usize,
    vec: MappedBitVec,
}

impl Iterator for MappedBitVecIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((index, prime)) = self.vec.first_one(self.index) {
            self.index = index + 1;
            Some(prime)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MappedBitVec;
    use bitvec::prelude::*;

    #[test]
    fn test_new() {
        let x = MappedBitVec::new(bitvec![1; 5], 7, 1).to_vec();
        assert_eq!(vec![1, 8, 15, 22, 29], x);
    }

    #[test]
    fn test_len() {
        let x = MappedBitVec::new(bitvec![1; 5], 7, 1);
        assert_eq!(5, x.len());
    }

    #[test]
    fn test_is_empty() {
        let x = MappedBitVec::new(bitvec![1; 5], 7, 1);
        assert!(!x.is_empty());

        let x = MappedBitVec::new(bitvec![], 7, 1);
        assert!(x.is_empty());
    }

    #[test]
    fn test_set() {
        let mut x = MappedBitVec::new(bitvec![1; 5], 7, 1);
        x.set(2, false);
        assert_eq!(vec![1, 8, 22, 29], x.to_vec());
    }

    #[test]
    fn test_get() {
        let x = MappedBitVec::new(bitvec![1; 5], 7, 1);
        assert_eq!(1, x.get(0));
        assert_eq!(8, x.get(1));
        assert_eq!(15, x.get(2));
        assert_eq!(22, x.get(3));
        assert_eq!(29, x.get(4));

        let x = MappedBitVec::new(bitvec![1; 34], 30, 31);
        assert_eq!(961, x.get(31));
    }

    #[test]
    fn test_first_one() {
        let mut x = MappedBitVec::new(bitvec![1; 5], 7, 1);
        x.set(2, false);
        assert_eq!(Some((0, 1)), x.first_one(0));
        assert_eq!(Some((1, 8)), x.first_one(1));
        assert_eq!(Some((3, 22)), x.first_one(2));
        assert_eq!(Some((3, 22)), x.first_one(3));
        assert_eq!(Some((4, 29)), x.first_one(4));
        assert_eq!(None, x.first_one(5));
    }

    #[test]
    fn test_into_iter() {
        let x = MappedBitVec::new(bitvec![1; 5], 7, 1);
        let x: Vec<usize> = x.into_iter().collect();
        assert_eq!(vec![1, 8, 15, 22, 29], x);
    }
}
