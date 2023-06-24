use bitvec::prelude::*;
use log::debug;
use tokio::sync::{mpsc, oneshot};

use crate::bitvec::MappedBitVec;

pub type PrimeInfo = (usize, usize, usize);

pub enum Command {
    NextPrimeFrom {
        from_index: usize,
        resp: oneshot::Sender<Option<PrimeInfo>>,
    },
    Sieve {
        prime: usize,
        composite: usize,
    },
    Break {
        resp: oneshot::Sender<Vec<usize>>,
    },
}

fn initialize_vec(offset: usize, modulus: usize, max: usize) -> BitVec {
    let size =
        (max / modulus) - (offset / modulus) + usize::from((offset % modulus) <= max % modulus);
    bitvec![1; size]
}

fn calculate_next_prime(
    thread_id: usize,
    wheel: &MappedBitVec,
    from_index: usize,
    resp: tokio::sync::oneshot::Sender<Option<(usize, usize, usize)>>,
) {
    let _ = resp.send(if let Some((next_index, p)) = wheel.first_one(from_index) {
        Some((p, next_index, thread_id))
    } else {
        None
    });
}

fn sieve(prime: usize, composite: usize, modulus: usize, sub: usize, wheel: &mut MappedBitVec) {
    let p_index = (composite / modulus) - sub;
    for i in (p_index..wheel.max_len()).step_by(prime) {
        wheel.set(i, false);
    }
}

pub async fn manage_wheel(
    mut rx: mpsc::Receiver<Command>,
    thread_id: usize,
    offset: usize,
    modulus: usize,
    max: usize,
) {
    let sub = offset / modulus;
    let mut wheel = MappedBitVec::new(initialize_vec(offset, modulus, max), modulus, offset);
    while let Some(command) = rx.recv().await {
        match command {
            Command::NextPrimeFrom { from_index, resp } => {
                debug!("Calculating next prime");
                calculate_next_prime(thread_id, &wheel, from_index, resp);
            }
            Command::Sieve { prime, composite } => {
                debug!("Sieving from {}", composite);
                sieve(prime, composite, modulus, sub, &mut wheel);
            }
            Command::Break { resp } => {
                debug!("Returning wheel");
                let _ = resp.send(wheel.to_vec());
                break;
            }
        }
    }
    debug!("Finished thread {}", thread_id);
}
