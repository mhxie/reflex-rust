use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug)]
pub struct Count {
    pub send: u64,
    pub recv: u64,
    pub send_bytes: u64,
    pub recv_bytes: u64,
}

impl Default for Count {
    fn default() -> Count {
        Count {
            send: 0,
            recv: 0,
            send_bytes: 0,
            recv_bytes: 0,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Args {
    pub addr: String,
    pub duration: u64,
    pub number: u32,
    pub length: usize,
    pub rw_ratio: u32,
}

impl Default for Args {
    fn default() -> Args {
        Args {
            addr: String::default(),
            duration: 0,
            number: 0,
            length: 0,
            rw_ratio: 100,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Perf {
    pub iops: u64,
    // tail latency, flat structure
    pub p10: u32,
    pub p50: u32,
    pub p95: u32,
    pub p99: u32,
}

impl Default for Perf {
    fn default() -> Self {
        Perf {
            iops: 0,
            p10: 0,
            p50: 0,
            p95: 0,
            p99: 0,
        }
    }
}

pub fn percentile(n: usize, latency: &[Duration]) -> Duration {
    if n > 100 {
        println!("Cannot calculate {}-percentile", n);
        Duration::default();
    }
    let ind = latency.len() * n / 100;
    latency[ind]
}

pub fn average(latency: &[Duration]) -> Duration {
    let s = latency.len();
    let mut sum = Duration::default();
    for rtt in latency {
        sum += *rtt;
    }
    Duration::from_secs_f64(sum.as_secs_f64() / s as f64)
}
