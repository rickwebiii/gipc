use simplelog::{Config, LevelFilter, TermLogger};

use std::sync::{Once};
use std::sync::atomic::{AtomicUsize, Ordering};

static START: Once = Once::new();
static IPC_SERVER_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn install_logger() {
    START.call_once(|| {
        TermLogger::init(LevelFilter::Debug, Config::default()).unwrap();
    });
}

pub fn get_server_name() -> String {
    format!("horsey_test_server{}", IPC_SERVER_COUNT.fetch_add(1, Ordering::SeqCst))
}