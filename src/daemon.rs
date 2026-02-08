use crate::crypto;
use crate::protocol::{self, Message, Stage, PROTOCOL_VERSION};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::rustls::{
    pki_types::CertificateDer, pki_types::PrivateKeyDer, ServerConfig,
};
use tokio_rustls::TlsAcceptor;

#[derive(serde::Deserialize, Clone)]
pub struct Config {
    pub agent: AgentConfig,
    pub network: NetworkConfig,
    pub openclaw: OpenClawConfig,
    pub keys: KeysConfig,
}

#[derive(serde::Deserialize, Clone)]
pub struct AgentConfig {
    pub id: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct NetworkConfig {
    pub listen_port: u16,
}

#[derive(serde::Deserialize, Clone)]
pub struct OpenClawConfig {
    pub path: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct KeysConfig {
    pub private_key_path: String,
    pub public_key_path: String,
    pub keyring_path: String,
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Clone)]
struct BlocklistEntry {
    agent_id: String,
    reason: BlockReason,
    blocked_at: u64,
    violation_count: u16,
}

#[derive(Clone)]
enum BlockReason {
    Spam = 1,
    MalformedMessages = 2,
    SizeViolations = 3,
    RateLimitViolations = 4,
    SuspiciousBehavior = 5,
    ManualBlock = 6,
}

struct Blocklist {
    entries: HashMap<String, BlocklistEntry>,
}

impl Blocklist {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    fn is_blocked(&self, agent_id: &str) -> bool {
        self.entries.contains_key(agent_id)
    }

    fn add_violation(&mut self, agent_id: &str, reason: BlockReason) {
        let now = protocol::current_timestamp() as u64;

        let entry = self.entries.entry(agent_id.to_string()).or_insert(BlocklistEntry {
            agent_id: agent_id.to_string(),
            reason: reason.clone(),
            blocked_at: 0,
            violation_count: 0,
        });

        entry.violation_count += 1;
        entry.reason = reason.clone();

        let should_block = match reason {
            BlockReason::SizeViolations => entry.violation_count >= 3,
            BlockReason::RateLimitViolations => entry.violation_count >= 10,
            BlockReason::MalformedMessages => entry.violation_count >= 5,
            BlockReason::ManualBlock => true,
            _ => entry.violation_count >= 5,
        };

        if should_block && entry.blocked_at == 0 {
            entry.blocked_at = now;
        }
    }

    fn block(&mut self, agent_id: &str, reason: BlockReason) {
        let now = protocol::current_timestamp() as u64;
        self.entries.insert(agent_id.to_string(), BlocklistEntry {
            agent_id: agent_id.to_string(),
            reason,
            blocked_at: now,
            violation_count: 1,
        });
    }
}

struct RateLimiter {
    knocks_per_hour: HashMap<String, (u32, u64)>,
    bytes_per_hour: HashMap<String, (u64, u64)>,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            knocks_per_hour: HashMap::new(),
            bytes_per_hour: HashMap::new(),
        }
    }

    fn check_knock(&mut self, agent_id: &str) -> Result<()> {
        let now = protocol::current_timestamp() as u64;
        let hour_ago = now.saturating_sub(3600);

        let (count, reset_time) = self.knocks_per_hour
            .entry(agent_id.to_string())
            .or_insert((0, now));

        if *reset_time < hour_ago {
            *count = 0;
            *reset_time = now;
        }

        if *count >= 100 {
            return Err(anyhow!("Rate limit exceeded: max 100 KNOCK per hour"));
        }

        *count += 1;
        Ok(())
    }

    fn check_bytes(&mut self, agent_id: &str, bytes: u64) -> Result<()> {
        let now = protocol::current_timestamp() as u64;
        let hour_ago = now.saturating_sub(3600);

        let (total_bytes, reset_time) = self.bytes_per_hour
            .entry(agent_id.to_string())
            .or_insert((0, now));

        if *reset_time < hour_ago {
            *total_bytes = 0;
            *reset_time = now;
        }

        if *total_bytes + bytes > 100 * 1024 * 1024 {
            return Err(anyhow!("Rate limit exceeded: max 100 MB per hour"));
        }

        *total_bytes += bytes;
        Ok(())
    }
}

pub async fn start_server(config: Config) -> Result<()> {
    let addr = format!("0.0.0.0:{}", config.network.listen_port);
    let certs = load_certs(&config.keys.cert_path)?;
    let key = load_key(&config.keys.key_path)?;

    let config_arc = Arc::new(config);
    let blocklist = Arc::new(Mutex::new(Blocklist::new()));
    let rate_limiter = Arc::new(Mutex::new(RateLimiter::new()));

    let server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| anyhow!("TLS error: {}", e))?;
    let acceptor = TlsAcceptor::from(Arc::new(server_config));

    let listener = TcpListener::bind(&addr).await?;
    println!("Wish Protocol daemon listening on {}", addr);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let config_clone = config_arc.clone();
        let blocklist_clone = blocklist.clone();
        let rate_limiter_clone = rate_limiter.clone();

        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(mut tls_stream) => {
                    if let Err(e) = handle_connection(
                        &mut tls_stream,
                        &config_clone,
                        blocklist_clone,
                        rate_limiter_clone,
                    ).await {
                        eprintln!("Error handling connection from {}: {}", peer_addr, e);
                    }
                }
                Err(e) => eprintln!("TLS accept error from {}: {}", peer_addr, e),
            }
        });
    }
}

fn load_certs(path: &str) -> Result<Vec<CertificateDer<'static>>> {
    let path = shellexpand::tilde(path).into_owned();
    let file = std::fs::File::open(&path)?;
    let mut reader = std::io::BufReader::new(file);
    rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow!("Error loading certs: {}", e))
}

fn load_key(path: &str) -> Result<PrivateKeyDer<'static>> {
    let path = shellexpand::tilde(path).into_owned();
    let file = std::fs::File::open(&path)?;
    let mut reader = std::io::BufReader::new(file);
    rustls_pemfile::private_key(&mut reader)
        .map(|opt| opt.ok_or(anyhow!("No private key found")))
        .map_err(|e| anyhow!("Error loading key: {}", e))?
}

async fn handle_connection<S>(
    stream: &mut S,
    config: &Config,
    blocklist: Arc<Mutex<Blocklist>>,
    rate_limiter: Arc<Mutex<RateLimiter>>,
) -> Result<()>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    let my_id = &config.agent.id;

    let knock_bytes = protocol::receive_framed_message(stream).await?;

    if let Err(e) = protocol::validate_size(Stage::Knock.to_u8(), knock_bytes.len()) {
        return Err(e);
    }

    let knock = protocol::decode_message(&knock_bytes)?;

    if knock.stage != Stage::Knock.to_u8() {
        return Err(anyhow!("Expected KNOCK, got stage {}", knock.stage));
    }

    let peer_id = &knock.from;

    {
        let blocklist = blocklist.lock().unwrap();
        if blocklist.is_blocked(peer_id) {
            return Err(anyhow!("Agent {} is blocked", peer_id));
        }
    }

    {
        let mut rate_limiter = rate_limiter.lock().unwrap();
        if let Err(e) = rate_limiter.check_knock(peer_id) {
            let mut blocklist = blocklist.lock().unwrap();
            blocklist.add_violation(peer_id, BlockReason::RateLimitViolations);
            return Err(e);
        }

        if let Err(e) = rate_limiter.check_bytes(peer_id, knock_bytes.len() as u64) {
            let mut blocklist = blocklist.lock().unwrap();
            blocklist.add_violation(peer_id, BlockReason::RateLimitViolations);
            return Err(e);
        }
    }

    let mut counter = knock.counter;

    let peer_eph_val = knock
        .payload
        .get("eph_key")
        .ok_or_else(|| anyhow!("Missing eph_key in KNOCK"))?;

    let peer_eph_bytes: Vec<u8> = serde_json::from_value(peer_eph_val.clone())
        .map_err(|_| anyhow!("Invalid eph_key format"))?;

    if peer_eph_bytes.len() != 32 {
        return Err(anyhow!("Invalid eph_key length"));
    }

    let mut peer_eph_array = [0u8; 32];
    peer_eph_array.copy_from_slice(&peer_eph_bytes);

    let (my_eph_secret, my_eph_public) = crypto::generate_ephemeral_key();

    let mut session_key =
        crypto::derive_session_key(&my_eph_secret, &peer_eph_array, peer_id, my_id)?;

    drop(my_eph_secret);

    let knock_decision = call_openclaw(&config.openclaw.path, &knock)?;
    let should_accept = knock_decision
        .get("accept")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    counter += 1;
    let timestamp = protocol::current_timestamp();

    let mut welcome_payload = HashMap::new();
    welcome_payload.insert(
        "eph_key".to_string(),
        serde_json::json!(my_eph_public.as_bytes().to_vec()),
    );

    if should_accept {
        welcome_payload.insert("st".to_string(), serde_json::json!(1));
        welcome_payload.insert("msg".to_string(), serde_json::json!("Welcome! Please share your wish."));
    } else {
        welcome_payload.insert("st".to_string(), serde_json::json!(2));
        let reason = knock_decision
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("busy");
        welcome_payload.insert("r".to_string(), serde_json::json!(reason));
    }

    let welcome = Message {
        stage: Stage::Welcome.to_u8(),
        counter,
        timestamp,
        from: my_id.clone(),
        to: peer_id.clone(),
        payload: welcome_payload,
    };

    let encoded_welcome = protocol::encode_message(&welcome)?;
    protocol::send_framed_message(stream, &encoded_welcome).await?;

    if !should_accept {
        let _ = receive_encrypted_message(stream, &session_key, &mut counter, peer_id, my_id).await;
        crypto::zeroize_key(&mut session_key);
        return Ok(());
    }

    let (wish, wish_size) = receive_encrypted_message(stream, &session_key, &mut counter, peer_id, my_id).await?;

    {
        let mut rate_limiter = rate_limiter.lock().unwrap();
        if let Err(e) = rate_limiter.check_bytes(peer_id, wish_size as u64) {
            let mut blocklist = blocklist.lock().unwrap();
            blocklist.add_violation(peer_id, BlockReason::RateLimitViolations);
            crypto::zeroize_key(&mut session_key);
            return Err(e);
        }
    }

    if wish.stage != Stage::Wish.to_u8() {
        return Err(anyhow!("Expected WISH, got stage {}", wish.stage));
    }

    let task_decision = call_openclaw(&config.openclaw.path, &wish)?;
    let should_grant = task_decision
        .get("accept")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    counter += 1;
    let mut grant_payload = HashMap::new();

    if should_grant {
        grant_payload.insert("st".to_string(), serde_json::json!(1));
        let est_time = task_decision
            .get("estimated_time")
            .and_then(|v| v.as_u64())
            .unwrap_or(60);
        grant_payload.insert("est_t".to_string(), serde_json::json!(est_time));
    } else {
        grant_payload.insert("st".to_string(), serde_json::json!(2));
        let reason = task_decision
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("excessive_request");
        grant_payload.insert("r".to_string(), serde_json::json!(reason));
    }

    send_encrypted_message(
        stream,
        Stage::Grant,
        &session_key,
        counter,
        my_id,
        peer_id,
    grant_payload,
    )
    .await?;

    if !should_grant {
        let _ = receive_encrypted_message(stream, &session_key, &mut counter, peer_id, my_id).await;
        crypto::zeroize_key(&mut session_key);
        return Ok(());
    }

    let task_result = call_openclaw(&config.openclaw.path, &wish)?;

    counter += 1;
    let mut gift_payload = HashMap::new();
    gift_payload.insert("ok".to_string(), serde_json::json!(true));
    gift_payload.insert("res".to_string(), serde_json::json!(task_result));

    let mut meta = HashMap::new();
    meta.insert("exec_t", serde_json::json!(1));
    gift_payload.insert("meta".to_string(), serde_json::json!(meta));

    send_encrypted_message(
        stream,
        Stage::Gift,
        &session_key,
        counter,
        my_id,
        peer_id,
        gift_payload,
    )
    .await?;

    let (thank, thank_size) = receive_encrypted_message(stream, &session_key, &mut counter, peer_id, my_id).await?;

    {
        let mut rate_limiter = rate_limiter.lock().unwrap();
        let _ = rate_limiter.check_bytes(peer_id, thank_size as u64);
    }

    if thank.stage != Stage::Thank.to_u8() {
        eprintln!("Warning: Expected THANK, got stage {}", thank.stage);
    }

    crypto::zeroize_key(&mut session_key);

    Ok(())
}

fn call_openclaw(path: &str, message: &Message) -> Result<HashMap<String, serde_json::Value>> {
    let input_json = serde_json::to_string(message)?;

    let mut child = Command::new(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    {
        let stdin = child.stdin.as_mut().ok_or_else(|| anyhow!("Failed to open stdin"))?;
        stdin.write_all(input_json.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("OpenClaw failed: {}", stderr));
    }

    let output_str = String::from_utf8(output.stdout)?;

    if output_str.trim().is_empty() {
        let mut result = HashMap::new();
        result.insert("accept".to_string(), serde_json::json!(true));
        return Ok(result);
    }

    serde_json::from_str(&output_str)
        .map_err(|e| anyhow!("Failed to parse OpenClaw response: {}", e))
}

async fn send_encrypted_message<W>(
    writer: &mut W,
    stage: Stage,
    session_key: &[u8; 32],
    counter: u32,
    from: &str,
    to: &str,
    payload: HashMap<String, serde_json::Value>,
) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let timestamp = protocol::current_timestamp();

    let message = Message {
        stage: stage.to_u8(),
        counter,
        timestamp,
        from: from.to_string(),
        to: to.to_string(),
        payload,
    };

    let plaintext = protocol::encode_message(&message)?;
    let aad = protocol::build_aad(PROTOCOL_VERSION, from, to);

    let encrypted = crypto::encrypt_message(session_key, counter, timestamp, &plaintext, &aad)?;

    let mut envelope = Vec::new();
    envelope.extend(counter.to_be_bytes());
    envelope.extend(timestamp.to_be_bytes());
    envelope.extend(encrypted);

    protocol::send_framed_message(writer, &envelope).await?;
    Ok(())
}

async fn receive_encrypted_message<R>(
    reader: &mut R,
    session_key: &[u8; 32],
    last_counter: &mut u32,
    expected_from: &str,
    expected_to: &str,
) -> Result<(Message, usize)>
where
    R: AsyncReadExt + Unpin,
{
    let envelope = protocol::receive_framed_message(reader).await?;
    let envelope_size = envelope.len();

    if envelope.len() < 8 {
        return Err(anyhow!("Envelope too short: {} bytes", envelope.len()));
    }

    let remote_counter = u32::from_be_bytes(envelope[0..4].try_into()?);
    let remote_timestamp = u32::from_be_bytes(envelope[4..8].try_into()?);
    let ciphertext = &envelope[8..];

    if remote_counter <= *last_counter {
        return Err(anyhow!(
            "Replay attack detected: counter {} <= {}",
            remote_counter,
            *last_counter
        ));
    }
    *last_counter = remote_counter;

    let aad = protocol::build_aad(PROTOCOL_VERSION, expected_from, expected_to);
    let plaintext =
        crypto::decrypt_message(session_key, remote_counter, remote_timestamp, ciphertext, &aad)?;

    let message: Message = protocol::decode_message(&plaintext)?;

    protocol::validate_size(message.stage, plaintext.len())?;

    if message.from != expected_from {
        return Err(anyhow!(
            "Message from mismatch: expected {}, got {}",
            expected_from,
            message.from
        ));
    }
    if message.to != expected_to {
        return Err(anyhow!(
            "Message to mismatch: expected {}, got {}",
            expected_to,
            message.to
        ));
    }

    Ok((message, envelope_size))
}
