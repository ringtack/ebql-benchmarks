mod pread_query {
    include!(concat!(env!("OUT_DIR"), "/pread_query.skel.rs"));
}

use std::{
    fmt::Display,
    mem,
    sync::atomic::{AtomicBool, Ordering::SeqCst},
    thread,
    time::{Duration, Instant},
};

use anyhow::{bail, Result};
use crossbeam::channel::{self, Sender};
use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder},
    RingBufferBuilder,
};
use pread_query::*;

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
struct Record {
    fd: u64,
    cpu: u64,
    count: u64,
    sum_count: u64,
    avg_count: u64,
}

impl Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Record({}, {}, {}, {}, {})",
            self.fd, self.cpu, self.count, self.sum_count, self.avg_count
        )
    }
}

const RECORD_SIZE: usize = mem::size_of::<Record>();

static DONE: AtomicBool = AtomicBool::new(false);

fn init_signal() {
    ctrlc::set_handler(move || {
        DONE.store(true, SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
}

fn bump_memlock_rlimit() -> Result<()> {
    let rlimit = libc::rlimit {
        rlim_cur: 128 << 20,
        rlim_max: 128 << 20,
    };
    if unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlimit) } != 0 {
        bail!("Failed to increase rlimit");
    }
    Ok(())
}

fn create_event_handler(tx: Sender<Vec<Record>>) -> impl FnMut(&[u8]) -> i32 {
    let tx = tx.clone();
    let callback = move |buf: &[u8]| -> i32 {
        if DONE.load(SeqCst) {
            return 1;
        }
        if buf.len() % RECORD_SIZE != 0 {
            log::error!(
                "Received record size ({}) that does not evenly divide record size ({})",
                buf.len(),
                RECORD_SIZE
            );
            return 1;
        }

        let records = buf
            .chunks(RECORD_SIZE)
            .map(|buf| unsafe {
                mem::transmute::<[u8; RECORD_SIZE], Record>(buf.try_into().unwrap())
            })
            .collect::<Vec<_>>();

        if tx.send(records).is_err() {
            return 1;
        }
        return 0;
    };
    callback
}

fn main() {
    init_signal();
    bump_memlock_rlimit().unwrap();

    log::set_max_level(log::LevelFilter::Trace);
    env_logger::builder()
        .filter(None, log::LevelFilter::Trace)
        .init();

    let skel_builder = PreadQuerySkelBuilder::default();
    let open_skel = skel_builder.open().unwrap();
    let mut skel = open_skel.load().unwrap();

    // Create channel to receive records
    let (tx, rx) = channel::bounded(1024);
    let process_records = create_event_handler(tx);

    // Create rb handler
    let maps = skel.maps();
    let mut builder = RingBufferBuilder::new();
    builder
        .add(&maps.ring_buf_pread_query(), process_records)
        .unwrap();
    let rb = builder.build().unwrap();

    // Attach program to event
    let _link = skel.progs_mut().pread_query().attach().unwrap();

    // Spawn thread to continuously poll
    thread::spawn(move || while rb.poll(Duration::MAX).is_ok() {});

    let mut n_records = 0;
    let now = Instant::now();
    loop {
        if DONE.load(SeqCst) {
            break;
        }
        if let Ok(records) = rx.recv_timeout(Duration::from_millis(100)) {
            println!("num records: {}", records.len());
            n_records += records.len();
            // println!("RecordBatch(");
            // for record in records {
            //     println!("\t{record}");
            // }
            // println!(")");
        }
    }
    println!(
        "Stopped probing. Records: {}\tTime elapsed: {:?}",
        n_records,
        now.elapsed()
    );
}
