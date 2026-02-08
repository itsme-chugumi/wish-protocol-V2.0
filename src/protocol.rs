use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const PROTOCOL_VERSION: u8 = 2;

pub const MAX_KNOCK_SIZE: usize = 2 * 1024;
pub const MAX_WELCOME_SIZE: usize = 2 * 1024;
pub const MAX_WISH_SIZE: usize = 200 * 1024;
pub const MAX_GRANT_SIZE: usize = 20 * 1024;
pub const MAX_WRAP_SIZE: usize = 2 * 1024;
pub const MAX_GIFT_SIZE: usize = 20 * 1024 * 1024;
pub const MAX_THANK_SIZE: usize = 4 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub stage: u8,
    pub counter: u32,
    pub timestamp: u32,
    pub from: String,
    pub to: String,
    pub payload: HashMap<String, serde_json::Value>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Stage {
    Knock = 1,
    Welcome = 2,
    Wish = 3,
    Grant = 4,
    Wrap = 5,
    Gift = 6,
    Thank = 7,
    Error = 255,
}

impl Stage {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Stage::Knock),
            2 => Ok(Stage::Welcome),
            3 => Ok(Stage::Wish),
            4 => Ok(Stage::Grant),
            5 => Ok(Stage::Wrap),
            6 => Ok(Stage::Gift),
            7 => Ok(Stage::Thank),
            255 => Ok(Stage::Error),
            _ => Err(anyhow!("Invalid stage: {}", value)),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Stage::Knock => 1,
            Stage::Welcome => 2,
            Stage::Wish => 3,
            Stage::Grant => 4,
            Stage::Wrap => 5,
            Stage::Gift => 6,
            Stage::Thank => 7,
            Stage::Error => 255,
        }
    }

    pub fn max_size(&self) -> usize {
        match self {
            Stage::Knock => MAX_KNOCK_SIZE,
            Stage::Welcome => MAX_WELCOME_SIZE,
            Stage::Wish => MAX_WISH_SIZE,
            Stage::Grant => MAX_GRANT_SIZE,
            Stage::Wrap => MAX_WRAP_SIZE,
            Stage::Gift => MAX_GIFT_SIZE,
            Stage::Thank => MAX_THANK_SIZE,
            Stage::Error => MAX_THANK_SIZE,
        }
    }
}

pub fn encode_message(message: &Message) -> Result<Vec<u8>> {
    rmp_serde::to_vec(message).map_err(|e| anyhow!("Serialization failed: {}", e))
}

pub fn decode_message(bytes: &[u8]) -> Result<Message> {
    rmp_serde::from_slice(bytes).map_err(|e| anyhow!("Deserialization failed: {}", e))
}

pub async fn send_framed_message<W>(writer: &mut W, data: &[u8]) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let total_len = (data.len() + 1) as u32;
    writer.write_all(&total_len.to_be_bytes()).await?;
    writer.write_all(&[PROTOCOL_VERSION]).await?;
    writer.write_all(data).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn receive_framed_message<R>(reader: &mut R) -> Result<Vec<u8>>
where
    R: AsyncReadExt + Unpin,
{
    let mut len_bytes = [0u8; 4];
    reader.read_exact(&mut len_bytes).await?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    if len > MAX_GIFT_SIZE + 1 {
        return Err(anyhow!("Message too large: {} bytes (max {})", len, MAX_GIFT_SIZE + 1));
    }

    let mut version = [0u8; 1];
    reader.read_exact(&mut version).await?;
    if version[0] != PROTOCOL_VERSION {
        return Err(anyhow!("Invalid protocol version: {} (expected {})", version[0], PROTOCOL_VERSION));
    }

    let mut buf = vec![0u8; len - 1];
    reader.read_exact(&mut buf).await?;
    Ok(buf)
}

pub fn build_aad(version: u8, from: &str, to: &str) -> Vec<u8> {
    let mut aad = vec![version];
    aad.extend(from.as_bytes());
    aad.extend(to.as_bytes());
    aad
}

pub fn current_timestamp() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as u32)
        .unwrap_or(0)
}

pub fn validate_size(stage: u8, size: usize) -> Result<()> {
    let limit = match stage {
        1 => MAX_KNOCK_SIZE,
        2 => MAX_WELCOME_SIZE,
        3 => MAX_WISH_SIZE,
        4 => MAX_GRANT_SIZE,
        5 => MAX_WRAP_SIZE,
        6 => MAX_GIFT_SIZE,
        7 => MAX_THANK_SIZE,
        _ => return Err(anyhow!("Unknown stage: {}", stage)),
    };

    if size > limit {
        return Err(anyhow!("Message too large for stage {}: {} bytes (max {})", stage, size, limit));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let mut payload = HashMap::new();
        payload.insert("key".to_string(), serde_json::json!("value"));

        let message = Message {
            stage: Stage::Knock.to_u8(),
            counter: 1,
            timestamp: 1678886400,
            from: "alice-12345678".to_string(),
            to: "bob-87654321".to_string(),
            payload,
        };

        let encoded = encode_message(&message).unwrap();
        let decoded = decode_message(&encoded).unwrap();

        assert_eq!(message, decoded);
    }

    #[test]
    fn test_stage_from_u8() {
        assert_eq!(Stage::from_u8(1).unwrap(), Stage::Knock);
        assert_eq!(Stage::from_u8(7).unwrap(), Stage::Thank);
        assert_eq!(Stage::from_u8(255).unwrap(), Stage::Error);
        assert!(Stage::from_u8(100).is_err());
    }

    #[test]
    fn test_build_aad() {
        let aad = build_aad(2, "alice", "bob");
        assert_eq!(aad, b"\x02alicebob".to_vec());
    }

    #[tokio::test]
    async fn test_framed_message() {
        use tokio::io::duplex;

        let (mut client, mut server) = duplex(1024);

        let original_data = b"Hello, Wish Protocol!";

        let send_handle = tokio::spawn(async move {
            send_framed_message(&mut client, original_data).await.unwrap();
            client
        });

        let received = receive_framed_message(&mut server).await.unwrap();
        assert_eq!(received, original_data.to_vec());

        send_handle.await.unwrap();
    }
}
