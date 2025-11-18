use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, TcpStream};
use std::process::{exit, Command};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    LayerChange { new: String },
    LayerNames { names: Vec<String> },
    CurrentLayerInfo { name: String, cfg_text: String },
    ConfigFileReload { new: String },
    CurrentLayerName { name: String },
    MessagePush { message: serde_json::Value },
    Error { msg: String },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status")]
pub enum ServerResponse {
    Ok,
    Error { msg: String },
}

#[derive(Debug, Deserialize)]
struct Config {
    /// Port that kanata's TCP server is listening on
    port: u16,

    /// Path to the script to execute on layer change
    script_path: String,

    /// Log level: "info", "debug", or "trace"
    #[serde(default = "default_log_level")]
    log_level: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn create_default_config(path: &str) -> std::io::Result<Config> {
    let default_config = Config {
        port: 5829,
        script_path: "~/.config/kanata-observer/layer_change.sh".to_string(),
        log_level: "info".to_string(),
    };

    let toml_content = format!(
        r#"# Kanata TCP Client Configuration

# Port that kanata's TCP server is listening on
port = {}

# Path to the script to execute on layer change
# The layer name will be passed as the first argument
script_path = "{}"

# Log level: "info", "debug", or "trace"
log_level = "{}"
"#,
        default_config.port, default_config.script_path, default_config.log_level
    );

    // Create parent directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, toml_content)?;
    eprintln!("Created default config file at: {}", path);
    eprintln!("Please edit it with your desired settings.");

    Ok(default_config)
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[clap(short, long, default_value = "~/.config/kanata-observer/config.toml")]
    config: String,

    /// Port that kanata's TCP server is listening on (overrides config file)
    #[clap(short, long)]
    port: Option<u16>,

    /// Enable debug logging (overrides config file)
    #[clap(short, long)]
    debug: bool,

    /// Enable trace logging (overrides config file)
    #[clap(short, long)]
    trace: bool,
}

fn main() {
    let args = Args::parse();

    // Expand ~ in config path
    let config_path = shellexpand::tilde(&args.config).to_string();

    // Read and parse config file, create default if not found
    let config: Config = match fs::read_to_string(&config_path) {
        Ok(contents) => toml::from_str(&contents).unwrap_or_else(|e| {
            eprintln!("Failed to parse config file {}: {}", config_path, e);
            exit(1);
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Config file doesn't exist, create a default one
            match create_default_config(&config_path) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!(
                        "Failed to create default config file {}: {}",
                        config_path, e
                    );
                    exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read config file {}: {}", config_path, e);
            exit(1);
        }
    };

    // Determine log level (CLI overrides config)
    let log_level = if args.trace {
        simplelog::LevelFilter::Trace
    } else if args.debug {
        simplelog::LevelFilter::Debug
    } else {
        match config.log_level.to_lowercase().as_str() {
            "trace" => simplelog::LevelFilter::Trace,
            "debug" => simplelog::LevelFilter::Debug,
            "info" => simplelog::LevelFilter::Info,
            _ => simplelog::LevelFilter::Info,
        }
    };

    simplelog::TermLogger::init(
        log_level,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .expect("failed to initialize logger");

    // Get port (CLI overrides config)
    let port = args.port.unwrap_or(config.port);

    // Connect with retry logic
    loop {
        log::info!("attempting to connect to kanata on port {}", port);
        match TcpStream::connect_timeout(
            &SocketAddr::from(([127, 0, 0, 1], port)),
            Duration::from_secs(5),
        ) {
            Ok(conn) => {
                log::info!("successfully connected to kanata");
                if let Err(e) = read_from_kanata(conn, &config.script_path) {
                    log::error!("connection lost: {}. retrying in 30 seconds...", e);
                    std::thread::sleep(Duration::from_secs(30));
                }
            }
            Err(e) => {
                log::error!(
                    "failed to connect to kanata: {}. retrying in 30 seconds...",
                    e
                );
                std::thread::sleep(Duration::from_secs(30));
            }
        }
    }
}

fn read_from_kanata(s: TcpStream, script_path: &str) -> std::io::Result<()> {
    log::debug!("reader starting");
    let mut reader = BufReader::new(s);
    let mut msg = String::new();

    // Expand ~ in script path
    let expanded_script_path = shellexpand::tilde(script_path).to_string();

    loop {
        msg.clear();
        let bytes_read = reader.read_line(&mut msg)?;

        // Connection closed
        if bytes_read == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionReset,
                "connection closed by kanata",
            ));
        }

        log::debug!("message received");

        if let Ok(ServerMessage::LayerChange { new }) = serde_json::from_str::<ServerMessage>(&msg)
        {
            log::debug!("Layer changed to: {}", new);
            let out = Command::new(&expanded_script_path).arg(&new).output();
            match out {
                Ok(output) => {
                    if output.status.success() {
                        log::debug!("Script executed successfully");
                    } else {
                        log::error!("Script failed: {}", String::from_utf8_lossy(&output.stderr));
                    }
                }
                Err(e) => log::error!("Failed to execute script: {}", e),
            }
        }
    }
}
