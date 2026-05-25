use clap::Parser;

use physis::cli::{Cli, Commands, PhysisApp};
use physis::config::PhysisConfig;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = PhysisConfig::default();
    let mut app = PhysisApp::new(config);

    match cli.command {
        Commands::Scan { dir, format } => {
            let output = app.run_scan(&dir, &format);
            println!("{output}");
        }
        Commands::DeepScan { dir } => {
            let rt = tokio::runtime::Runtime::new()?;
            let output = rt.block_on(app.run_deep_scan(&dir))?;
            println!("{output}");
        }
        Commands::Query { query, max_results } => {
            let results = app.run_query(&query, max_results);
            if results.is_empty() {
                println!("No results found for: {query}");
            } else {
                for path in results {
                    println!("  {}", path.join(" → "));
                }
            }
        }
        Commands::Dream { count } => {
            let dreams = app.run_dream(count);
            for dream in &dreams {
                println!("[{}] source={:?} embedding={:?} grade={:?}",
                    dream.id,
                    &dream.source[..dream.source.len().min(4)],
                    &dream.embedding[..dream.embedding.len().min(4)],
                    dream.grade);
            }
        }
        Commands::Evaluate { id, grade } => {
            if app.run_evaluate(&id, grade) {
                println!("Dream {id} evaluated with grade {grade}");
            } else {
                println!("Dream {id} not found");
            }
        }
        #[cfg(feature = "mcp")]
        Commands::Serve => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let config = PhysisConfig::default();
                physis::mcp::start_mcp_server(config).await
            })?;
        }
        Commands::Watch { dirs } => {
            let output = app.run_watch(dirs);
            println!("{output}");
        }
        Commands::Stats => {
            let output = app.run_stats();
            println!("{output}");
        }
        Commands::Config => {
            let config_str = serde_json::to_string_pretty(&PhysisConfig::default())?;
            println!("{config_str}");
        }
        Commands::Translate { text, lense } => {
            let output = app.run_translate(&text, lense.as_deref());
            println!("{output}");
        }
    }

    Ok(())
}
