use std::{cmp::Reverse, collections::BinaryHeap};

use log::debug;
use primes_rs::{manager::SpokeManager, task};
use tokio::sync::mpsc;

const MOD: usize = 30;
const SMALL_PRIMES: [usize; 3] = [2, 3, 5];
const RELATIVE_PRIMES_SIZE: usize = 8;
const RELATIVE_PRIMES: [usize; RELATIVE_PRIMES_SIZE] = [7, 11, 13, 17, 19, 23, 29, 31];
const DIFFS: [[usize; RELATIVE_PRIMES_SIZE]; RELATIVE_PRIMES_SIZE] = [
    [0, 4, 2, 4, 2, 4, 6, 2],
    [0, 2, 4, 2, 4, 6, 2, 6],
    [0, 4, 2, 4, 6, 2, 6, 4],
    [0, 2, 4, 6, 2, 6, 4, 2],
    [0, 4, 6, 2, 6, 4, 2, 4],
    [0, 6, 2, 6, 4, 2, 4, 2],
    [0, 2, 6, 4, 2, 4, 2, 4],
    [0, 6, 4, 2, 4, 2, 4, 6],
];
const MULTIPLICATION_TABLE: [[usize; RELATIVE_PRIMES_SIZE]; RELATIVE_PRIMES_SIZE] = [
    [4, 3, 7, 6, 2, 1, 5, 0],
    [7, 5, 0, 6, 2, 4, 1, 3],
    [4, 1, 0, 6, 3, 2, 7, 5],
    [4, 5, 7, 2, 3, 6, 0, 1],
    [7, 3, 1, 4, 2, 6, 0, 5],
    [4, 0, 5, 1, 2, 6, 7, 3],
    [7, 6, 5, 4, 3, 2, 1, 0],
    [7, 0, 1, 2, 3, 4, 5, 6],
];

fn sieve(max: usize) -> Vec<usize> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (mut len, primes_vec) = rt.block_on(async {
        let managers = RELATIVE_PRIMES
            .into_iter()
            .enumerate()
            .map(|(i, p)| {
                let (ctx, crx) = mpsc::channel(32);
                tokio::spawn(async move {
                    task::manage_wheel(crx, i, p, MOD, max).await;
                });
                SpokeManager::new(ctx)
            })
            .collect::<Vec<_>>();

        // Initialze min BinaryHeap
        let mut min_queue = BinaryHeap::with_capacity(RELATIVE_PRIMES_SIZE);
        for manager in managers.iter() {
            if let Some(elem) = manager.next_prime_from(0).await {
                let p = elem.0;
                if p * p < max {
                    min_queue.push(Reverse(elem));
                }
            }
        }
        debug!("Queue: {:?}", min_queue);

        // Do the actually sieve
        while !min_queue.is_empty() {
            if let Some((prime, _, spoke_index)) =
                SpokeManager::pop_next_prime(&managers, &mut min_queue, max).await
            {
                let diffs = &DIFFS[spoke_index];
                let table = &MULTIPLICATION_TABLE[spoke_index];

                let mut relative_prime = prime;
                for (i, &diff) in diffs.iter().enumerate() {
                    relative_prime += diff;
                    let composite = prime * relative_prime;
                    if composite <= max {
                        let manager = &managers[table[i]];
                        manager.sieve(prime, composite).await;
                    } else {
                        break;
                    }
                }
            }
        }
        let mut primes = Vec::with_capacity(managers.len());
        let mut len = 0;
        for manager in managers.iter() {
            if let Ok(spoke) = manager.get_spoke().await {
                len += spoke.len();
                primes.push(spoke);
            }
        }
        (len, primes)
    });
    len += SMALL_PRIMES.len();
    println!("{len}");
    let mut primes: Vec<usize> = vec![0; len];
    let mut index = 0;
    primes[index..SMALL_PRIMES.len()].copy_from_slice(&SMALL_PRIMES);
    index += SMALL_PRIMES.len();
    for v in primes_vec {
        primes[index..index + v.len()].copy_from_slice(&v);
        index += v.len();
    }
    primes.sort();
    primes
}

fn main() {
    env_logger::init();
    let primes: Vec<usize> = sieve(10000000000);
    println!("{:?}", primes.len());
}
