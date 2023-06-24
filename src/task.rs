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
        resp: oneshot::Sender<MappedBitVec>,
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
    debug!("Calculating next prime");
    let _ = resp.send(if let Some((next_index, p)) = wheel.first_one(from_index) {
        Some((p, next_index, thread_id))
    } else {
        None
    });
}

pub async fn manage_wheel(
    mut rx: mpsc::Receiver<Command>,
    thread_id: usize,
    offset: usize,
    modulus: usize,
    max: usize,
) {
    let mut wheel = MappedBitVec::new(initialize_vec(offset, modulus, max), modulus, offset);
    while let Some(command) = rx.recv().await {
        match command {
            Command::NextPrimeFrom { from_index, resp } => {
                calculate_next_prime(thread_id, &wheel, from_index, resp)
            }
            Command::Sieve { prime, composite } => {
                debug!("Sieving from {}", composite);
                let p_index = (composite / modulus) - (offset / modulus);
                for i in (p_index..wheel.max_len()).step_by(prime) {
                    wheel.set(i, false);
                }
            }
            Command::Break { resp } => {
                debug!("Returning wheel");
                let _ = resp.send(wheel);
                break;
            }
        }
    }
    debug!("Finished thread {}", thread_id);
}
