use bitvec::{prelude::*, slice::IterOnes};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let modulus = 30;
    let max = 10000000;
    let (tx, mut rx) = mpsc::channel(32);
    for i in [7, 11, 13, 17, 19, 23, 29, 31] {
        let tx2 = tx.clone();
        tokio::spawn(async move {
            let _ = tx2.send(get_primes(i, modulus, max)).await;
        });
    }
    let _ = tx.send(vec![2, 3, 5]).await;
    drop(tx);

    let mut size = 0;
    while let Some(message) = rx.recv().await {
        //println!("GOT = {:?}", message);
        size += message.len();
    }
    println!("Size = {}", size);

    let vec = bitvec!(1; 30);

    let seq = PrimeSequence {
        vec,
        modulus: 6,
        start: 5,
    };

    for p in seq.iter() {
        println!("{}", p);
    }
}

struct PrimeSequence {
    vec: BitVec,
    modulus: u32,
    start: u32,
}

impl PrimeSequence {
    fn iter(&self) -> PrimeSequenceIterator<'_> {
        PrimeSequenceIterator {
            seq: self,
            iter: self.vec.iter_ones(),
        }
    }
}

struct PrimeSequenceIterator<'a> {
    seq: &'a PrimeSequence,
    iter: IterOnes<'a, usize, Lsb0>,
}

impl Iterator for PrimeSequenceIterator<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|i| (i as u32) * self.seq.modulus + self.seq.start)
    }
}

fn is_prime(p: u32) -> bool {
    let root = (p as f64).sqrt() as u32;
    for n in 2..(root + 1) {
        if p % n == 0 {
            return false;
        }
    }
    true
}

fn get_primes(start: u32, delta: u32, max: u32) -> Vec<u32> {
    let size = (max / delta) - (start / delta) + u32::from((start % delta) <= max % delta);

    let mut vec = bitvec!(1; size as usize);

    for (i, mut e) in vec.iter_mut().enumerate() {
        let p = (i as u32) * delta + start;
        if !is_prime(p) {
            e.set(false);
        }
    }
    vec.iter_ones()
        .map(|i| (i as u32) * delta + start)
        .collect()
}
