use bitvec::prelude::*;
use primes_rs::{MappedBitVec, PrimeSequence, WheelMapping};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let modulus = 30;
    let max = 100;
    let (tx, mut rx) = mpsc::channel(32);
    for i in [7, 11, 13, 17, 19, 23, 29, 31] {
        let tx2 = tx.clone();
        tokio::spawn(async move {
            let _ = tx2.send(get_primes(i, modulus, max)).await;
        });
    }
    let _ = tx
        .send(MappedBitVec::new(
            bitvec![0, 0, 1, 1, 0, 1],
            WheelMapping::default(),
        ))
        .await;
    drop(tx);

    let mut size = 0;
    let mut primes = Vec::with_capacity(9);
    while let Some(message) = rx.recv().await {
        //println!("GOT = {}", message);
        size += message.len();
        primes.push(message);
    }
    let seq = PrimeSequence::new(&primes[..]);
    for elem in seq.into_iter() {
        println!("{}", elem);
    }
    println!("Size = {}", size);

    let smalls = MappedBitVec::new(bitvec![0, 0, 1, 1, 0, 1], WheelMapping::default());
    for p in smalls.into_iter() {
        println!("{}", p);
    }
    println!("{}", smalls);
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

fn get_primes(start: u32, modulus: u32, max: u32) -> MappedBitVec {
    let size = (max / modulus) - (start / modulus) + u32::from((start % modulus) <= max % modulus);

    let mut vec = bitvec!(1; size as usize);
    let mapping = WheelMapping::new(start, modulus);

    for (i, mut e) in vec.iter_mut().enumerate() {
        let p = mapping.apply(i);
        if !is_prime(p) {
            e.set(false);
        }
    }
    MappedBitVec::new(vec, mapping)
}
