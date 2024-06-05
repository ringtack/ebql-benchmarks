use std::sync::{
    atomic::{AtomicBool, Ordering::SeqCst},
    Arc,
};

use anyhow::{bail, Result};
use crossbeam::channel::Sender;
use log::LevelFilter;

use crate::bpf_structs::FromBytes;

pub fn init_log(level: LevelFilter) {
    log::set_max_level(level);
    env_logger::builder().filter(None, level).init();
}

pub fn bump_memlock_rlimit() -> Result<()> {
    let rlimit = libc::rlimit {
        rlim_cur: 128 << 20,
        rlim_max: 128 << 20,
    };
    if unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlimit) } != 0 {
        bail!("Failed to increase rlimit");
    }
    Ok(())
}

pub fn create_event_handler<T>(
    tx: Sender<Vec<T>>,
    done: Arc<AtomicBool>,
) -> impl FnMut(&[u8]) -> i32
where
    T: FromBytes,
{
    let tx = tx.clone();
    let callback = move |buf: &[u8]| -> i32 {
        if done.load(SeqCst) {
            return 1;
        }
        if buf.len() % std::mem::size_of::<T>() != 0 {
            eprintln!(
                "Received record size ({}) that does not evenly divide record size ({})",
                buf.len(),
                std::mem::size_of::<T>(),
            );
            return 1;
        }

        let records = buf
            .chunks(std::mem::size_of::<T>())
            .map(|buf| T::from_bytes(buf))
            .collect::<Vec<_>>();

        if let Err(e) = tx.send(records) {
            println!("got error: {e}");
            return 1;
        }
        return 0;
    };
    callback
}
