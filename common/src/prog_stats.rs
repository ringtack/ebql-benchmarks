use std::fmt::Display;

use chrono::{Duration, Local};
use procfs::{process::Process, WithCurrentSystemInfo};

pub struct ProgStats {
    pub utime: u64,
    pub stime: u64,
    pub runtime: Duration,
    pub clock_tps: u64,
}

impl Display for ProgStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}",
            self.utime,
            self.stime,
            self.runtime.num_nanoseconds().unwrap(),
            self.clock_tps,
        )
    }
}

impl ProgStats {
    pub fn get() -> Self {
        let me = Process::myself().unwrap();
        let stat = me.stat().unwrap();
        let tps = procfs::ticks_per_second();
        let start_time = stat.starttime().get().unwrap();
        let now = Local::now();

        Self {
            utime: stat.utime,
            stime: stat.stime,
            runtime: now - start_time,
            clock_tps: tps,
        }
    }
}
