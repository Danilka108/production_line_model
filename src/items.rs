use std::{ops::ControlFlow, sync::mpsc};

use rand::{prelude::Distribution, rngs::ThreadRng, thread_rng};

use crate::{exp_distribution::ExpDistribution, into_control_flow::IntoControlFlow};

pub struct Item {
    t: f64,
}

pub(crate) struct ItemsBuilder {
    items_max_count: usize,
    incoming_items_rx: Option<mpsc::Receiver<Item>>,
    moving_items_tx: Option<mpsc::Sender<Item>>,
    handling_time_distr: ExpDistribution,
    idle_time_distr: ExpDistribution,
}

impl ItemsBuilder {
    pub fn new(
        items_max_count: usize,
        handling_time_distr: ExpDistribution,
        idle_time_distr: ExpDistribution,
    ) -> Self {
        Self {
            items_max_count,
            handling_time_distr,
            idle_time_distr,
            incoming_items_rx: None,
            moving_items_tx: None,
        }
    }

    pub fn incoming_items_rx(mut self, value: mpsc::Receiver<Item>) -> Self {
        self.incoming_items_rx = Some(value);
        self
    }

    pub fn moving_items_tx(mut self, value: Option<mpsc::Sender<Item>>) -> Self {
        self.moving_items_tx = value;
        self
    }

    pub fn build(self) -> Items {
        Items {
            items_max_count: self.items_max_count,
            incoming_items_rx: self.incoming_items_rx.unwrap(),
            moving_items_tx: self.moving_items_tx,
            items: Vec::with_capacity(self.items_max_count),
            handling_time_distr: self.handling_time_distr,
            idle_time_distr: self.idle_time_distr,
            total_idle_time: 0.0,
            rand_gen: thread_rng(),
        }
    }
}

pub(crate) struct Items {
    items_max_count: usize,
    incoming_items_rx: mpsc::Receiver<Item>,
    moving_items_tx: Option<mpsc::Sender<Item>>,
    items: Vec<Item>,
    handling_time_distr: ExpDistribution,
    idle_time_distr: ExpDistribution,
    total_idle_time: f64,
    rand_gen: ThreadRng,
}

impl Items {
    pub fn is_full(&self) -> bool {
        self.items.len() == self.items_max_count
    }

    pub fn add_incoming_items(&mut self) -> ControlFlow<(), ()> {
        while let Some(item_to_add) = self.incoming_items_rx.try_recv().into_control_flow()? {
            self.items.push(item_to_add);
        }

        ControlFlow::Continue(())
    }

    pub fn handle_item(&mut self) -> ControlFlow<(), ()> {
        if let Some(mut item) = self.items.pop() {
            let handling_time = self.handling_time_distr.sample(&mut self.rand_gen);
            let idle_time = self.idle_time_distr.sample(&mut self.rand_gen);

            item.t += handling_time + idle_time + self.total_idle_time;

            if let Some(ref moving_items_tx) = self.moving_items_tx {
                moving_items_tx.send(item).into_control_flow()?;
            }

            self.total_idle_time += handling_time + idle_time;
        } else {
            self.total_idle_time = 0.0;
        }

        ControlFlow::Continue(())
    }
}
