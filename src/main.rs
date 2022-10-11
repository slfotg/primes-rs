use std::{cmp::Reverse, collections::BinaryHeap};

use bitvec::prelude::*;
use log::debug;
use primes_rs::{MappedBitVec, WheelMapping};
use tokio::sync::{mpsc, oneshot};

const MOD: u64 = 30;
const RELATIVE_PRIMES_SIZE: usize = 8;
const RELATIVE_PRIMES: [u64; RELATIVE_PRIMES_SIZE] = [7, 11, 13, 17, 19, 23, 29, 31];
const DIFFS: [[u64; RELATIVE_PRIMES_SIZE]; RELATIVE_PRIMES_SIZE] = [
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

type PrimeInfo = (u64, usize, usize);
type PriorityQueue = BinaryHeap<Reverse<PrimeInfo>>;

enum Command {
    NextPrimeFrom {
        from_index: usize,
        resp: oneshot::Sender<Option<PrimeInfo>>,
    },
    Sieve {
        prime: u64,
        composite: u64,
    },
    Break {
        resp: oneshot::Sender<MappedBitVec>,
    },
}

struct SpokeManager {
    sender: mpsc::Sender<Command>,
}

impl SpokeManager {
    async fn next_prime_from(&self, from_index: usize) -> Option<PrimeInfo> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(Command::NextPrimeFrom {
                from_index,
                resp: tx,
            })
            .await;
        if let Ok(result) = rx.await {
            result
        } else {
            None
        }
    }

    async fn pop_next_prime(
        managers: &[SpokeManager],
        min_queue: &mut PriorityQueue,
        max: u64,
    ) -> Option<PrimeInfo> {
        loop {
            if let Some(elem) = min_queue.pop() {
                let elem = elem.0;
                let (_, prime_index, spoke_index) = elem;

                if let Some(current_elem) = managers[spoke_index].next_prime_from(prime_index).await
                {
                    if elem == current_elem {
                        if let Some(next_elem) =
                            managers[spoke_index].next_prime_from(prime_index + 1).await
                        {
                            let p = next_elem.0;
                            if p * p < max {
                                min_queue.push(Reverse(next_elem));
                            }
                        }
                        return Some(elem);
                    } else {
                        min_queue.push(Reverse(current_elem));
                    }
                }
            } else {
                return None;
            }
        }
    }

    async fn sieve(&self, prime: u64, composite: u64) {
        let _ = self.sender.send(Command::Sieve { prime, composite }).await;
    }

    async fn get_spoke(&self) -> Result<MappedBitVec, tokio::sync::oneshot::error::RecvError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.sender.send(Command::Break { resp: tx }).await;
        rx.await
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let max = 1_000_000_000;

    let mut managers = Vec::with_capacity(RELATIVE_PRIMES_SIZE);

    for (i, &p) in RELATIVE_PRIMES.iter().enumerate() {
        let (ctx, crx) = mpsc::channel(32);
        managers.push(SpokeManager { sender: ctx });
        tokio::spawn(async move {
            manage_wheel(crx, i, p, MOD, max).await;
        });
    }

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
    let mut size = 3;
    for manager in managers.iter() {
        if let Ok(spoke) = manager.get_spoke().await {
            size += spoke.len();
        }
    }
    println!("Size: {}", size);
}

async fn manage_wheel(
    mut rx: mpsc::Receiver<Command>,
    index: usize,
    offset: u64,
    modulus: u64,
    max: u64,
) {
    let mut wheel = initialize_vec(offset, modulus, max);
    let mapping = WheelMapping::new(offset, modulus);
    while let Some(command) = rx.recv().await {
        match command {
            Command::NextPrimeFrom { from_index, resp } => {
                debug!("Calculating next prime");
                let mut next_index = from_index;
                let _ = resp.send(if let Some(first_one) = &wheel[next_index..].first_one() {
                    let p = mapping.apply(next_index + first_one);
                    next_index += first_one;
                    Some((p, next_index, index))
                } else {
                    None
                });
            }
            Command::Sieve { prime, composite } => {
                debug!("Sieving from {}", composite);
                let p_index = ((composite / modulus) - (offset / modulus)) as usize;
                for i in (p_index..wheel.len()).step_by(prime as usize) {
                    wheel.set(i, false);
                }
            }
            Command::Break { resp } => {
                debug!("Returning wheel");
                let _ = resp.send(MappedBitVec::new(wheel, mapping));
                break;
            }
        }
    }
    debug!("Finished thread {}", index);
}

fn initialize_vec(offset: u64, modulus: u64, max: u64) -> BitVec {
    let size =
        (max / modulus) - (offset / modulus) + u64::from((offset % modulus) <= max % modulus);
    bitvec!(1; size as usize)
}
