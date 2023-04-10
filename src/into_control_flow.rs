use std::{ops::ControlFlow, sync::mpsc::{TryRecvError, RecvError, SendError}};

pub(crate) trait IntoControlFlow<B, C> {
    fn into_control_flow(self) -> ControlFlow<B, C>;
}

impl<T> IntoControlFlow<(), Option<T>> for Result<T, TryRecvError> {
    fn into_control_flow(self) -> ControlFlow<(), Option<T>> {
        match self {
            Ok(v) => ControlFlow::Continue(Some(v)),
            Err(TryRecvError::Empty) => ControlFlow::Continue(None),
            Err(TryRecvError::Disconnected) => ControlFlow::Break(()),
        }
    }
}

impl<T> IntoControlFlow<(), T> for Result<T, RecvError> {
    fn into_control_flow(self) -> ControlFlow<(), T> {
        match self {
            Ok(v) => ControlFlow::Continue(v),
            Err(_) => ControlFlow::Break(()),
        }
    }
}

impl<T> IntoControlFlow<(), ()> for Result<(), SendError<T>> {
    fn into_control_flow(self) -> ControlFlow<(), ()> {
        match self {
            Ok(_) => ControlFlow::Continue(()),
            Err(_) => ControlFlow::Break(()),
        }
    }
}
