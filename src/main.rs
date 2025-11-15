use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use panarchy::{
    scenario::ScenarioLoader,
    web::{self, WebServerConfig},
};

#[derive(Debug, Clone, Parser)]
#[command(author, version, about = "PANARCHY runner with immersive web UI")]
struct Cli {
    /// Path to the scenario YAML file
    #[arg(long, default_value = "scenarios/tiny_island.yaml")]
    scenario: PathBuf,

    /// Override tick count (uses scenario default when omitted)
    #[arg(long)]
    ticks: Option<u64>,

    /// Override snapshot interval in ticks
    #[arg(long)]
    snapshot_interval: Option<u64>,

    /// Directory for snapshots
    #[arg(long)]
    snapshot_dir: Option<PathBuf>,

    /// Host/IP the UI server should bind to
    #[arg(long, default_value = "127.0.0.1")]
    web_host: String,

    /// Port for the UI server
    #[arg(long, default_value_t = 8080)]
    web_port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    run_with_ui(cli).await
}

async fn run_with_ui(cli: Cli) -> Result<()> {
    let loader = ScenarioLoader::new(".");
    let scenario = loader.load(&cli.scenario)?;
    let ticks = scenario.ticks(cli.ticks);
    let snapshot_interval = cli
        .snapshot_interval
        .unwrap_or(scenario.snapshot_interval_ticks);
    let snapshot_dir = cli
        .snapshot_dir
        .unwrap_or_else(|| PathBuf::from("snapshots"));

    let config = WebServerConfig {
        scenario,
        ticks,
        snapshot_interval,
        snapshot_dir,
        host: cli.web_host,
        port: cli.web_port,
    };
    web::run(config).await
}
