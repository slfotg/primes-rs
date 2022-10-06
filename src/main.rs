use bitvec::prelude::*;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let modulus = 10;
    let max = 1000;
    let (tx, mut rx) = mpsc::channel(32);
    for i in [2, 3, 5, 7, 9, 11] {
        let tx2 = tx.clone();
        tokio::spawn(async move {
            let _ = tx2.send(get_primes(i, modulus, max)).await;
        });
    }
    drop(tx);

    while (rx.recv().await).is_some() {
        //println!("GOT = {}", message);
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

fn get_primes(start: u32, delta: u32, max: u32) -> BitVec {
    let size = (max / delta) - (start / delta);
    let mut vec = bitvec!(1; size as usize);
    for (i, mut e) in vec.iter_mut().enumerate() {
        let p = (i as u32) * delta + start;
        if !is_prime(p) {
            e.set(false);
        }
    }
    let t: Vec<u32> = vec
        .iter_ones()
        .map(|i| (i as u32) * delta + start)
        .collect();
    println!("{:?}", t);
    vec
}
