use std::time::Instant;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

use super::topology::NetworkTopology;

/// Active latency prober for measuring real network conditions.
///
/// Sends lightweight TCP handshake probes to validator TPU/RPC endpoints
/// and updates the topology graph with fresh measurements.
pub struct LatencyProbe {
    timeout_ms: u64,
    max_concurrent: usize,
}

/// Result of a single probe to an endpoint.
#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub node_id: usize,
    pub addr: String,
    pub latency_ms: f64,
    pub success: bool,
    pub timestamp: u64,
}

impl Default for LatencyProbe {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            max_concurrent: 32,
        }
    }
}

impl LatencyProbe {
    pub fn new(timeout_ms: u64, max_concurrent: usize) -> Self {
        Self {
            timeout_ms,
            max_concurrent,
        }
    }

    /// Probe a single TCP endpoint and return round-trip time.
    pub async fn probe_endpoint(&self, addr: &str) -> Option<f64> {
        let start = Instant::now();
        let duration = Duration::from_millis(self.timeout_ms);

        match timeout(duration, TcpStream::connect(addr)).await {
            Ok(Ok(_stream)) => {
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                Some(elapsed)
            }
            _ => None,
        }
    }

    /// Probe multiple endpoints concurrently and return results.
    pub async fn probe_batch(&self, endpoints: Vec<(usize, String)>) -> Vec<ProbeResult> {
        let mut results = Vec::with_capacity(endpoints.len());
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Process in chunks to respect max_concurrent limit
        for chunk in endpoints.chunks(self.max_concurrent) {
            let mut handles = Vec::with_capacity(chunk.len());

            for (node_id, addr) in chunk {
                let addr = addr.clone();
                let node_id = *node_id;
                let timeout_ms = self.timeout_ms;

                handles.push(tokio::spawn(async move {
                    let start = Instant::now();
                    let duration = Duration::from_millis(timeout_ms);

                    match timeout(duration, TcpStream::connect(&addr)).await {
                        Ok(Ok(_)) => {
                            let latency = start.elapsed().as_secs_f64() * 1000.0;
                            (node_id, addr, latency, true)
                        }
                        _ => (node_id, addr, f64::MAX, false),
                    }
                }));
            }

            for handle in handles {
                if let Ok((node_id, addr, latency_ms, success)) = handle.await {
                    results.push(ProbeResult {
                        node_id,
                        addr,
                        latency_ms,
                        success,
                        timestamp: now,
                    });
                }
            }
        }

        results
    }

    /// Probe all stale edges in the topology and update latencies.
    pub async fn refresh_stale_edges(
        &self,
        topology: &mut NetworkTopology,
        stale_threshold_ms: u64,
        addr_resolver: &dyn Fn(usize) -> Option<String>,
    ) -> usize {
        let stale = topology.stale_edges(stale_threshold_ms);
        if stale.is_empty() {
            return 0;
        }

        let mut endpoints: Vec<(usize, String)> = Vec::new();
        let mut edge_map: Vec<(usize, usize)> = Vec::new();

        for (from, to) in &stale {
            if let Some(addr) = addr_resolver(*to) {
                endpoints.push((*to, addr));
                edge_map.push((*from, *to));
            }
        }

        let results = self.probe_batch(endpoints).await;
        let mut updated = 0;

        for (result, (from, to)) in results.iter().zip(edge_map.iter()) {
            if result.success {
                topology.update_latency(*from, *to, result.latency_ms);
                updated += 1;
            }
        }

        updated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_probe_localhost() {
        let probe = LatencyProbe::new(1000, 4);
        // This will likely fail (no server) but should not panic
        let result = probe.probe_endpoint("127.0.0.1:19999").await;
        // Just verify it returns None for unreachable endpoint
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_probe_batch_empty() {
        let probe = LatencyProbe::default();
        let results = probe.probe_batch(vec![]).await;
        assert!(results.is_empty());
    }
}
