#![feature(async_await, await_macro, futures_api)]

mod ipc;
mod windows;

pub use self::ipc::{
    RawIpcClient,
    RawIpcConnection,
    RawIpcServer,
};
