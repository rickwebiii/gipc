#![feature(async_await, futures_api)]

mod ipc;
mod windows;

#[cfg(test)]
mod test_utils;

pub use self::ipc::{
    MessageIpcClient,
    MessageIpcConnection,
    MessageIpcServer,
    RawIpcClient,
    RawIpcConnection,
    RawIpcServer,
};
