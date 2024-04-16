pub mod request_stat;

use std::{
    fs,
    io::Write,
    path::PathBuf,
    process,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use clap::Parser;
use crossbeam::channel::{bounded, Receiver, Sender};
use lazy_static::lazy_static;
use rand::prelude::*;
use request_stat::*;
use rocksdb::{DBWithThreadMode, MultiThreaded, SingleThreaded, ThreadMode, WriteOptions};

static READ_QUERIES: AtomicUsize = AtomicUsize::new(0);
static WRITE_QUERIES: AtomicUsize = AtomicUsize::new(0);
static TOTAL: AtomicUsize = AtomicUsize::new(0);
static DONE: AtomicBool = AtomicBool::new(false);

// const NUM_OPS: usize = 20_000_000;
// const DELAY_SECS: u64 = 10; // num secs to delay before starting, to warm up
// cache

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    throughput_path: String,

    #[arg(long)]
    quantiles_path: String,

    #[arg(long)]
    db_path: String,

    #[arg(short, long, default_value_t = 0.5)]
    read_percent: f64,

    #[arg(short, long, default_value_t = 8)]
    writer_threads: usize,

    #[arg(short, long, default_value_t = 512)]
    data_size: usize,

    #[arg(short, long, default_value_t = 1000000)]
    key_space: usize,

    #[arg(short, long, default_value_t = false)]
    setup_db: bool,

    #[arg(short, long, default_value_t = 20_000_000)]
    num_ops: usize,

    #[arg(long, default_value_t = 10)]
    delay_secs: u64,
}

fn init_counters(args: &Args) {
    let out_path = args.throughput_path.clone();
    thread::spawn(move || {
        let mut f = std::fs::File::create(out_path).unwrap();
        let mut counter = 0;
        write!(f, "second,reads,writes,total\n").unwrap();
        loop {
            counter += 1;
            let cur_read = READ_QUERIES.swap(0, SeqCst);
            let cur_write = WRITE_QUERIES.swap(0, SeqCst);
            let total = TOTAL.load(SeqCst);
            let s = format!("{}, {}, {}, {}", counter, cur_read, cur_write, total);
            println!("{}", s);
            write!(f, "{}\n", s).unwrap();
            thread::sleep(Duration::from_secs(1));
            if DONE.load(SeqCst) {
                break;
            }
        }
    });
}

fn init_signal() {
    ctrlc::set_handler(move || {
        DONE.store(true, SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
}

lazy_static! {
    // static ref ARGS: Args = {
    //     match Args::try_parse() {
    //         Ok(x) => x,
    //         Err(x) => {
    //             use clap::error::{DefaultFormatter, ErrorFormatter};
    //             println!("Error: {}", DefaultFormatter::format_error(&x));
    //             std::process::exit(1);
    //         }
    //     }
    // };
    static ref START_TIME: Instant = Instant::now();
}

fn new_data(args: &Args) -> Vec<u8> {
    let mut rng = thread_rng();
    let mut item = vec![0u8; args.data_size];
    for i in &mut item {
        *i = rng.gen()
    }
    item
}

fn do_work<T: ThreadMode>(
    db: Arc<DBWithThreadMode<T>>,
    keys: Vec<u64>,
    stats: Sender<Vec<RequestStat>>,
    args: Args,
) {
    let pid = process::id() as u64;
    //let pid = 0;
    let mut read_latency = Vec::new();
    let mut counter = 0;
    let mut read = 0;
    let mut write = 0;
    let mut rng = rand::thread_rng();
    let data = new_data(&args);
    loop {
        // Only perform count increases if past delay
        if START_TIME.elapsed().as_secs() >= args.delay_secs {
            let n = TOTAL.fetch_add(1, SeqCst);
            if n == 0 {}
            // Increment counter; limit to NUM_OPS runs
            if n >= args.num_ops {
                DONE.store(true, SeqCst);
            }
        }
        let key = *keys.choose(&mut rng).unwrap();
        counter += 1;
        let utc: DateTime<Utc> = Utc::now();
        let read_op = rng.gen::<f64>() < args.read_percent;
        if read_op {
            let now = Instant::now();
            let _sl = db.get(key.to_be_bytes()).unwrap().unwrap();
            let d = now.elapsed();
            //assert_eq!(sl.len(), DATA_SIZE);
            let dur = d.as_secs_f64();
            // collect only for read operations
            let stat = RequestStat::new(pid, utc.timestamp_micros() as u64, dur);
            read_latency.push(stat);
            read += 1;
        } else {
            let mut opt = WriteOptions::default();
            opt.set_sync(false);
            db.put_opt(key.to_be_bytes(), &data, &opt).unwrap();
            write += 1;
        };

        // Update global counters for printout
        if counter > 0 && counter % 100 == 0 {
            READ_QUERIES.fetch_add(read, SeqCst);
            WRITE_QUERIES.fetch_add(write, SeqCst);
            read = 0;
            write = 0;
        }

        if DONE.load(SeqCst) {
            break;
        }
    }

    // If exiting, send the read latencies off to be measured
    stats.send(read_latency).unwrap();
}

#[allow(dead_code)]
fn setup_db_multithreaded(
    keys: &[u64],
    args: &Args,
    path: PathBuf,
) -> Arc<DBWithThreadMode<MultiThreaded>> {
    let _ = std::fs::remove_dir_all(&path);
    let db = Arc::new(DBWithThreadMode::<MultiThreaded>::open_default(path).unwrap());
    let mut opt = WriteOptions::default();
    opt.set_sync(false);
    opt.disable_wal(true);
    let data = new_data(args);
    for key in keys {
        db.put_opt(key.to_be_bytes(), &data, &opt).unwrap();
        if DONE.load(SeqCst) {
            std::process::exit(1);
        }
    }

    db.flush().unwrap();
    db
}

#[allow(dead_code)]
fn setup_db_singlethreaded(
    keys: &[u64],
    args: &Args,
    path: &PathBuf,
) -> Arc<DBWithThreadMode<SingleThreaded>> {
    //let path = common::APP_ROCKSDB_DIR.as_path();
    //let path = "tmp_app_data";
    let _ = std::fs::remove_dir_all(path);
    let db = DBWithThreadMode::<SingleThreaded>::open_default(path).unwrap();
    //let mut map = HashMap::new();
    let mut opt = WriteOptions::default();
    opt.set_sync(false);
    opt.disable_wal(true);
    let data = new_data(args);
    for key in keys {
        db.put_opt(key.to_be_bytes(), &data, &opt).unwrap();
        if DONE.load(SeqCst) {
            std::process::exit(1);
        }
    }
    db.flush().unwrap();
    Arc::new(db)
}

fn open_db_singlethreaded(path: &PathBuf) -> Arc<DBWithThreadMode<SingleThreaded>> {
    let db = DBWithThreadMode::<SingleThreaded>::open_default(path).unwrap();
    let mut opt = WriteOptions::default();
    opt.set_sync(false);
    opt.disable_wal(true);
    Arc::new(db)
}

#[allow(dead_code)]
fn setup_workers_shareddb(path: PathBuf, args: Args, stats_tx: Sender<Vec<RequestStat>>) {
    let mut rng = thread_rng();
    let keys: Vec<u64> = (0..args.key_space).map(|_| rng.gen()).collect();
    println!("Setting up shared db");
    let db = setup_db_multithreaded(&keys, &args, path);
    let n_writers = args.writer_threads;
    for _ in 0..n_writers {
        let stats_tx = stats_tx.clone();
        let db = db.clone();
        let keys = keys.clone();
        let args = args.clone();
        std::thread::spawn(move || {
            do_work(db, keys, stats_tx, args);
        });
    }
}

#[allow(dead_code)]
fn setup_workers(path: PathBuf, args: Args, stats_tx: Sender<Vec<RequestStat>>) {
    let n_writers = args.writer_threads;
    for i in 0..n_writers {
        let stats_tx = stats_tx.clone();
        let path = path.clone();
        let args = args.clone();
        std::thread::spawn(move || {
            println!("Setting up db{}", i);
            let mut rng = thread_rng();
            let keys: Vec<u64> = (0..args.key_space).map(|_| rng.gen()).collect();
            let db = setup_db_singlethreaded(&keys, &args, &path.join(format!("subdir-{}", i)));
            println!("Done setting up db{}", i);
            do_work(db, keys, stats_tx, args);
        });
    }
}

fn setup_database(path: PathBuf, args: Args) {
    let mut handles = Vec::new();
    for i in 0..args.writer_threads {
        let path = path.clone();
        let args = args.clone();
        handles.push(thread::spawn(move || {
            println!("Setting up db{i}");
            let mut rng = thread_rng();
            let keys: Vec<u64> = (0..args.key_space).map(|_| rng.gen()).collect();
            // Set up database with keys
            let subdir_path = path.join(format!("subdir-{}", i));
            let _db = setup_db_singlethreaded(&keys, &args, &subdir_path);

            // Write keys to disk
            let keys_buf = keys
                .iter()
                .map(|x| x.to_be_bytes())
                .flatten()
                .collect::<Vec<_>>();
            let mut keys_f = std::fs::File::create(subdir_path.join("keys")).unwrap();
            keys_f.write(&keys_buf).unwrap();
            println!("Done setting up db{}", i);
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
}

fn setup_workers_existing_db(path: PathBuf, args: Args, stats_tx: Sender<Vec<RequestStat>>) {
    for i in 0..args.writer_threads {
        let stats_tx = stats_tx.clone();
        let path = path.clone();
        let args = args.clone();
        std::thread::spawn(move || {
            println!("Opening db{}", i);

            // Open existing DB
            let subdir_path = &path.join(format!("subdir-{}", i));
            let db = open_db_singlethreaded(subdir_path);

            // Get existing keys
            let keys_path = subdir_path.join("keys");
            let keys_bytes = fs::read(keys_path).unwrap();
            let keys = keys_bytes
                .chunks(std::mem::size_of::<u64>())
                .map(|b| u64::from_be_bytes(b.try_into().unwrap()))
                .collect::<Vec<u64>>();

            println!("Done setting up db{}", i);
            do_work(db, keys, stats_tx, args);
        });
    }
}

fn gather_stats(stats_rx: Receiver<Vec<RequestStat>>, delay_secs: u64) -> Vec<RequestStat> {
    let now = Instant::now()
        .checked_add(Duration::from_secs(delay_secs))
        .unwrap();
    let mut vec = Vec::new();
    while let Ok(mut stats) = stats_rx.recv() {
        vec.append(&mut stats);
    }
    println!(
        "Gathering results took {:?} (total ops: {})",
        now.elapsed(),
        TOTAL.load(SeqCst)
    );
    vec
}

fn main() {
    let args = Args::parse();

    let db_path = PathBuf::from(&args.db_path);

    // If setup database, just set up and return
    if args.setup_db {
        setup_database(db_path, args);
        return;
    }

    // Otherwise, setup workers and start
    println!("PID: {}", process::id());
    init_signal();
    init_counters(&args);

    let delay_secs = args.delay_secs;
    let quantile_path = args.quantiles_path.clone();

    let (stats_tx, stats_rx) = bounded(1024);
    setup_workers_existing_db(db_path, args, stats_tx);

    println!("Gathering statistics from worker threads");
    let mut stats = gather_stats(stats_rx, delay_secs);

    println!("Calculating percentiles");
    stats.sort_by(|a, b| a.duration_secs.partial_cmp(&b.duration_secs).unwrap());
    let n: f64 = stats.len() as f64;
    let percentiles = [
        0.01, 0.1, 0.25, 0.5, 0.75, 0.85, 0.9, 0.95, 0.975, 0.99, 0.999,
    ];

    // Write quantiles out
    let mut f = std::fs::File::create(quantile_path).unwrap();
    write!(
        f,
        "{}\n",
        percentiles
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
    .unwrap();

    write!(
        f,
        "{}\n",
        percentiles
            .iter()
            .map(|f| {
                let idx = (f * n) as usize;
                stats[idx].duration_secs.to_string()
            })
            .collect::<Vec<_>>()
            .join(", ")
    )
    .unwrap();

    for percentile in &percentiles {
        let idx = (percentile * n) as usize;
        println!("{}, {}", percentile, stats[idx].duration_secs);
    }
}
