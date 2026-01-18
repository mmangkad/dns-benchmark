//! DNS Benchmark - High-performance DNS benchmarking tool

use clap::Parser;
use console::style;
use dns_benchmark::benchmark::{collect_servers, BenchmarkEngine};
use dns_benchmark::cli::{Cli, Command, ConfigCommand};
use dns_benchmark::config::Config;
use dns_benchmark::output::{get_formatter, OutputFormat};
use dns_benchmark::platform::get_system_dns_servers;
use std::io;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{} {}", style("Error:").red().bold(), e);
            ExitCode::FAILURE
        }
    }
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Config(cmd)) => handle_config_command(cmd),
        None => run_benchmark(cli).await,
    }
}

/// Handle config subcommands
fn handle_config_command(cmd: ConfigCommand) -> anyhow::Result<()> {
    match cmd {
        ConfigCommand::Init => {
            if Config::exists()? {
                println!("{} Config file already exists.", style("ℹ").blue());
                println!("  Use 'dns-benchmark config show' to view current settings.");
            } else {
                Config::default().save()?;
                println!("{} Config file created with default values.", style("✓").green());
            }
        }

        ConfigCommand::Show => {
            if !Config::exists()? {
                println!("{} No config file found.", style("ℹ").blue());
                println!("  Using default values. Run 'dns-benchmark config init' to create one.");
                println!();
            }
            let config = Config::load_or_default();
            println!("{}", style("Current Configuration:").cyan().bold());
            println!("{}", config);
        }

        ConfigCommand::Set(args) => {
            if !Config::exists()? {
                anyhow::bail!("Config file does not exist. Run 'dns-benchmark config init' first.");
            }

            let mut config = Config::load_or_default();
            config.merge(&args.options.to_overrides());
            config.save()?;
            println!("{} Configuration updated.", style("✓").green());
        }

        ConfigCommand::Reset => {
            if !Config::exists()? {
                println!("{} No config file to reset.", style("ℹ").blue());
            } else {
                Config::default().save()?;
                println!("{} Configuration reset to defaults.", style("✓").green());
            }
        }

        ConfigCommand::Delete => {
            if !Config::exists()? {
                println!("{} No config file to delete.", style("ℹ").blue());
            } else {
                Config::delete()?;
                println!("{} Configuration file deleted.", style("✓").green());
            }
        }

        ConfigCommand::Path => {
            let path = Config::path()?;
            println!("{}", path.display());
        }
    }

    Ok(())
}

/// Run the DNS benchmark
async fn run_benchmark(cli: Cli) -> anyhow::Result<()> {
    // Load config and apply CLI overrides
    let mut config = Config::load_or_default();
    config.merge(&cli.options.to_overrides());

    // Save config if requested
    if cli.options.save_config {
        config.save()?;
        if config.format == OutputFormat::Table {
            println!("{} Configuration saved.", style("✓").green());
        }
    }

    // Collect DNS servers to benchmark
    let servers = collect_servers(&config)?;

    if servers.is_empty() {
        anyhow::bail!("No DNS servers to benchmark");
    }

    // Get system DNS IPs for highlighting
    let system_ips: Vec<_> = if config.skip_system {
        vec![]
    } else {
        get_system_dns_servers(config.name_server_ip)
            .map(|s| s.into_iter().map(|ds| ds.ip()).collect())
            .unwrap_or_default()
    };

    // Run benchmark
    let engine = BenchmarkEngine::new(config.clone(), servers);
    let result = engine.run().await;

    // Output results
    let formatter = get_formatter(config.format);
    let mut stdout = io::stdout().lock();
    formatter.write(&result, &config, &system_ips, &mut stdout)?;

    Ok(())
}
