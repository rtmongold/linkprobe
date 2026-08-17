use crate::result::RunResult;

fn escape_label(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

/// OpenMetrics /Prometheus text exposition for a single run.
pub fn format_openmetrics(result: &RunResult) -> String {
    let backend = escape_label(&result.backend);
    let server = escape_label(&result.server.name);
    let labels = format!("backend=\"{backend}\", server=\"{server}\"");
    let mut out = String::new();

    out.push_str("# HELP linkprobe_ok 1 if the last probe succeeded.\n");
    out.push_str("# TYPE linkprobe_ok gauge\n");
    out.push_str(&format!("linkprobe_ok{{{labels}}} 1\n"));

    if let Some(ms) = result.measurement.latency_ms {
        out.push_str("# HELP linkprobe_latency_milliseconds Round-trip latency.\n");
        out.push_str("# TYPE linkprobe_latency_milliseconds gauge\n");
        out.push_str(&format!(
            "linkprobe_latency_milliseconds{{{labels}}} {ms}\n"
        ));
    }
    if let Some(ms) = result.measurement.jitter_ms {
        out.push_str("# HELP linkprobe_jitter_milliseconds Jitter.\n");
        out.push_str("# TYPE linkprobe_jitter_milliseconds gauge\n");
        out.push_str(&format!("linkprobe_jitter_milliseconds{{{labels}}} {ms}\n"));
    }
    if let Some(ref dl) = result.measurement.download {
        out.push_str("# HELP linkprobe_download_bits_per_second Download throughput.\n");
        out.push_str("# TYPE linkprobe_download_bits_per_second gauge\n");
        out.push_str(&format!(
            "linkprobe_download_bits_per_second{{{labels}}} {}\n",
            dl.bps
        ));
    }
    if let Some(ref ul) = result.measurement.upload {
        out.push_str("# HELP linkprobe_upload_bits_per_second Upload throughput.\n");
        out.push_str("# TYPE linkprobe_upload_bits_per_second gauge\n");
        out.push_str(&format!(
            "linkprobe_upload_bits_per_second{{{labels}}} {}\n",
            ul.bps
        ));
    }
    if let Some(loss) = result.measurement.packet_loss {
        out.push_str("# HELP linkprobe_packet_loss Packet loss ration in [0,1].\n");
        out.push_str("# TYPE linkprobe_packet_loss gauge\n");
        out.push_str(&format!("linkprobe_packet_loss{{{labels}}} {loss}\n"));
    }

    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measurement::{Measurement, Throughput};
    use crate::server::Server;

    #[test]
    fn formats_gauges() {
        let result = RunResult::new(
            "librespeed",
            Server::librespeed("https://example/"),
            Measurement {
                latency_ms: Some(12.5),
                jitter_ms: Some(1.0),
                download: Some(Throughput::from_bps(100_000_000.0)),
                upload: Some(Throughput::from_bps(50_000_000.0)),
                packet_loss: None,
            },
        );
        let text = format_openmetrics(&result);
        assert!(text.contains("linkprobe_ok{"));
        assert!(text.contains("linkprobe_latency_milliseconds{"));
        assert!(text.contains("12.5"));
        assert!(text.contains("linkprobe_download_bits_per_second{"));
        assert!(!text.contains("linkprobe_packet_loss_ratio"));
    }
}
