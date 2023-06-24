use std::{cmp::Reverse, collections::BinaryHeap};

use tokio::sync::{mpsc, oneshot};

use crate::task::{Command, PrimeInfo};

type PriorityQueue = BinaryHeap<Reverse<PrimeInfo>>;

pub struct SpokeManager {
    sender: mpsc::Sender<Command>,
}

impl SpokeManager {
    pub fn new(sender: mpsc::Sender<Command>) -> Self {
        Self { sender }
    }
}

impl SpokeManager {
    pub async fn next_prime_from(&self, from_index: usize) -> Option<PrimeInfo> {
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

    pub async fn pop_next_prime(
        managers: &[SpokeManager],
        min_queue: &mut PriorityQueue,
        max: usize,
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

    pub async fn sieve(&self, prime: usize, composite: usize) {
        let _ = self.sender.send(Command::Sieve { prime, composite }).await;
    }

    pub async fn get_spoke(&self) -> Result<Vec<usize>, tokio::sync::oneshot::error::RecvError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.sender.send(Command::Break { resp: tx }).await;
        rx.await
    }
}
