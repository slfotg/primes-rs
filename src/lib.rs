pub mod bitvec;
pub mod manager;
pub mod task;

// impl<'a> IntoIterator for &'a MappedBitVec {
//     type Item = usize;

//     type IntoIter = MappedBitVecIterator<'a>;

//     fn into_iter(self) -> Self::IntoIter {
//         MappedBitVecIterator {
//             offset: 0,
//             slice: &self.vec,
//             func: &self.func,
//         }
//     }
// }

// impl fmt::Display for MappedBitVec {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let mut elements = Vec::with_capacity(self.len());
//         for elem in self.into_iter() {
//             elements.push(format!("{}", elem));
//         }
//         write!(f, "[{}]", elements.join(", "))
//     }
// }

// pub struct MappedBitVecIterator<'a> {
//     offset: usize,
//     slice: &'a BitSlice,
//     func: &'a WheelMapping,
// }

// impl Iterator for MappedBitVecIterator<'_> {
//     type Item = usize;

//     fn next(&mut self) -> Option<Self::Item> {
//         if let Some(index) = self.slice.first_one() {
//             let val = self.func.apply(index + self.offset);
//             self.offset += index + 1;
//             self.slice = &self.slice[index + 1..];
//             Some(val)
//         } else {
//             None
//         }
//     }
// }

// pub struct PrimeSequence<'a> {
//     wheels: &'a [MappedBitVec],
// }

// impl<'a> PrimeSequence<'a> {
//     pub fn new(wheels: &'a [MappedBitVec]) -> PrimeSequence<'a> {
//         PrimeSequence { wheels }
//     }
// }

// impl<'a> IntoIterator for &'a PrimeSequence<'a> {
//     type Item = usize;

//     type IntoIter = PrimeSequenceIterator<'a>;

//     fn into_iter(self) -> Self::IntoIter {
//         let iters: Vec<MappedBitVecIterator<'a>> =
//             self.wheels.iter().map(|w| w.into_iter()).collect();
//         PrimeSequenceIterator::new(iters)
//     }
// }

// pub struct PrimeSequenceIterator<'a> {
//     iters: Vec<MappedBitVecIterator<'a>>,
//     queue: BinaryHeap<Reverse<(usize, usize)>>,
// }

// impl<'a> PrimeSequenceIterator<'a> {
//     fn new(mut iters: Vec<MappedBitVecIterator<'a>>) -> PrimeSequenceIterator {
//         let mut queue = BinaryHeap::with_capacity(iters.len());
//         for (i, iter) in iters.iter_mut().enumerate() {
//             if let Some(w) = iter.next() {
//                 queue.push(Reverse((w, i)));
//             }
//         }
//         PrimeSequenceIterator { iters, queue }
//     }
// }

// impl<'a> Iterator for PrimeSequenceIterator<'a> {
//     type Item = usize;

//     fn next(&mut self) -> Option<Self::Item> {
//         let head = self.queue.pop();
//         if let Some(elem) = head {
//             let next = elem.0 .0;
//             let index = elem.0 .1;
//             let iter = &mut self.iters[index];
//             if let Some(p) = iter.next() {
//                 self.queue.push(Reverse((p, index)));
//             }
//             Some(next)
//         } else {
//             None
//         }
//     }
// }
