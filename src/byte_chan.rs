use std::sync::mpsc::{Receiver, Sender};

pub fn bounded(max: usize) -> (ByteTx, ByteRx) {
    todo!()
}

pub struct ByteTx {
    tx: Sender<Box<[u8]>>,
}

pub struct ByteRx {
    tx: Receiver<Box<[u8]>>,
}
