use std::ops::ControlFlow;

use rand::rngs::ThreadRng;

use crate::blockings::{WorkerBlocker, WorkerBlockingWaiter};
use crate::items::Items;

pub(crate) struct Worker {
    prev_worker_blocker: WorkerBlocker,
    blocking_waiter: Option<WorkerBlockingWaiter>,
    items: Items<ThreadRng>,
    should_be_stopped: bool,
}

impl Worker {
    pub fn new(
        prev_worker_blocker: WorkerBlocker,
        blocking_waiter: Option<WorkerBlockingWaiter>,
        items: Items<ThreadRng>,
    ) -> Self {
        Self {
            prev_worker_blocker,
            blocking_waiter,
            items,
            should_be_stopped: false,
        }
    }

    pub fn work(self) {
        self.start_loop();
    }

    fn start_loop(mut self) -> ControlFlow<()> {
        loop {
            if let Some(waiter) = self.blocking_waiter {
                waiter.wait()?
            };

            self.items.add_incoming_items()?;

            let _guard = self.prev_worker_blocker.block_if(self.items.is_full())?;
            self.items.handle_item()?;
        }
    }
}
