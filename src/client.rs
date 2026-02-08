use crate::crypto;
use crate::protocol::{self, Message, Stage, PROTOCOL_VERSION};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::rustls::{self, pki_types::ServerName, ClientConfig};
use tokio_rustls::TlsConnector;

pub async fn send_message(
    agent_id: &str,
    input_payload: HashMap<String, serde_json::Value>,
    config: &crate::daemon::Config,
) -> Result<Message> {
    let addr = format!("127.0.0.1:{}", config.network.listen_port);

    let connector = create_tls_connector()?;
    let stream = TcpStream::connect(&addr).await?;
    let domain = ServerName::try_from("localhost")
        .map_err(|_| anyhow!("Invalid domain"))?
        .to_owned();
    let mut stream = connector.connect(domain, stream).await?;

    let (my_eph_secret, my_eph_public) = crypto::generate_ephemeral_key();
    let my_id = &config.agent.id;
    let peer_id = agent_id;

    let mut counter = 1u32;
    let timestamp = protocol::current_timestamp();

    let mut knock_payload = HashMap::new();
    if let Some(c) = input_payload.get("c") {
        knock_payload.insert("c".to_string(), c.clone());
    } else {
        knock_payload.insert("c".to_string(), serde_json::json!(1));
    }
    if let Some(pri) = input_payload.get("pri") {
        knock_payload.insert("pri".to_string(), pri.clone());
    } else {
        knock_payload.insert("pri".to_string(), serde_json::json!(2));
    }
    if let Some(prev) = input_payload.get("prev") {
        knock_payload.insert("prev".to_string(), prev.clone());
    }

    let my_eph_bytes: Vec<u8> = my_eph_public.as_bytes().to_vec();
    knock_payload.insert("eph_key".to_string(), serde_json::json!(my_eph_bytes));

    let knock = Message {
        stage: Stage::Knock.to_u8(),
        counter,
        timestamp,
        from: my_id.clone(),
        to: peer_id.to_string(),
        payload: knock_payload,
    };

    let encoded_knock = protocol::encode_message(&knock)?;
    protocol::send_framed_message(&mut stream, &encoded_knock).await?;

    let welcome_bytes = protocol::receive_framed_message(&mut stream).await?;
    let welcome = protocol::decode_message(&welcome_bytes)?;

    if welcome.stage != Stage::Welcome.to_u8() {
        return Err(anyhow!("Expected WELCOME, got stage {}", welcome.stage));
    }

    if welcome.counter <= counter {
        return Err(anyhow!("Invalid counter in WELCOME"));
    }
    counter = welcome.counter;

    let peer_eph_val = welcome
        .payload
        .get("eph_key")
        .ok_or_else(|| anyhow!("Missing eph_key in WELCOME"))?;

    let peer_eph_bytes: Vec<u8> = serde_json::from_value(peer_eph_val.clone())
        .map_err(|_| anyhow!("Invalid eph_key format"))?;

    if peer_eph_bytes.len() != 32 {
        return Err(anyhow!("Invalid eph_key length: {}", peer_eph_bytes.len()));
    }

    let mut peer_eph_array = [0u8; 32];
    peer_eph_array.copy_from_slice(&peer_eph_bytes);

    let mut session_key =
        crypto::derive_session_key(&my_eph_secret, &peer_eph_array, my_id, peer_id)?;

    drop(my_eph_secret);

    let status = welcome
        .payload
        .get("st")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u8;

    if status == 2 {
        counter += 1;
        let thank_payload = build_thank_payload(2, true, None);
        send_encrypted_message(
            &mut stream,
            Stage::Thank,
            &session_key,
            counter,
            my_id,
            peer_id,
            thank_payload,
        )
        .await?;

        crypto::zeroize_key(&mut session_key);
        return Ok(welcome);
    }

    counter += 1;
    let wish_payload = input_payload;

    send_encrypted_message(
        &mut stream,
        Stage::Wish,
        &session_key,
        counter,
        my_id,
        peer_id,
        wish_payload,
    )
    .await?;

    let grant = receive_encrypted_message(&mut stream, &session_key, &mut counter, peer_id, my_id).await?;

    if grant.stage != Stage::Grant.to_u8() {
        return Err(anyhow!("Expected GRANT, got stage {}", grant.stage));
    }

    let grant_status = grant
        .payload
        .get("st")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u8;

    if grant_status == 2 {
        counter += 1;
        let thank_payload = build_thank_payload(2, true, None);
        send_encrypted_message(
            &mut stream,
            Stage::Thank,
            &session_key,
            counter,
            my_id,
            peer_id,
            thank_payload,
        )
        .await?;

        crypto::zeroize_key(&mut session_key);
        return Ok(grant);
    }

    let mut gift: Option<Message> = None;
    loop {
        let msg = receive_encrypted_message(&mut stream, &session_key, &mut counter, peer_id, my_id).await?;
        match Stage::from_u8(msg.stage)? {
            Stage::Wrap => {
                let progress = msg.payload.get("prog").and_then(|v| v.as_u64()).unwrap_or(0);
                eprintln!("Progress: {}%", progress);
            }
            Stage::Gift => {
                gift = Some(msg);
                break;
            }
            _ => {
                return Err(anyhow!("Unexpected stage {} while waiting for GIFT", msg.stage));
            }
        }
    }

    let gift = gift.ok_or_else(|| anyhow!("No GIFT received"))?;

    counter += 1;
    let thank_payload = build_thank_payload(1, false, Some("Thank you!"));
    send_encrypted_message(
        &mut stream,
        Stage::Thank,
        &session_key,
        counter,
        my_id,
        peer_id,
        thank_payload,
    )
    .await?;

    crypto::zeroize_key(&mut session_key);

    Ok(gift)
}

fn create_tls_connector() -> Result<TlsConnector> {
    let mut root_store = rustls::RootCertStore::empty();

    let ca_path = shellexpand::tilde("~/.wish/ca.pem").into_owned();
    if std::path::Path::new(&ca_path).exists() {
        let file = std::fs::File::open(&ca_path)?;
        let mut reader = std::io::BufReader::new(file);
        for cert in rustls_pemfile::certs(&mut reader) {
            root_store.add(cert?)?;
        }
    } else {
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    }

    let client_config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(TlsConnector::from(Arc::new(client_config)))
}

fn build_thank_payload(
    context: u8,
    understanding: bool,
    feedback: Option<&str>,
) -> HashMap<String, serde_json::Value> {
    let mut payload = HashMap::new();
    payload.insert("ctx".to_string(), serde_json::json!(context));
    if context != 1 {
        payload.insert("und".to_string(), serde_json::json!(understanding));
    }
    if let Some(fb) = feedback {
        payload.insert("fb".to_string(), serde_json::json!(fb));
    }
    payload
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
) -> Result<Message>
where
    R: AsyncReadExt + Unpin,
{
    let envelope = protocol::receive_framed_message(reader).await?;

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

    Ok(message)
}
