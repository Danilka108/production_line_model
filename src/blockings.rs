use std::{ops::ControlFlow, sync::mpsc};

use crate::into_control_flow::IntoControlFlow;

pub(crate) fn blocking() -> (WorkerBlockingWaiter, WorkerBlocker) {
    let (blocking_sender, blocking_rec) = mpsc::channel();
    let (unblocking_sender, unblocking_rec) = mpsc::channel();

    (
        WorkerBlockingWaiter {
            blocking_rec,
            unblocking_rec,
        },
        WorkerBlocker {
            blocking_sender,
            unblocking_sender,
        },
    )
}

pub(crate) struct WorkerBlockingWaiter {
    blocking_rec: mpsc::Receiver<()>,
    unblocking_rec: mpsc::Receiver<()>,
}

impl WorkerBlockingWaiter {
    pub fn wait(&self) -> ControlFlow<(), ()> {
        self.blocking_rec.try_recv().into_control_flow()?;
        self.unblocking_rec.recv().into_control_flow()
    }
}

pub(crate) struct WorkerBlocker {
    blocking_sender: mpsc::Sender<()>,
    unblocking_sender: mpsc::Sender<()>,
}

pub(crate) struct BlockerGuard<'blocker> {
    blocker: &'blocker mut WorkerBlocker,
}

impl WorkerBlocker {
    pub fn block_if<'blocker>(
        &'blocker mut self,
        should_be_blocked: bool,
    ) -> ControlFlow<(), Option<BlockerGuard<'blocker>>> {
        ControlFlow::Continue(if should_be_blocked {
            self.blocking_sender.send(()).into_control_flow()?;
            Some(BlockerGuard { blocker: self })
        } else {
            None
        })
    }
}

impl<'blocker> Drop for BlockerGuard<'blocker> {
    fn drop(&mut self) {
        let _ = self.blocker.unblocking_sender.send(());
    }
}
