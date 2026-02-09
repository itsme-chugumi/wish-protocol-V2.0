mod client;
mod crypto;
mod daemon;
mod keyring;
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
    Keygen,
    AddPeer {
        agent_id: String,
        public_key: String,
    },
    ListPeers,
    Gencert,
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
        Commands::Keygen => {
            handle_keygen()?;
        }
        Commands::AddPeer { agent_id, public_key } => {
            handle_add_peer(agent_id, public_key)?;
        }
        Commands::ListPeers => {
            handle_list_peers()?;
        }
        Commands::Gencert => {
            handle_gencert()?;
        }
    }

    Ok(())
}

fn load_config() -> Result<Config> {
    let config_path = shellexpand::tilde("~/.wish-protocol/config.toml").into_owned();
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
            private_key_path: "~/.wish-protocol/keys/private.key".to_string(),
            public_key_path: "~/.wish-protocol/keys/public.key".to_string(),
            keyring_path: "~/.wish-protocol/keyring.msgpack".to_string(),
            cert_path: "~/.wish-protocol/cert.pem".to_string(),
            key_path: "~/.wish-protocol/key.pem".to_string(),
        },
    })
}

fn handle_keygen() -> Result<()> {
    use x25519_dalek::{StaticSecret, PublicKey};
    use rand::rngs::OsRng;
    use sha2::{Sha256, Digest};
    use hex;

    println!("Generating keypair...");

    let private = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&private);

    let keys_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?
        .join(".wish-protocol/keys");
    std::fs::create_dir_all(&keys_dir)?;

    std::fs::write(keys_dir.join("private.key"), private.to_bytes())?;
    std::fs::write(keys_dir.join("public.key"), public.as_bytes())?;

    let mut hasher = Sha256::new();
    hasher.update(public.as_bytes());
    let hash = hasher.finalize();
    let fingerprint = hex::encode(&hash[..4]);

    println!("✓ Keys saved to ~/.wish-protocol/keys/");
    println!();
    println!("Your agent ID: yourname-{}", fingerprint);
    println!("Example: nono-{}", fingerprint);
    println!();
    println!("Next: Add to ~/.wish-protocol/config.toml:");
    println!("[agent]");
    println!("id = \"yourname-{}\"", fingerprint);
    println!();
    println!("Share your public key with peers:");
    println!("Public key: {}", hex::encode(public.as_bytes()));

    Ok(())
}

fn handle_add_peer(agent_id: String, public_key: String) -> Result<()> {
    use hex;
    use base64::{Engine as _, engine::general_purpose};

    let key_bytes = if public_key.len() == 64 {
        hex::decode(&public_key)?
    } else {
        general_purpose::STANDARD.decode(&public_key)?
    };

    if key_bytes.len() != 32 {
        return Err(anyhow::anyhow!("Invalid key length: {}", key_bytes.len()));
    }

    let mut key_array = [0u8; 32];
    key_array.copy_from_slice(&key_bytes);

    let keyring_path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?
        .join(".wish-protocol/keyring.msgpack");

    let mut keyring = keyring::Keyring::load(keyring_path)?;
    keyring.add(agent_id.clone(), key_array)?;

    println!("✓ Added peer: {}", agent_id);

    Ok(())
}

fn handle_list_peers() -> Result<()> {
    let keyring_path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?
        .join(".wish-protocol/keyring.msgpack");

    let keyring = keyring::Keyring::load(keyring_path)?;
    let entries = keyring.list();

    if entries.is_empty() {
        println!("No peers in keyring.");
        println!("Add peers with: wishp add-peer <agent-id> <public-key>");
    } else {
        println!("Known peers:");
        for entry in entries {
            use chrono::{DateTime, Utc};
            let dt = DateTime::<Utc>::from_timestamp(entry.added_at as i64, 0)
                .unwrap_or_else(|| Utc::now());
            println!("  {} (added: {})", entry.agent_id, dt.format("%Y-%m-%d %H:%M:%S"));
        }
    }

    Ok(())
}

fn handle_gencert() -> Result<()> {
    use rcgen::generate_simple_self_signed;

    println!("Generating self-signed certificate...");

    let subject_alt_names = vec!["localhost".to_string()];
    let certified_key = generate_simple_self_signed(subject_alt_names)
        .map_err(|e| anyhow::anyhow!("Failed to generate certificate: {}", e))?;

    let cert_pem = certified_key.cert.pem();
    let key_pem = certified_key.key_pair.serialize_pem();

    let cert_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?
        .join(".wish-protocol");
    std::fs::create_dir_all(&cert_dir)?;

    std::fs::write(cert_dir.join("cert.pem"), cert_pem)?;
    std::fs::write(cert_dir.join("key.pem"), key_pem)?;

    println!("✓ Certificate saved to ~/.wish-protocol/cert.pem");
    println!("✓ Private key saved to ~/.wish-protocol/key.pem");

    Ok(())
}
