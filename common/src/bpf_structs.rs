use std::fmt::Display;

pub trait FromBytes {
    fn from_bytes(buf: &[u8]) -> Self;
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct PreadQueryRecord {
    pub fd: u64,
    pub cpu: u64,
    pub count: u64,
    pub max_count: u64,
    pub avg_count: u64,
}

impl Display for PreadQueryRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Record({}, {}, {}, {}, {})",
            self.fd, self.cpu, self.count, self.max_count, self.avg_count
        )
    }
}

impl FromBytes for PreadQueryRecord {
    fn from_bytes(buf: &[u8]) -> Self {
        unsafe {
            std::mem::transmute::<[u8; std::mem::size_of::<PreadQueryRecord>()], PreadQueryRecord>(
                buf.try_into().unwrap(),
            )
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct RawPreadRecord {
    pub time: u64,
    pub fd: u64,
    pub cpu: u64,
    pub count: u64,
}

impl Display for RawPreadRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Record({}, {}, {}, {})",
            self.time, self.fd, self.cpu, self.count
        )
    }
}

impl FromBytes for RawPreadRecord {
    fn from_bytes(buf: &[u8]) -> Self {
        unsafe {
            std::mem::transmute::<[u8; std::mem::size_of::<RawPreadRecord>()], RawPreadRecord>(
                buf.try_into().unwrap(),
            )
        }
    }
}
