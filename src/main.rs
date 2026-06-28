//! ace-tool-rs CLI and MCP server entrypoint.

use ace_tool::config::{Config, ConfigOptions};
use ace_tool::enhancer::prompt_enhancer::{get_enhancer_endpoint, PromptEnhancer};
use ace_tool::index::{IndexManager, IndexResult};
use ace_tool::mcp::{McpServer, TransportMode};
use ace_tool::service::get_third_party_config;
use ace_tool::tools::search_context::{SearchContextArgs, SearchContextTool};
use anyhow::{anyhow, Context, Result};
use clap::{Args as ClapArgs, Parser, Subcommand, ValueEnum};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const REPO_SKILL_PATH: &str = "skills/ace-tool-rs";
const DEFAULT_CONFIG_RELATIVE_PATH: &str = "ace-tool-rs/config.toml";

#[derive(Debug, Clone, Default, Deserialize)]
struct FileConfig {
    base_url: Option<String>,
    token: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct AceCredentials {
    base_url: Option<String>,
    token: Option<String>,
}

#[derive(ValueEnum, Debug, Copy, Clone, Eq, PartialEq, Default)]
enum TransportArg {
    #[default]
    Auto,
    Lsp,
    Line,
}

#[derive(ValueEnum, Debug, Copy, Clone, Eq, PartialEq)]
enum AgentTarget {
    Codex,
    Claude,
    Pi,
}

#[derive(ClapArgs, Debug, Clone, Default)]
struct AceConfigArgs {
    /// Path to config file (default: ~/.config/ace-tool-rs/config.toml)
    #[arg(long)]
    config: Option<PathBuf>,

    /// API base URL for the indexing service
    #[arg(long)]
    base_url: Option<String>,

    /// Authentication token
    #[arg(long)]
    token: Option<String>,

    /// Maximum lines per blob (default: 800)
    #[arg(long)]
    max_lines_per_blob: Option<usize>,

    /// Upload timeout in seconds (default: adaptive)
    #[arg(long)]
    upload_timeout: Option<u64>,

    /// Upload concurrency (default: adaptive)
    #[arg(long)]
    upload_concurrency: Option<usize>,

    /// Retrieval timeout in seconds (default: 60)
    #[arg(long)]
    retrieval_timeout: Option<u64>,

    /// Disable adaptive strategy
    #[arg(long, default_value = "false")]
    no_adaptive: bool,
}

impl AceConfigArgs {
    fn has_values(&self) -> bool {
        self.config.is_some()
            || self.base_url.is_some()
            || self.token.is_some()
            || self.max_lines_per_blob.is_some()
            || self.upload_timeout.is_some()
            || self.upload_concurrency.is_some()
            || self.retrieval_timeout.is_some()
            || self.no_adaptive
    }
}

#[derive(ClapArgs, Debug, Clone, Default)]
struct PromptUiArgs {
    /// Disable web browser interaction for enhance_prompt, return API result directly
    #[arg(long, default_value = "false")]
    no_webbrowser_enhance_prompt: bool,

    /// Force using xdg-open instead of explorer.exe in WSL environment
    #[arg(long, default_value = "false")]
    force_xdg_open: bool,

    /// Bind address and port for the enhance_prompt Web UI server
    #[arg(long)]
    webui_addr: Option<String>,
}

impl PromptUiArgs {
    fn has_values(&self) -> bool {
        self.no_webbrowser_enhance_prompt || self.force_xdg_open || self.webui_addr.is_some()
    }
}

#[derive(ClapArgs, Debug, Clone, Default)]
struct LegacyArgs {
    #[command(flatten)]
    ace: AceConfigArgs,

    #[command(flatten)]
    prompt_ui: PromptUiArgs,

    /// Transport framing: auto, lsp, line
    #[arg(long, value_enum, default_value = "auto")]
    transport: TransportArg,

    /// Index-only mode: index current directory and exit (no MCP server)
    #[arg(long, default_value = "false")]
    index_only: bool,

    /// Enhance a prompt and output the result to stdout, then exit
    #[arg(long)]
    enhance_prompt: Option<String>,
}

impl LegacyArgs {
    fn has_values(&self) -> bool {
        self.ace.has_values()
            || self.prompt_ui.has_values()
            || self.transport != TransportArg::Auto
            || self.index_only
            || self.enhance_prompt.is_some()
    }
}

#[derive(Parser, Debug)]
#[command(name = "ace-tool-rs")]
#[command(version)]
#[command(about = "CLI, skill installer, and MCP server for codebase semantic search")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    legacy: LegacyArgs,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the MCP server over stdio
    Mcp(McpCommand),
    /// Index a project and exit
    Index(IndexCommand),
    /// Search a project with natural language
    Search(SearchCommand),
    /// Enhance a prompt and print the result
    Enhance(EnhanceCommand),
    /// Install the bundled assistant skill locally
    InstallSkill(InstallSkillCommand),
}

#[derive(ClapArgs, Debug, Clone)]
struct McpCommand {
    #[command(flatten)]
    ace: AceConfigArgs,

    #[command(flatten)]
    prompt_ui: PromptUiArgs,

    /// Transport framing: auto, lsp, line
    #[arg(long, value_enum, default_value = "auto")]
    transport: TransportArg,
}

#[derive(ClapArgs, Debug, Clone)]
struct IndexCommand {
    #[command(flatten)]
    ace: AceConfigArgs,

    /// Project root to index (defaults to current directory)
    #[arg(long)]
    project_root: Option<PathBuf>,
}

#[derive(ClapArgs, Debug, Clone)]
struct SearchCommand {
    #[command(flatten)]
    ace: AceConfigArgs,

    /// Project root to search (defaults to current directory)
    #[arg(long)]
    project_root: Option<PathBuf>,

    /// Natural-language code search query
    #[arg(long)]
    query: String,
}

#[derive(ClapArgs, Debug, Clone)]
struct EnhanceCommand {
    #[command(flatten)]
    ace: AceConfigArgs,

    #[command(flatten)]
    prompt_ui: PromptUiArgs,

    /// Prompt text to enhance
    #[arg(long)]
    prompt: String,

    /// Recent conversation history in "User: ...\nAssistant: ..." format
    #[arg(long)]
    conversation_history: Option<String>,

    /// Project root for optional codebase context (defaults to current directory)
    #[arg(long)]
    project_root: Option<PathBuf>,
}

#[derive(ClapArgs, Debug, Clone)]
struct InstallSkillCommand {
    /// Comma-separated agents to install for: codex, claude, pi
    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        default_value = "codex,claude,pi"
    )]
    agents: Vec<AgentTarget>,

    /// Skill source directory (defaults to skills/ace-tool-rs in the repo)
    #[arg(long)]
    source: Option<PathBuf>,

    /// Replace an existing ace-tool-rs skill directory
    #[arg(long, default_value = "false")]
    force: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    if let Some(command) = args.command {
        if args.legacy.has_values() {
            return Err(anyhow!(
                "legacy top-level options cannot be combined with subcommands; put options after the subcommand"
            ));
        }
        return run_command(command).await;
    }

    run_legacy(args.legacy).await
}

async fn run_command(command: Commands) -> Result<()> {
    match command {
        Commands::Mcp(command) => run_mcp(command.ace, command.prompt_ui, command.transport).await,
        Commands::Index(command) => {
            let project_root = command.project_root.unwrap_or(env::current_dir()?);
            run_index(command.ace, project_root, true).await
        }
        Commands::Search(command) => {
            let project_root = command.project_root.unwrap_or(env::current_dir()?);
            run_search(command.ace, project_root, command.query).await
        }
        Commands::Enhance(command) => {
            let project_root = command.project_root.unwrap_or(env::current_dir()?);
            run_enhance(
                command.ace,
                command.prompt_ui,
                command.prompt,
                command.conversation_history.unwrap_or_default(),
                project_root,
            )
            .await
        }
        Commands::InstallSkill(command) => run_install_skill(command),
    }
}

async fn run_legacy(args: LegacyArgs) -> Result<()> {
    if let Some(prompt) = args.enhance_prompt {
        let project_root = env::current_dir()?;
        return run_enhance(
            args.ace,
            args.prompt_ui,
            prompt,
            String::new(),
            project_root,
        )
        .await;
    }

    if args.index_only {
        let project_root = env::current_dir()?;
        return run_index(args.ace, project_root, false).await;
    }

    run_mcp(args.ace, args.prompt_ui, args.transport).await
}

async fn run_mcp(
    ace_args: AceConfigArgs,
    prompt_ui_args: PromptUiArgs,
    transport: TransportArg,
) -> Result<()> {
    let config = new_required_ace_config(&ace_args, &prompt_ui_args)?;

    info!("Starting ace-tool MCP server");

    let server = McpServer::new(config, transport_mode(transport));

    if let Err(e) = server.run().await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

async fn run_index(
    ace_args: AceConfigArgs,
    project_root: PathBuf,
    print_summary: bool,
) -> Result<()> {
    let config = new_required_ace_config(&ace_args, &PromptUiArgs::default())?;

    info!("Index mode: indexing project");
    info!("Project root: {:?}", project_root);

    let manager = IndexManager::new(config, project_root)?;
    let result = manager.index_project().await;
    handle_index_result(result, print_summary)
}

async fn run_search(ace_args: AceConfigArgs, project_root: PathBuf, query: String) -> Result<()> {
    let config = new_required_ace_config(&ace_args, &PromptUiArgs::default())?;
    let tool = SearchContextTool::new(config);
    let result = tool
        .execute(SearchContextArgs {
            project_root_path: Some(project_root.to_string_lossy().to_string()),
            query: Some(query),
        })
        .await;

    if result.text.starts_with("Error:") {
        return Err(anyhow!("{}", result.text));
    }

    println!("{}", result.text);
    Ok(())
}

async fn run_enhance(
    ace_args: AceConfigArgs,
    prompt_ui_args: PromptUiArgs,
    prompt: String,
    conversation_history: String,
    project_root: PathBuf,
) -> Result<()> {
    info!("Enhance mode: enhancing prompt");
    info!("Project root: {:?}", project_root);

    let config = new_enhance_config(&ace_args, &prompt_ui_args)?;
    let enhancer = PromptEnhancer::new(config)?;
    let enhanced = enhancer
        .enhance_simple(&prompt, &conversation_history, Some(&project_root))
        .await?;

    println!("{}", enhanced);
    Ok(())
}

fn handle_index_result(result: IndexResult, print_summary: bool) -> Result<()> {
    match result.status.as_str() {
        "success" => {
            info!("Indexing completed successfully: {}", result.message);
            if let Some(stats) = &result.stats {
                info!(
                    "Stats: {} total blobs, {} existing, {} new",
                    stats.total_blobs, stats.existing_blobs, stats.new_blobs
                );
            }
            if print_summary {
                print_index_summary(&result);
            }
            Ok(())
        }
        "partial" => {
            warn!("Indexing completed with warnings: {}", result.message);
            if let Some(stats) = &result.stats {
                if let Some(failed_batches) = stats.failed_batches {
                    warn!(
                        "Stats: {} total blobs, {} existing, {} new, {} failed batches",
                        stats.total_blobs, stats.existing_blobs, stats.new_blobs, failed_batches
                    );
                } else {
                    warn!(
                        "Stats: {} total blobs, {} existing, {} new",
                        stats.total_blobs, stats.existing_blobs, stats.new_blobs
                    );
                }
            }
            if print_summary {
                print_index_summary(&result);
            }
            std::process::exit(2);
        }
        _ => Err(anyhow!("Indexing failed: {}", result.message)),
    }
}

fn print_index_summary(result: &IndexResult) {
    println!("{}", result.message);
    if let Some(stats) = &result.stats {
        if let Some(failed_batches) = stats.failed_batches {
            println!(
                "total_blobs={} existing_blobs={} new_blobs={} failed_batches={}",
                stats.total_blobs, stats.existing_blobs, stats.new_blobs, failed_batches
            );
        } else {
            println!(
                "total_blobs={} existing_blobs={} new_blobs={}",
                stats.total_blobs, stats.existing_blobs, stats.new_blobs
            );
        }
    }
}

fn new_required_ace_config(
    ace_args: &AceConfigArgs,
    prompt_ui_args: &PromptUiArgs,
) -> Result<Arc<Config>> {
    let credentials = resolve_ace_credentials(ace_args)?;
    let base_url = credentials
        .base_url
        .ok_or_else(|| anyhow!("--base-url, config base_url, or ACE_BASE_URL is required"))?;
    let token = credentials
        .token
        .ok_or_else(|| anyhow!("--token, config token, or ACE_TOKEN is required"))?;

    Config::new(base_url, token, config_options(ace_args, prompt_ui_args))
}

fn new_enhance_config(
    ace_args: &AceConfigArgs,
    prompt_ui_args: &PromptUiArgs,
) -> Result<Arc<Config>> {
    let endpoint = get_enhancer_endpoint();
    if endpoint.is_third_party() {
        let _ = get_third_party_config(endpoint)
            .map_err(|e| anyhow!("Third-party endpoint configuration error: {}", e))?;
        info!("Using third-party endpoint: {}", endpoint);
        let credentials = resolve_ace_credentials(ace_args)?;

        return match (credentials.base_url, credentials.token) {
            (Some(base_url), Some(token)) => {
                info!("Using CLI base_url/token to enable ACE search features");
                Config::new(base_url, token, config_options(ace_args, prompt_ui_args))
            }
            (None, None) => Ok(Config::new_for_third_party_enhancer()),
            _ => Err(anyhow!(
                "--base-url and --token must be provided together in third-party enhance mode"
            )),
        };
    }

    let credentials = resolve_ace_credentials(ace_args)?;
    let base_url = credentials
        .base_url
        .ok_or_else(|| anyhow!("--base-url is required for '{}' endpoint", endpoint))?;
    let token = credentials
        .token
        .ok_or_else(|| anyhow!("--token is required for '{}' endpoint", endpoint))?;

    Config::new(base_url, token, config_options(ace_args, prompt_ui_args))
}

fn resolve_ace_credentials(ace_args: &AceConfigArgs) -> Result<AceCredentials> {
    let file_config = load_file_config(ace_args.config.as_deref())?;

    Ok(AceCredentials {
        base_url: first_non_empty([
            ace_args.base_url.clone(),
            file_config.base_url,
            env::var("ACE_BASE_URL").ok(),
        ]),
        token: first_non_empty([
            ace_args.token.clone(),
            file_config.token,
            env::var("ACE_TOKEN").ok(),
        ]),
    })
}

fn first_non_empty(values: [Option<String>; 3]) -> Option<String> {
    values
        .into_iter()
        .flatten()
        .map(|value| value.trim().to_string())
        .find(|value| !value.is_empty())
}

fn load_file_config(config_path: Option<&Path>) -> Result<FileConfig> {
    let Some(path) = resolve_config_path(config_path)? else {
        return Ok(FileConfig::default());
    };

    if !path.exists() {
        return Ok(FileConfig::default());
    }

    let content =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&content).with_context(|| format!("failed to parse {}", path.display()))
}

fn resolve_config_path(config_path: Option<&Path>) -> Result<Option<PathBuf>> {
    if let Some(path) = config_path {
        return Ok(Some(path.to_path_buf()));
    }

    if let Some(config_home) = env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
        return Ok(Some(
            PathBuf::from(config_home).join(DEFAULT_CONFIG_RELATIVE_PATH),
        ));
    }

    Ok(env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|home| home.join(".config").join(DEFAULT_CONFIG_RELATIVE_PATH)))
}

fn config_options(ace_args: &AceConfigArgs, prompt_ui_args: &PromptUiArgs) -> ConfigOptions {
    ConfigOptions {
        max_lines_per_blob: ace_args.max_lines_per_blob,
        upload_timeout: ace_args.upload_timeout,
        upload_concurrency: ace_args.upload_concurrency,
        retrieval_timeout: ace_args.retrieval_timeout,
        no_adaptive: ace_args.no_adaptive,
        no_webbrowser_enhance_prompt: prompt_ui_args.no_webbrowser_enhance_prompt,
        force_xdg_open: prompt_ui_args.force_xdg_open,
        webui_addr: prompt_ui_args.webui_addr.clone(),
    }
}

fn transport_mode(transport: TransportArg) -> Option<TransportMode> {
    match transport {
        TransportArg::Auto => None,
        TransportArg::Lsp => Some(TransportMode::Lsp),
        TransportArg::Line => Some(TransportMode::Line),
    }
}

fn run_install_skill(command: InstallSkillCommand) -> Result<()> {
    let source = resolve_skill_source(command.source)?;
    validate_skill_source(&source)?;

    let mut agents = Vec::new();
    for agent in command.agents {
        if !agents.contains(&agent) {
            agents.push(agent);
        }
    }

    if agents.is_empty() {
        return Err(anyhow!("--agents must include at least one target"));
    }

    let home = home_dir()?;
    for agent in agents {
        let target = skill_target_dir(agent, &home);
        install_skill_dir(&source, &target, command.force)
            .with_context(|| format!("failed to install {:?} skill", agent))?;
        println!("Installed {:?} skill to {}", agent, target.display());
    }

    Ok(())
}

fn resolve_skill_source(source: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(source) = source {
        return Ok(source);
    }

    let cwd_source = env::current_dir()?.join(REPO_SKILL_PATH);
    if cwd_source.join("SKILL.md").is_file() {
        return Ok(cwd_source);
    }

    let manifest_source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(REPO_SKILL_PATH);
    if manifest_source.join("SKILL.md").is_file() {
        return Ok(manifest_source);
    }

    Err(anyhow!(
        "could not find {}; run from the repository root or pass --source",
        REPO_SKILL_PATH
    ))
}

fn validate_skill_source(source: &Path) -> Result<()> {
    if !source.is_dir() {
        return Err(anyhow!(
            "skill source is not a directory: {}",
            source.display()
        ));
    }

    let skill_md = source.join("SKILL.md");
    if !skill_md.is_file() {
        return Err(anyhow!("skill source is missing {}", skill_md.display()));
    }

    Ok(())
}

fn install_skill_dir(source: &Path, target: &Path, force: bool) -> Result<()> {
    let parent = target
        .parent()
        .ok_or_else(|| anyhow!("target path has no parent: {}", target.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;

    if target.exists() {
        if !force {
            return Err(anyhow!(
                "{} already exists; pass --force to replace it",
                target.display()
            ));
        }
        remove_path(target)?;
    }

    copy_dir_recursive(source, target)
}

fn remove_path(path: &Path) -> Result<()> {
    let metadata =
        fs::symlink_metadata(path).with_context(|| format!("failed to stat {}", path.display()))?;
    if metadata.is_dir() {
        fs::remove_dir_all(path).with_context(|| format!("failed to remove {}", path.display()))?;
    } else {
        fs::remove_file(path).with_context(|| format!("failed to remove {}", path.display()))?;
    }
    Ok(())
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target).with_context(|| format!("failed to create {}", target.display()))?;

    for entry in
        fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry = entry.with_context(|| format!("failed to read {}", source.display()))?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to stat {}", source_path.display()))?;

        if file_type.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &target_path).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    source_path.display(),
                    target_path.display()
                )
            })?;
        }
    }

    Ok(())
}

fn home_dir() -> Result<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("could not determine home directory"))
}

fn skill_target_dir(agent: AgentTarget, home: &Path) -> PathBuf {
    match agent {
        AgentTarget::Codex => home.join(".codex").join("skills").join("ace-tool-rs"),
        AgentTarget::Claude => home.join(".claude").join("skills").join("ace-tool-rs"),
        AgentTarget::Pi => home
            .join(".pi")
            .join("agent")
            .join("skills")
            .join("ace-tool-rs"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use tempfile::TempDir;

    #[test]
    fn help_lists_new_subcommands() {
        let help = Args::command().render_long_help().to_string();
        assert!(help.contains("mcp"));
        assert!(help.contains("index"));
        assert!(help.contains("search"));
        assert!(help.contains("enhance"));
        assert!(help.contains("install-skill"));
    }

    #[test]
    fn command_reports_package_version() {
        let command = Args::command();
        assert_eq!(command.get_version(), Some(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn resolves_credentials_from_config_file() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.toml");
        fs::write(
            &config_path,
            r#"
base_url = "https://config.example.com/"
token = "config-token"
"#,
        )
        .unwrap();

        let credentials = resolve_ace_credentials(&AceConfigArgs {
            config: Some(config_path),
            ..AceConfigArgs::default()
        })
        .unwrap();

        assert_eq!(
            credentials.base_url.as_deref(),
            Some("https://config.example.com/")
        );
        assert_eq!(credentials.token.as_deref(), Some("config-token"));
    }

    #[test]
    fn cli_credentials_override_config_file() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.toml");
        fs::write(
            &config_path,
            r#"
base_url = "https://config.example.com/"
token = "config-token"
"#,
        )
        .unwrap();

        let credentials = resolve_ace_credentials(&AceConfigArgs {
            config: Some(config_path),
            base_url: Some("https://cli.example.com/".to_string()),
            token: Some("cli-token".to_string()),
            ..AceConfigArgs::default()
        })
        .unwrap();

        assert_eq!(
            credentials.base_url.as_deref(),
            Some("https://cli.example.com/")
        );
        assert_eq!(credentials.token.as_deref(), Some("cli-token"));
    }

    #[test]
    fn parses_search_subcommand_with_config_path() {
        let args = Args::try_parse_from([
            "ace-tool-rs",
            "search",
            "--config",
            "/tmp/ace-config.toml",
            "--project-root",
            "/tmp/project",
            "--query",
            "find auth flow",
        ])
        .unwrap();

        match args.command {
            Some(Commands::Search(command)) => {
                assert_eq!(
                    command.ace.config.as_deref(),
                    Some(Path::new("/tmp/ace-config.toml"))
                );
                assert_eq!(command.query, "find auth flow");
            }
            _ => panic!("expected search command"),
        }
    }

    #[test]
    fn parses_legacy_index_mode() {
        let args = Args::try_parse_from([
            "ace-tool-rs",
            "--base-url",
            "https://api.example.com",
            "--token",
            "test-token",
            "--index-only",
        ])
        .unwrap();

        assert!(args.command.is_none());
        assert!(args.legacy.index_only);
        assert_eq!(
            args.legacy.ace.base_url.as_deref(),
            Some("https://api.example.com")
        );
    }

    #[test]
    fn parses_search_subcommand() {
        let args = Args::try_parse_from([
            "ace-tool-rs",
            "search",
            "--base-url",
            "https://api.example.com",
            "--token",
            "test-token",
            "--project-root",
            "/tmp/project",
            "--query",
            "find auth flow",
        ])
        .unwrap();

        match args.command {
            Some(Commands::Search(command)) => {
                assert_eq!(command.query, "find auth flow");
                assert_eq!(
                    command.project_root.as_deref(),
                    Some(Path::new("/tmp/project"))
                );
            }
            _ => panic!("expected search command"),
        }
    }

    #[test]
    fn parses_default_skill_agents() {
        let args = Args::try_parse_from(["ace-tool-rs", "install-skill"]).unwrap();

        match args.command {
            Some(Commands::InstallSkill(command)) => {
                assert_eq!(
                    command.agents,
                    vec![AgentTarget::Codex, AgentTarget::Claude, AgentTarget::Pi]
                );
            }
            _ => panic!("expected install-skill command"),
        }
    }

    #[test]
    fn install_skill_refuses_existing_target_without_force() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("SKILL.md"), "skill").unwrap();
        fs::create_dir_all(&target).unwrap();

        let error = install_skill_dir(&source, &target, false).unwrap_err();
        assert!(error.to_string().contains("already exists"));
    }

    #[test]
    fn install_skill_replaces_existing_target_with_force() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let nested = source.join("agents");
        let target = temp.path().join("target");
        fs::create_dir_all(&nested).unwrap();
        fs::write(source.join("SKILL.md"), "skill").unwrap();
        fs::write(nested.join("openai.yaml"), "interface: {}\n").unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("old.txt"), "old").unwrap();

        install_skill_dir(&source, &target, true).unwrap();

        assert!(target.join("SKILL.md").is_file());
        assert!(target.join("agents").join("openai.yaml").is_file());
        assert!(!target.join("old.txt").exists());
    }
}
