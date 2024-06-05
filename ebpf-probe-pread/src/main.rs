mod pread_query {
    pub mod ebql {
        include!(concat!(env!("OUT_DIR"), "/ebql_pread_query.skel.rs"));
    }

    pub mod opt {
        include!(concat!(env!("OUT_DIR"), "/opt_pread_query.skel.rs"));
    }

    pub mod unopt {
        include!(concat!(env!("OUT_DIR"), "/unopt_pread_query.skel.rs"));
    }
}

use std::{
    collections::{btree_map::Entry, BTreeMap},
    fs,
    io::Write,
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use clap::Parser;
use common::{
    bpf_prog, bpf_stats,
    bpf_structs::{PreadQueryRecord, RawPreadRecord},
    prog_stats::ProgStats,
};
use crossbeam::channel;
use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder},
    RingBufferBuilder,
};
use pread_query::*;

// static DONE: AtomicBool = AtomicBool::new(false);

fn init_signal(done: Arc<AtomicBool>) {
    ctrlc::set_handler(move || {
        done.store(true, SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    probe_type: String,

    #[arg(short, long, default_value_t=String::from(""))]
    stats_path: String,
}

fn main() {
    let args = Args::parse();
    if args.probe_type.is_empty() {
        panic!("Must provide probe type!");
    }
    let done = Arc::new(AtomicBool::new(false));
    init_signal(done.clone());
    bpf_prog::bump_memlock_rlimit().unwrap();
    bpf_prog::init_log(log::LevelFilter::Trace);
    bpf_stats::enable_bpf_stats().unwrap();

    // Create channel to receive records
    let (tx, rx) = channel::bounded(1024);
    let process_records = bpf_prog::create_event_handler::<PreadQueryRecord>(tx, done.clone());
    let (tx1, rx1) = channel::bounded(1024);
    let process_records_raw = bpf_prog::create_event_handler::<RawPreadRecord>(tx1, done.clone());

    let probe_type = args.probe_type.to_lowercase();
    // Depending on probe type, build different programs
    let (rb, _link) = match probe_type.as_str() {
        "ebql" => {
            log::info!("starting ebql probe");
            let mut skel = ebql::PreadQuerySkelBuilder::default()
                .open()
                .unwrap()
                .load()
                .unwrap();
            // Create rb handler
            let maps = skel.maps();
            let mut builder = RingBufferBuilder::new();
            builder
                .add(&maps.ring_buf_pread_query(), process_records)
                .unwrap();
            let rb = builder.build().unwrap();

            // Attach program to event
            let link = skel.progs_mut().pread_query().attach().unwrap();

            (rb, link)
        }
        "opt" => {
            log::info!("starting opt probe");
            let mut skel = opt::PreadQueryNextSkelBuilder::default()
                // let mut skel = opt::PreadQuerySkelBuilder::default()
                .open()
                .unwrap()
                .load()
                .unwrap();
            // Create rb handler
            let maps = skel.maps();
            let mut builder = RingBufferBuilder::new();
            builder
                .add(&maps.ring_buf_pread_query(), process_records)
                .unwrap();
            let rb = builder.build().unwrap();
            // Attach program to event
            let link = skel.progs_mut().pread_query().attach().unwrap();

            (rb, link)
        }
        "unopt" => {
            log::info!("starting unopt probe");
            let mut skel = unopt::PreadQuerySkelBuilder::default()
                .open()
                .unwrap()
                .load()
                .unwrap();
            // Create rb handler
            let maps = skel.maps();
            let mut builder = RingBufferBuilder::new();
            builder
                .add(&maps.ring_buf_pread_query(), process_records_raw)
                .unwrap();
            let rb = builder.build().unwrap();
            // Attach program to event
            let link = skel.progs_mut().pread_query().attach().unwrap();

            (rb, link)
        }
        _ => panic!("Probe type {} not supported", probe_type),
    };

    // Spawn thread to continuously poll
    thread::spawn(move || while rb.poll(Duration::MAX).is_ok() {});

    let mut n_records = 0;
    let now = Instant::now();

    match probe_type.as_str() {
        "opt" | "ebql" => {
            loop {
                if done.load(SeqCst) {
                    break;
                }
                if let Ok(records) = rx.recv_timeout(Duration::from_millis(100)) {
                    println!("num records: {}", records.len());
                    n_records += records.len();
                }
            }
        }
        "unopt" => {
            let mut total_records = 0;
            let mut last_seen = 0;
            let sec_ns = Duration::from_secs(1).as_nanos() as u64;
            // Map of (fd, cpu) -> (count, max, avg (repr as sum))
            let mut aggs = BTreeMap::new();
            loop {
                if done.load(SeqCst) {
                    break;
                }
                if let Ok(records) = rx1.recv_timeout(Duration::from_millis(100)) {
                    total_records += records.len();
                    let mut seen = 0;
                    for r in records {
                        let agg = match aggs.entry((r.fd, r.cpu)) {
                            Entry::Occupied(v) => v.into_mut(),
                            Entry::Vacant(v) => v.insert((0, 0, 0)),
                        };
                        agg.0 += 1;
                        if r.count > agg.1 {
                            agg.1 = r.count;
                        }
                        agg.2 += r.count;
                        seen = r.time;
                    }
                    if last_seen == 0 {
                        last_seen = seen;
                    }
                    if seen - last_seen > sec_ns {
                        last_seen = seen;
                        n_records += aggs.len();
                        aggs.clear();
                    }
                }
            }

            println!("Got {} total records", total_records);
        }
        _ => panic!("Probe type {} not supported", probe_type),
    };
    println!(
        "Stopped probing. Records: {}\tTime elapsed: {:?}",
        n_records,
        now.elapsed()
    );

    let stat = ProgStats::get();
    println!(
        "utime: {}\tstime: {}\tticks per second: {}\tProgram runtime: {}",
        stat.utime, stat.stime, stat.clock_tps, stat.runtime,
    );

    // Get BPF stats
    let progs = bpf_stats::get_bpf_stats();
    if !progs.is_empty() {
        for prog in &progs {
            println!(
                "runtime ns: {}\trun count: {}\trecursion misses: {}",
                prog.run_time_ns, prog.run_cnt, prog.recursion_misses
            );
        }
    }

    if !args.stats_path.is_empty() {
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(args.stats_path.clone())
            .unwrap();
        write!(f, "{}\n", stat).unwrap();

        if !progs.is_empty() {
            // Write bpf stats to separate file
            let bpf_path = format!("{}-bpf", args.stats_path);
            f = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(bpf_path)
                .unwrap();
            write!(f, "{}, {}\n", progs[0].run_time_ns, progs[0].run_cnt).unwrap();
        }
    }
}
