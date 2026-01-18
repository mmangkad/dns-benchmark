//! Table output formatter.

use super::{format_duration_ms, get_success_color, get_time_color, OutputFormatter};
use crate::benchmark::BenchmarkResult;
use crate::config::{Config, TableStyle};
use crate::error::OutputError;
use console::{style, Color};
use std::io::Write;
use std::net::IpAddr;
use std::time::Duration;
use tabled::settings::{object, Alignment, Color as TabledColor, Modify, Style};
use tabled::{Table, Tabled};

/// Table output formatter
pub struct TableFormatter;

impl OutputFormatter for TableFormatter {
    fn write(
        &self,
        result: &BenchmarkResult,
        config: &Config,
        system_ips: &[IpAddr],
        writer: &mut dyn Write,
    ) -> Result<(), OutputError> {
        let rows: Vec<TableRow> = result
            .servers
            .iter()
            .map(|s| TableRow::from_result(s, system_ips))
            .collect();

        let mut table = Table::new(&rows);

        // Apply style
        apply_style(&mut table, config.style);

        // Center header
        table.with(Modify::new(object::Rows::first()).with(Alignment::center()));

        // Apply colors to data cells
        for (i, s) in result.servers.iter().enumerate() {
            let row_idx = i + 1; // Skip header row

            // Success rate color
            table.with(
                Modify::new(object::Cell::new(row_idx, 3))
                    .with(to_tabled_color(get_success_color(s.success_rate()))),
            );

            // Time columns color (if we have data)
            if let Some(min) = s.min_time {
                let ms = min.as_secs_f64() * 1000.0;
                table.with(
                    Modify::new(object::Cell::new(row_idx, 4))
                        .with(to_tabled_color(get_time_color(ms))),
                );
            }
            if let Some(max) = s.max_time {
                let ms = max.as_secs_f64() * 1000.0;
                table.with(
                    Modify::new(object::Cell::new(row_idx, 5))
                        .with(to_tabled_color(get_time_color(ms))),
                );
            }
            if let Some(avg) = s.avg_time {
                let ms = avg.as_secs_f64() * 1000.0;
                table.with(
                    Modify::new(object::Cell::new(row_idx, 6))
                        .with(to_tabled_color(get_time_color(ms))),
                );
            }
        }

        writeln!(writer, "{}", table)?;

        // Print summary
        writeln!(writer)?;
        writeln!(
            writer,
            "{} Benchmark completed in {:.2?}",
            style("✓").green().bold(),
            result.duration
        )?;

        if let Some(fastest) = result.fastest() {
            if let Some(avg) = fastest.avg_time {
                writeln!(
                    writer,
                    "{} Fastest: {} ({}) - {}",
                    style("★").yellow().bold(),
                    style(&fastest.name).green(),
                    fastest.ip,
                    style(format_duration_ms(avg.as_secs_f64() * 1000.0)).cyan()
                )?;
            }
        }

        Ok(())
    }
}

/// Table row representation
#[derive(Debug, Tabled)]
struct TableRow {
    #[tabled(rename = "Server")]
    name: String,
    #[tabled(rename = "IP Address")]
    ip: String,
    #[tabled(rename = "Resolved IP")]
    resolved_ip: String,
    #[tabled(rename = "Success Rate")]
    success_rate: String,
    #[tabled(rename = "Min")]
    min: String,
    #[tabled(rename = "Max")]
    max: String,
    #[tabled(rename = "Avg ↑")]
    avg: String,
}

impl TableRow {
    fn from_result(r: &crate::benchmark::ServerResult, system_ips: &[IpAddr]) -> Self {
        let name = if system_ips.contains(&r.ip) {
            format!("▸ {}", r.name)
        } else {
            r.name.clone()
        };

        Self {
            name,
            ip: r.ip.to_string(),
            resolved_ip: r.resolved_ip.map(|ip| ip.to_string()).unwrap_or_else(|| "-".into()),
            success_rate: format!(
                "{}/{} ({:.1}%)",
                r.successful_requests,
                r.total_requests,
                r.success_rate()
            ),
            min: format_time(r.min_time),
            max: format_time(r.max_time),
            avg: format_time(r.avg_time),
        }
    }
}

/// Format a duration for display
fn format_time(d: Option<Duration>) -> String {
    match d {
        Some(d) => format_duration_ms(d.as_secs_f64() * 1000.0),
        None => "-".into(),
    }
}

/// Apply table style
fn apply_style(table: &mut Table, style: TableStyle) {
    match style {
        TableStyle::Empty => { table.with(Style::empty()); }
        TableStyle::Blank => { table.with(Style::blank()); }
        TableStyle::Ascii => { table.with(Style::ascii()); }
        TableStyle::Psql => { table.with(Style::psql()); }
        TableStyle::Markdown => { table.with(Style::markdown()); }
        TableStyle::Modern => { table.with(Style::modern()); }
        TableStyle::Sharp => { table.with(Style::sharp()); }
        TableStyle::Rounded => { table.with(Style::rounded()); }
        TableStyle::ModernRounded => { table.with(Style::modern_rounded()); }
        TableStyle::Extended => { table.with(Style::extended()); }
        TableStyle::Dots => { table.with(Style::dots()); }
        TableStyle::ReStructuredText => { table.with(Style::re_structured_text()); }
        TableStyle::AsciiRounded => { table.with(Style::ascii_rounded()); }
    };
}

/// Convert console color to tabled color
fn to_tabled_color(color: Color) -> TabledColor {
    match color {
        Color::Green => TabledColor::FG_BRIGHT_GREEN,
        Color::Yellow => TabledColor::FG_BRIGHT_YELLOW,
        Color::Red => TabledColor::FG_BRIGHT_RED,
        Color::Magenta => TabledColor::FG_MAGENTA,
        Color::Cyan => TabledColor::FG_CYAN,
        _ => TabledColor::default(),
    }
}
