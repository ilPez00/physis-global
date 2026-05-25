use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::actor::PDCActor;
use crate::config::{OntologyLoader, PhysisConfig};
use crate::dream::DreamEngine;
use crate::mapper::OntologyMapper;
use crate::models::{Goal, Score};
use crate::network::NetworkScanner;
use crate::output;
use crate::trie::DynamicVectorTrie;

#[derive(Parser)]
#[command(name = "physis", version, about = "Ontological mapper and PDCA dream engine")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan a directory and build ontological map
    Scan {
        /// Directory to scan
        dir: PathBuf,
        /// Output format (wiki, json, mermaid)
        #[arg(short, long, default_value = "wiki")]
        format: String,
    },
    /// Deep AI-powered scan of files and images
    DeepScan {
        /// Directory to scan
        dir: PathBuf,
    },
    /// Query the ontology trie
    Query {
        /// Search query
        query: String,
        #[arg(short, long, default_value = "10")]
        max_results: usize,
    },
    /// Generate stochastic dreams
    Dream {
        /// Number of dreams to generate
        #[arg(short, long, default_value = "5")]
        count: usize,
    },
    /// Evaluate a dream
    Evaluate {
        /// Dream ID
        id: String,
        /// Grade (0.0 - 1.0)
        grade: Score,
    },
    /// Start MCP server
    #[cfg(feature = "mcp")]
    Serve,
    /// Watch directories for changes
    Watch {
        /// Directories to watch
        dirs: Vec<PathBuf>,
    },
    /// Show stats
    Stats,
    /// Print config
    Config,
}

pub struct PhysisApp {
    pub config: PhysisConfig,
    pub ontology: OntologyLoader,
    pub mapper: OntologyMapper,
    pub actor: PDCActor,
    pub dreams: DreamEngine,
    pub goals: Vec<Goal>,
}

impl PhysisApp {
    pub fn new(config: PhysisConfig) -> Self {
        let ontology = OntologyLoader::load_all(&config);
        let mapper = OntologyMapper::new(ontology.clone());
        let actor = PDCActor::new(config.pdca_stagnant_threshold, config.pdca_stagnant_window);
        let dreams = DreamEngine::new();

        Self {
            config,
            ontology,
            mapper,
            actor,
            dreams,
            goals: Vec::new(),
        }
    }

    pub fn run_scan(&mut self, dir: &std::path::Path, format: &str) -> String {
        let goals = self.mapper.map_filesystem(dir, None);
        self.goals = goals;
        // trie lives in mapper; dreams uses vector operations directly

        match format {
            "json" => output::format_json_graph(&self.mapper.trie),
            "mermaid" => output::format_mermaid_mindmap(&self.mapper.trie, "Physis Scan"),
            _ => output::format_wiki(&self.mapper.trie, &self.goals, "Physis Scan"),
        }
    }

    pub fn run_query(&self, query: &str, max_results: usize) -> Vec<Vec<String>> {
        let words: Vec<&str> = query.split_whitespace().collect();
        let tids: Vec<u32> = words
            .iter()
            .filter_map(|w| self.mapper.trie.token_id(w))
            .collect();
        if tids.is_empty() {
            return vec![];
        }
        self.mapper.trie.prefix_search(&tids, 2, max_results)
    }

    pub fn run_dream(&mut self, count: usize) -> Vec<crate::models::Dream> {
        self.dreams.generate_dreams(&self.goals, count)
    }

    pub fn run_evaluate(&mut self, id: &str, grade: Score) -> bool {
        self.dreams.evaluate_dream(id, grade)
    }

    pub async fn run_deep_scan(&mut self, dir: &std::path::Path) -> anyhow::Result<String> {
        use crate::ai::provider::ProviderCascade;
        use crate::ai::agent::{run_agent, AgentConfig};
        use crate::ai::tools::ToolRegistry;
        use crate::scanner::{scan_project, BINARY_EXTENSIONS};
        use base64::prelude::*;

        println!("Starting Deep Scan on {}...", dir.display());
        let cascade = ProviderCascade::from_env();
        let tools = ToolRegistry::new();
        let mut global_map = crate::models::OntologicalMap::new();

        let files = scan_project(dir, None);
        println!("Found {} candidate files.", files.len());

        for file in files {
            println!("Processing {}...", file.path);
            let ext = file.ext.to_lowercase();
            
            let result = if [".png", ".jpg", ".jpeg"].contains(&ext.as_str()) {
                // Image Deep Scan
                let bytes = std::fs::read(&file.abs_path)?;
                let b64 = BASE64_STANDARD.encode(bytes);
                let config = AgentConfig {
                    system_prompt: r#"You are Physis Vision. Extract an ontological map (JSON).
Output ONLY a JSON object with this EXACT structure:
{
  "entities": {
    "entity_id": {"id":"entity_id", "name":"Human Name", "kind":"kind", "description":"...", "attributes":{}}
  },
  "relationships": [
    {"source":"entity_id1", "target":"entity_id2", "predicate":"verb", "weight":1.0}
  ]
}
No other text."#.into(),
                    ..Default::default()
                };
                run_agent(&cascade, &tools, &config, &[], &format!("Analyze image: {}", file.path), Some(&b64), "DATA", None).await
            } else {
                // Text Deep Scan
                let text = std::fs::read_to_string(&file.abs_path)?;
                let config = AgentConfig {
                    system_prompt: r#"You are Physis. Extract an ontological map (JSON).
Output ONLY a JSON object with this EXACT structure:
{
  "entities": {
    "entity_id": {"id":"entity_id", "name":"Human Name", "kind":"kind", "description":"...", "attributes":{}}
  },
  "relationships": [
    {"source":"entity_id1", "target":"entity_id2", "predicate":"verb", "weight":1.0}
  ]
}
No other text."#.into(),
                    ..Default::default()
                };
                run_agent(&cascade, &tools, &config, &[], &format!("Analyze text ({}): \n{}", file.path, text), None, "DATA", None).await
            };

            if let Ok(output) = result {
                if let (Some(s), Some(e)) = (output.text.find('{'), output.text.rfind('}')) {
                    if let Ok(new_map) = serde_json::from_str::<crate::models::OntologicalMap>(&output.text[s..=e]) {
                        global_map.merge(new_map);
                        println!("Merged ontology for {}.", file.path);
                    }
                }
            }
        }

        Ok(serde_json::to_string_pretty(&global_map)?)
    }

    pub fn run_watch(&mut self, dirs: Vec<PathBuf>) -> String {
        let cache_path = self.config.data_dir.join("hash_cache.json");
        let scanner = NetworkScanner::new(dirs.clone(), cache_path, self.config.network_scan_interval_secs);
        let diffs = scanner.scan_all();
        let mut changes = Vec::new();
        for diff in &diffs {
            changes.push(format!("{} new, {} changed, {} deleted",
                diff.summary.new, diff.summary.changed, diff.summary.deleted));
        }
        changes.join("\n")
    }

    pub fn run_stats(&self) -> String {
        let stats = self.mapper.stats();
        let mut out = String::new();
        out.push_str("=== Physis Stats ===\n");
        let mut pairs: Vec<_> = stats.into_iter().collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        for (k, v) in pairs {
            out.push_str(&format!("  {k}: {v}\n"));
        }
        let actor_stats = self.actor.stats(&self.goals);
        out.push_str(&format!("  total_actions: {}\n", actor_stats.total_actions));
        out.push_str(&format!("  avg_grade: {:.3}\n", actor_stats.avg_grade));
        if actor_stats.stagnant_count > 0 {
            out.push_str(&format!("  stagnant_goals: {}\n", actor_stats.stagnant_count));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_parser_scan() {
        let cli = Cli::try_parse_from(["physis", "scan", "/tmp"]);
        assert!(cli.is_ok());
        match cli.unwrap().command {
            Commands::Scan { dir, format } => {
                assert_eq!(dir, std::path::PathBuf::from("/tmp"));
                assert_eq!(format, "wiki");
            }
            _ => panic!("expected Scan command"),
        }
    }

    #[test]
    fn test_cli_parser_scan_with_format() {
        let cli = Cli::try_parse_from(["physis", "scan", "/tmp", "--format", "json"]);
        assert!(cli.is_ok());
        match cli.unwrap().command {
            Commands::Scan { format, .. } => assert_eq!(format, "json"),
            _ => panic!("expected Scan command"),
        }
    }

    #[test]
    fn test_cli_parser_query() {
        let cli = Cli::try_parse_from(["physis", "query", "hello world"]);
        assert!(cli.is_ok());
        match cli.unwrap().command {
            Commands::Query { query, max_results } => {
                assert_eq!(query, "hello world");
                assert_eq!(max_results, 10);
            }
            _ => panic!("expected Query command"),
        }
    }

    #[test]
    fn test_cli_parser_dream() {
        let cli = Cli::try_parse_from(["physis", "dream"]);
        assert!(cli.is_ok());
        match cli.unwrap().command {
            Commands::Dream { count } => assert_eq!(count, 5),
            _ => panic!("expected Dream command"),
        }
    }

    #[test]
    fn test_cli_parser_evaluate() {
        let cli = Cli::try_parse_from(["physis", "evaluate", "dream-1", "0.85"]);
        assert!(cli.is_ok());
        match cli.unwrap().command {
            Commands::Evaluate { id, grade } => {
                assert_eq!(id, "dream-1");
                assert!((grade - 0.85).abs() < 1e-6);
            }
            _ => panic!("expected Evaluate command"),
        }
    }

    #[test]
    fn test_cli_parser_stats() {
        let cli = Cli::try_parse_from(["physis", "stats"]);
        assert!(cli.is_ok());
        assert!(matches!(cli.unwrap().command, Commands::Stats));
    }

    #[test]
    fn test_cli_parser_config() {
        let cli = Cli::try_parse_from(["physis", "config"]);
        assert!(cli.is_ok());
        assert!(matches!(cli.unwrap().command, Commands::Config));
    }

    #[test]
    fn test_cli_parser_watch() {
        let cli = Cli::try_parse_from(["physis", "watch", "/dir1", "/dir2"]);
        assert!(cli.is_ok());
        match cli.unwrap().command {
            Commands::Watch { dirs } => assert_eq!(dirs.len(), 2),
            _ => panic!("expected Watch command"),
        }
    }

    #[test]
    fn test_cli_parser_unknown_command_fails() {
        let cli = Cli::try_parse_from(["physis", "bogus"]);
        assert!(cli.is_err());
    }

    #[test]
    fn test_cli_parser_missing_arg_fails() {
        let cli = Cli::try_parse_from(["physis", "scan"]);
        assert!(cli.is_err());
    }

    #[test]
    fn test_cli_command_factory_usage() {
        let cli = Cli::command();
        let name = cli.get_name();
        assert_eq!(name, "physis");
    }

    #[test]
    fn test_app_new_with_default_config() {
        let config = PhysisConfig::default();
        let app = PhysisApp::new(config);
        assert!(app.goals.is_empty());
        assert!(app.actor.stats(&[]).total_actions == 0);
    }

    #[test]
    fn test_run_stats_empty() {
        let config = PhysisConfig::default();
        let app = PhysisApp::new(config);
        let stats = app.run_stats();
        assert!(stats.contains("Physis Stats"));
        assert!(stats.contains("total_actions: 0"));
    }

    #[test]
    fn test_run_query_empty() {
        let config = PhysisConfig::default();
        let app = PhysisApp::new(config);
        let results = app.run_query("nonexistent", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_run_dream_empty_goals() {
        let config = PhysisConfig::default();
        let mut app = PhysisApp::new(config);
        let dreams = app.run_dream(3);
        assert_eq!(dreams.len(), 0, "no goals means no dreams");
    }

    #[test]
    fn test_run_evaluate_nonexistent() {
        let config = PhysisConfig::default();
        let mut app = PhysisApp::new(config);
        let result = app.run_evaluate("bogus", 0.5);
        assert!(!result);
    }
}
