use serde::{Deserialize, Serialize};
#[derive(Default, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct RequestStat {
    pub pid: u64,
    pub timestamp: u64,
    pub duration_secs: f64,
}

pub trait SerializeDeserialize {
    fn serialize_into(&self, data: &mut Vec<u8>);
    fn deserialize_from(data: &[u8]) -> (Self, usize)
    where
        Self: Sized;
}

impl SerializeDeserialize for RequestStat {
    //const fn _bytes_size() -> usize {
    //    24
    //}
    fn serialize_into(&self, data: &mut Vec<u8>) {
        data.extend_from_slice(&self.pid.to_be_bytes());
        data.extend_from_slice(&self.timestamp.to_be_bytes());
        data.extend_from_slice(&self.duration_secs.to_be_bytes());
    }

    fn deserialize_from(data: &[u8]) -> (Self, usize) {
        let pid = u64::from_be_bytes(data[..8].try_into().unwrap());
        let timestamp = u64::from_be_bytes(data[8..16].try_into().unwrap());
        let duration_secs = f64::from_be_bytes(data[16..24].try_into().unwrap());
        let read = 24;
        let result = Self {
            pid,
            timestamp,
            duration_secs,
        };
        (result, read)
    }
}

impl RequestStat {
    pub fn new(pid: u64, timestamp: u64, duration_secs: f64) -> Self {
        Self {
            pid,
            timestamp,
            duration_secs,
        }
    }
}

const REQUEST_STAT_BUFFER_SIZE: usize = 1024;

#[derive(Debug, Copy, Clone)]
pub struct RequestStatBuffer {
    pub len: usize,
    pub data: [RequestStat; REQUEST_STAT_BUFFER_SIZE],
}

impl Default for RequestStatBuffer {
    fn default() -> Self {
        Self {
            len: 0,
            data: [RequestStat::default(); REQUEST_STAT_BUFFER_SIZE],
        }
    }
}

impl RequestStatBuffer {
    pub fn data(&self) -> &[RequestStat] {
        &self.data[..self.len]
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn new() -> Self {
        Self {
            len: 0,
            data: [RequestStat::default(); REQUEST_STAT_BUFFER_SIZE],
        }
    }

    pub fn push(&mut self, item: RequestStat) -> Result<(), ()> {
        if self.len < REQUEST_STAT_BUFFER_SIZE {
            self.data[self.len] = item;
            self.len += 1;
            Ok(())
        } else {
            Err(())
        }
    }
}
