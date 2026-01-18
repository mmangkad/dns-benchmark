//! XML output formatter.

use super::OutputFormatter;
use crate::benchmark::BenchmarkResult;
use crate::config::Config;
use crate::error::OutputError;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use std::io::{Cursor, Write};
use std::net::IpAddr;

/// XML output formatter
pub struct XmlFormatter;

impl OutputFormatter for XmlFormatter {
    fn write(
        &self,
        result: &BenchmarkResult,
        _config: &Config,
        _system_ips: &[IpAddr],
        writer: &mut dyn Write,
    ) -> Result<(), OutputError> {
        let mut buffer = Cursor::new(Vec::new());
        let mut xml_writer = Writer::new_with_indent(&mut buffer, b' ', 2);

        // XML declaration
        xml_writer
            .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
            .map_err(|e| OutputError::Xml(e.to_string()))?;

        // Root element
        let root = BytesStart::new("DnsBenchmarkResults");
        xml_writer
            .write_event(Event::Start(root))
            .map_err(|e| OutputError::Xml(e.to_string()))?;

        // Metadata
        write_element(&mut xml_writer, "Domain", &result.domain)?;
        write_element(&mut xml_writer, "RequestsPerServer", &result.requests_per_server.to_string())?;
        write_element(&mut xml_writer, "TotalServers", &result.servers.len().to_string())?;
        write_element(&mut xml_writer, "DurationMs", &format!("{:.2}", result.duration.as_secs_f64() * 1000.0))?;

        // Results
        let results_start = BytesStart::new("Results");
        xml_writer
            .write_event(Event::Start(results_start))
            .map_err(|e| OutputError::Xml(e.to_string()))?;

        for server in &result.servers {
            let server_start = BytesStart::new("Server");
            xml_writer
                .write_event(Event::Start(server_start))
                .map_err(|e| OutputError::Xml(e.to_string()))?;

            write_element(&mut xml_writer, "Name", &server.name)?;
            write_element(&mut xml_writer, "Ip", &server.ip.to_string())?;

            if let Some(resolved) = server.resolved_ip {
                write_element(&mut xml_writer, "ResolvedIp", &resolved.to_string())?;
            }

            write_element(&mut xml_writer, "TotalRequests", &server.total_requests.to_string())?;
            write_element(&mut xml_writer, "SuccessfulRequests", &server.successful_requests.to_string())?;
            write_element(&mut xml_writer, "SuccessRate", &format!("{:.2}", server.success_rate()))?;

            if let Some(min) = server.min_time {
                write_element(&mut xml_writer, "MinMs", &format!("{:.3}", min.as_secs_f64() * 1000.0))?;
            }
            if let Some(max) = server.max_time {
                write_element(&mut xml_writer, "MaxMs", &format!("{:.3}", max.as_secs_f64() * 1000.0))?;
            }
            if let Some(avg) = server.avg_time {
                write_element(&mut xml_writer, "AvgMs", &format!("{:.3}", avg.as_secs_f64() * 1000.0))?;
            }

            if server.all_failed() {
                if let Some(ref error) = server.last_error {
                    write_element(&mut xml_writer, "Error", error)?;
                }
            }

            xml_writer
                .write_event(Event::End(BytesEnd::new("Server")))
                .map_err(|e| OutputError::Xml(e.to_string()))?;
        }

        xml_writer
            .write_event(Event::End(BytesEnd::new("Results")))
            .map_err(|e| OutputError::Xml(e.to_string()))?;

        xml_writer
            .write_event(Event::End(BytesEnd::new("DnsBenchmarkResults")))
            .map_err(|e| OutputError::Xml(e.to_string()))?;

        let xml_content = String::from_utf8(buffer.into_inner())?;
        writeln!(writer, "{}", xml_content)?;

        Ok(())
    }
}

/// Helper to write a simple XML element
fn write_element<W: Write>(
    writer: &mut Writer<W>,
    name: &str,
    value: &str,
) -> Result<(), OutputError> {
    let start = BytesStart::new(name);
    writer
        .write_event(Event::Start(start))
        .map_err(|e| OutputError::Xml(e.to_string()))?;

    writer
        .write_event(Event::Text(BytesText::new(value)))
        .map_err(|e| OutputError::Xml(e.to_string()))?;

    writer
        .write_event(Event::End(BytesEnd::new(name)))
        .map_err(|e| OutputError::Xml(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmark::ServerResult;
    use crate::dns::ServerSource;
    use std::time::Duration;

    fn make_test_result() -> BenchmarkResult {
        BenchmarkResult {
            servers: vec![ServerResult {
                name: "Test".to_string(),
                ip: "8.8.8.8".parse().unwrap(),
                source: ServerSource::Builtin,
                resolved_ip: Some("1.2.3.4".parse().unwrap()),
                total_requests: 10,
                successful_requests: 9,
                min_time: Some(Duration::from_millis(5)),
                max_time: Some(Duration::from_millis(50)),
                avg_time: Some(Duration::from_millis(20)),
                last_error: None,
            }],
            duration: Duration::from_secs(1),
            domain: "google.com".to_string(),
            requests_per_server: 10,
        }
    }

    #[test]
    fn test_xml_output() {
        let result = make_test_result();
        let config = Config::default();
        let mut output = Vec::new();

        XmlFormatter.write(&result, &config, &[], &mut output).unwrap();

        let xml_str = String::from_utf8(output).unwrap();
        assert!(xml_str.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml_str.contains("<DnsBenchmarkResults>"));
        assert!(xml_str.contains("<Name>Test</Name>"));
    }
}
