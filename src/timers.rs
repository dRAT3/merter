use std::sync::atomic::{AtomicUsize, Ordering};

static SLEEPTIMEJSONRPC: AtomicUsize = AtomicUsize::new(0);

pub fn push_time_rpc(time: usize) {
    SLEEPTIMEJSONRPC.fetch_add(time, Ordering::SeqCst);
}

pub fn get_sleep_time_rpc() -> usize {
    SLEEPTIMEJSONRPC.load(Ordering::SeqCst)
}

pub fn count_down_rpc() {
    SLEEPTIMEJSONRPC.store(0, Ordering::SeqCst);
    let milli = std::time::Duration::from_millis(1);
    loop {
        let time = get_sleep_time_rpc();
        if time.gt(&0) {
            SLEEPTIMEJSONRPC.fetch_sub(1, Ordering::SeqCst);
        }
        std::thread::sleep(milli);
    }
}
