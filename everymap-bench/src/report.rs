use crate::benchmark::BenchmarkReport;

/// Format a benchmark report as JSON.
pub fn format_json(report: &BenchmarkReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
}

/// Format a benchmark report as a markdown table.
pub fn format_markdown(report: &BenchmarkReport) -> String {
    let mut md = String::new();
    md.push_str(&format!("# Benchmark: {}\n\n", report.scenario));
    md.push_str(&format!("**Domain**: {} | **Timestamp**: {}\n\n", report.domain, report.timestamp));

    md.push_str("| Provider | Duration (ms) | Success | Results | Error |\n");
    md.push_str("|----------|---------------|---------|---------|-------|\n");

    for result in &report.results {
        let success = if result.success { "Yes" } else { "No" };
        let error = result.error.as_deref().unwrap_or("-");
        md.push_str(&format!("| {} | {} | {} | {} | {} |\n",
            result.provider, result.duration_ms, success, result.result_count, error));
    }

    if let Some(fastest) = &report.fastest {
        md.push_str(&format!("\n**Fastest**: {}\n", fastest));
    }
    if let Some(most) = &report.most_results {
        md.push_str(&format!("**Most results**: {}\n", most));
    }

    if let Some(percentiles) = &report.percentiles {
        md.push_str("\n## Percentile Latency\n\n");
        md.push_str("| Provider | p50 (ms) | p95 (ms) | p99 (ms) | Min (ms) | Max (ms) | OK/Err |\n");
        md.push_str("|----------|----------|----------|----------|----------|----------|--------|\n");
        for p in percentiles {
            md.push_str(&format!("| {} | {} | {} | {} | {} | {} | {}/{} |\n",
                p.provider, p.p50_ms, p.p95_ms, p.p99_ms, p.min_ms, p.max_ms, p.success_count, p.error_count));
        }
    }

    md
}

/// Format a benchmark report as a plain text table.
pub fn format_table(report: &BenchmarkReport) -> String {
    let mut table = String::new();
    table.push_str(&format!("Benchmark: {} ({})\n", report.scenario, report.domain));
    table.push_str(&"-".repeat(70));
    table.push_str(&format!("{:<12} {:>12} {:>8} {:>8}  {}\n", "Provider", "Duration(ms)", "Success", "Results", "Error"));
    table.push_str(&"-".repeat(70));

    for result in &report.results {
        let success = if result.success { "OK" } else { "FAIL" };
        let error = result.error.as_deref().unwrap_or("-");
        table.push_str(&format!("{:<12} {:>12} {:>8} {:>8}  {}\n",
            result.provider, result.duration_ms, success, result.result_count, error));
    }

    if let Some(fastest) = &report.fastest {
        table.push_str(&format!("\nFastest: {}\n", fastest));
    }

    if let Some(percentiles) = &report.percentiles {
        table.push_str(&format!("\n{:<12} {:>10} {:>10} {:>10} {:>10} {:>10}  {}\n",
            "Provider", "p50(ms)", "p95(ms)", "p99(ms)", "Min(ms)", "Max(ms)", "OK/Err"));
        table.push_str(&"-".repeat(70));
        for p in percentiles {
            table.push_str(&format!("{:<12} {:>10} {:>10} {:>10} {:>10} {:>10}  {}/{}\n",
                p.provider, p.p50_ms, p.p95_ms, p.p99_ms, p.min_ms, p.max_ms, p.success_count, p.error_count));
        }
    }

    table
}