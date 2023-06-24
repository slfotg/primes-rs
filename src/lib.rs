use std::{cmp::Reverse, collections::BinaryHeap};

use crate::bitvec::{MappedBitVec, MappedBitVecIterator};

pub mod bitvec;
pub mod manager;
pub mod task;

pub struct PrimeSequence {
    wheels: Vec<MappedBitVec>,
}

impl PrimeSequence {
    pub fn new(wheels: Vec<MappedBitVec>) -> PrimeSequence {
        PrimeSequence { wheels }
    }
}

impl IntoIterator for PrimeSequence {
    type Item = usize;

    type IntoIter = PrimeSequenceIterator;

    fn into_iter(self) -> Self::IntoIter {
        let iters: Vec<MappedBitVecIterator> =
            self.wheels.iter().cloned().map(|w| w.into_iter()).collect();
        PrimeSequenceIterator::new(iters)
    }
}

pub struct PrimeSequenceIterator {
    iters: Vec<MappedBitVecIterator>,
    queue: BinaryHeap<Reverse<(usize, usize)>>,
}

impl PrimeSequenceIterator {
    fn new(mut iters: Vec<MappedBitVecIterator>) -> PrimeSequenceIterator {
        let mut queue = BinaryHeap::with_capacity(iters.len());
        for (i, iter) in iters.iter_mut().enumerate() {
            if let Some(w) = iter.next() {
                queue.push(Reverse((w, i)));
            }
        }
        PrimeSequenceIterator { iters, queue }
    }
}

impl Iterator for PrimeSequenceIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let head = self.queue.pop();
        if let Some(elem) = head {
            let next = elem.0 .0;
            let index = elem.0 .1;
            let iter = &mut self.iters[index];
            if let Some(p) = iter.next() {
                self.queue.push(Reverse((p, index)));
            }
            Some(next)
        } else {
            None
        }
    }
}
