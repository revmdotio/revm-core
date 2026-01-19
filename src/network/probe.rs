use std::time::Instant;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

use super::topology::NetworkTopology;

pub struct LatencyProbe {
    timeout_ms: u64,
    max_concurrent: usize,
}

impl Default for LatencyProbe {
    fn default() -> Self {
        Self { timeout_ms: 5000, max_concurrent: 32 }
    }
}

impl LatencyProbe {
    pub fn new(timeout_ms: u64, max_concurrent: usize) -> Self {
        Self { timeout_ms, max_concurrent }
    }

    pub async fn probe_endpoint(&self, addr: &str) -> Option<f64> {
        let start = Instant::now();
        let duration = Duration::from_millis(self.timeout_ms);
        match timeout(duration, TcpStream::connect(addr)).await {
            Ok(Ok(_stream)) => Some(start.elapsed().as_secs_f64() * 1000.0),
            _ => None,
        }
    }
}
