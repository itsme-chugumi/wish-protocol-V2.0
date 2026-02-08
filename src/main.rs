mod client;
mod crypto;
mod daemon;
mod protocol;

use anyhow::Result;
use clap::{Parser, Subcommand};
use daemon::{Config, AgentConfig, NetworkConfig, OpenClawConfig, KeysConfig};
use std::collections::HashMap;
use std::io::Read;

#[derive(Parser)]
#[command(name = "wishp")]
#[command(about = "Wish Protocol v2.0 Daemon and CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Daemon,
    Send {
        agent_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = load_config()?;

    match cli.command {
        Commands::Daemon => {
            daemon::start_server(config).await?;
        }
        Commands::Send { agent_id } => {
            let mut buffer = String::new();
            std::io::stdin().read_to_string(&mut buffer)?;
            let payload: HashMap<String, serde_json::Value> = serde_json::from_str(&buffer)?;

            match client::send_message(&agent_id, payload, &config).await {
                Ok(response) => {
                    println!("{}", serde_json::to_string_pretty(&response.payload)?);
                }
                Err(e) => {
                    eprintln!("Error sending message: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

fn load_config() -> Result<Config> {
    let config_path = shellexpand::tilde("~/.wish/config.toml").into_owned();
    if std::path::Path::new(&config_path).exists() {
        let content = std::fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&content)?;
        return Ok(config);
    }

    Ok(Config {
        agent: AgentConfig {
            id: "test-agent-1".to_string(),
        },
        network: NetworkConfig {
            listen_port: 7779,
        },
        openclaw: OpenClawConfig {
            path: "./mock_openclaw.sh".to_string(),
        },
        keys: KeysConfig {
            private_key_path: "~/.wish/keys/private.key".to_string(),
            public_key_path: "~/.wish/keys/public.key".to_string(),
            keyring_path: "~/.wish/keyring.msgpack".to_string(),
            cert_path: "~/.wish/cert.pem".to_string(),
            key_path: "~/.wish/key.pem".to_string(),
        },
    })
}
