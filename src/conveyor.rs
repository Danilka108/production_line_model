use std::{
    sync::mpsc,
    thread::{self, JoinHandle},
};

use crate::{
    blockings::{blocking, WorkerBlockingWaiter},
    exp_distribution::ExpDistribution,
    items::{Item, ItemsBuilder},
    worker::Worker,
};

pub struct Conveyor {
    threads: Vec<JoinHandle<()>>,
    first_worker_blocking_waiter: WorkerBlockingWaiter,
    first_worker_moving_items_tx: mpsc::Sender<Item>,
}

impl Conveyor {
    pub fn new(
        mut workers_max_free_spaces: Vec<usize>,
        handling_time_distr: ExpDistribution,
        idle_time_distr: ExpDistribution,
    ) -> Self {
        let mut next_moving_items_tx = None;
        let mut prev_blocking_waiter = None;

        let workers_count = workers_max_free_spaces.len();
        let mut threads = Vec::with_capacity(workers_count);

        for i in (0..workers_count).rev() {
            let (moving_items_tx, incoming_items_rx) = mpsc::channel();

            let items = ItemsBuilder::new(
                workers_max_free_spaces.pop().unwrap(),
                handling_time_distr,
                idle_time_distr,
            )
            .incoming_items_rx(incoming_items_rx)
            .moving_items_tx(next_moving_items_tx)
            .build();

            next_moving_items_tx = Some(moving_items_tx);

            let (blocking_waiter, blocker) = blocking();
            let worker = Worker::new(blocker, prev_blocking_waiter, items).work();
            prev_blocking_waiter = Some(blocking_waiter);

            threads.push(thread::spawn(move || {
                worker.work();
            }))
        }

        Self {
            threads,
            first_worker_blocking_waiter: prev_blocking_waiter.unwrap(),
            first_worker_moving_items_tx: next_moving_items_tx.unwrap(),
        }
    }
}
