# Wish Protocol Specification v2.0

**Secure Peer-to-Peer Agent Communication Protocol**  
_"Grant wishes, share gifts, with courtesy and respect."_

**Document Version:** 2.0  
**Date:** 2025-02-08  
**Status:** Draft Specification

---

## Table of Contents

1. [Introduction](https://claude.ai/chat/7fc09495-7287-40d9-8ebb-99b6067fcfe4#1-introduction)
2. [Protocol Basics](https://claude.ai/chat/7fc09495-7287-40d9-8ebb-99b6067fcfe4#2-protocol-basics)
3. [Transport Security](https://claude.ai/chat/7fc09495-7287-40d9-8ebb-99b6067fcfe4#3-transport-security)
4. [Connection Establishment](https://claude.ai/chat/7fc09495-7287-40d9-8ebb-99b6067fcfe4#4-connection-establishment)
5. [Protocol Flow](https://claude.ai/chat/7fc09495-7287-40d9-8ebb-99b6067fcfe4#5-protocol-flow)
6. [Binary Message Format](https://claude.ai/chat/7fc09495-7287-40d9-8ebb-99b6067fcfe4#6-binary-message-format)
7. [Stage Specifications](https://claude.ai/chat/7fc09495-7287-40d9-8ebb-99b6067fcfe4#7-stage-specifications)
8. [Negotiation Protocol](https://claude.ai/chat/7fc09495-7287-40d9-8ebb-99b6067fcfe4#8-negotiation-protocol)
9. [Rejection Handling](https://claude.ai/chat/7fc09495-7287-40d9-8ebb-99b6067fcfe4#9-rejection-handling)
10. [Error Handling](https://claude.ai/chat/7fc09495-7287-40d9-8ebb-99b6067fcfe4#10-error-handling)
11. [Encryption and Security](https://claude.ai/chat/7fc09495-7287-40d9-8ebb-99b6067fcfe4#11-encryption-and-security)
12. [Blocklist Management](https://claude.ai/chat/7fc09495-7287-40d9-8ebb-99b6067fcfe4#12-blocklist-management)
13. [Rate Limiting and DoS Protection](https://claude.ai/chat/7fc09495-7287-40d9-8ebb-99b6067fcfe4#13-rate-limiting-and-dos-protection)
14. [Rendezvous Server Protocol](https://claude.ai/chat/7fc09495-7287-40d9-8ebb-99b6067fcfe4#14-rendezvous-server-protocol)
15. [Implementation Guide](https://claude.ai/chat/7fc09495-7287-40d9-8ebb-99b6067fcfe4#15-implementation-guide)

---

## 1. Introduction

### 1.1 Overview

Wish Protocol v2.0 is a lightweight, secure, peer-to-peer communication protocol designed for AI agent-to-agent interaction. It emphasizes consent-based communication, end-to-end encryption with forward secrecy, binary efficiency, and direct P2P connections.

### 1.2 What's New in v2.0

**Major Changes from v1.0:**

- **Peer-to-Peer Architecture**: Direct agent-to-agent connections, no mandatory central server
- **Binary Protocol**: MessagePack encoding for 50-70% size reduction
- **Forward Secrecy**: Ephemeral session keys protect past messages
- **Automatic Blocklist**: Agents can automatically block malicious actors
- **Replay Protection**: Message counters prevent replay attacks
- **Rendezvous Support**: Optional lightweight server for NAT traversal

### 1.3 Design Philosophy

- **Consent-based**: No action without explicit agreement
- **Ephemeral**: Messages are read once and deleted
- **Encrypted**: End-to-end encryption with forward secrecy
- **Peer-to-Peer**: Direct connections, minimal infrastructure
- **Courteous**: Every interaction ends with acknowledgment
- **Efficient**: Binary protocol for minimal bandwidth
- **Self-defending**: Automatic protection against malicious agents

### 1.4 Key Features

- 7-stage interaction protocol
- End-to-end encryption with forward secrecy (X25519 + AES-256-GCM)
- MessagePack binary encoding
- P2P direct connections
- Optional rendezvous server for NAT traversal
- No message caching or storage
- Built-in negotiation mechanism
- Graceful rejection handling
- Automatic blocklist management
- Task exchange via structured data
- Document reference system for large data
- Special flows: tip sharing, knowledge transfer, document sharing

### 1.5 Design Goals

**DO:**

- Ensure both parties consent to communication
- Encrypt all message content end-to-end
- Delete messages after reading
- Connect directly peer-to-peer
- Handle rejections gracefully
- Enable negotiation and bargaining
- Protect against malicious agents automatically

**DO NOT:**

- Store messages on servers
- Cache conversation history
- Proceed without consent
- Leave conversations unfinished
- Trust unknown agents without verification

---

## 2. Protocol Basics

### 2.1 URL Schemes

Wish Protocol v2.0 supports multiple connection methods:

**Direct Connection (IP/Hostname):**

```
wish://[agent-id]@[host]:[port]/
```

**Via Rendezvous Server:**

```
wish://[agent-id]@rdv:[rendezvous-server]:[port]/
```

**Components:**

- `wish://` - Protocol identifier
- `agent-id` - Unique agent identifier
- `host` - Direct IP address or hostname
- `rdv:` - Indicates rendezvous server mode
- `rendezvous-server` - Rendezvous server address
- `port` - Port number (optional, default: 7779)

**Examples:**

```
wish://churi@192.168.1.100:7779/
wish://nono@agent.example.com/
wish://alice@rdv:rendezvous.example.com/
```

### 2.2 Default Port

**7779** - Wish Protocol over TLS

### 2.3 Transport Layer

**Direct P2P Connection:**

- TCP connection with mandatory TLS 1.3 or higher
- Direct agent-to-agent connection
- Ephemeral connections (close after conversation)
- No persistent connections

**Via Rendezvous:**

- Initial connection to rendezvous server
- NAT traversal assistance
- Upgrade to direct P2P connection
- Rendezvous server disconnects after connection established

### 2.4 Agent Identity

**Agent ID Format:**

```
[name]-[public-key-fingerprint-first-8-chars]

Examples:
nono-a3f28c91
churi-7b9e4d2a
```

**Requirements:**

- Name: 1-32 characters, alphanumeric and hyphen only
- Fingerprint: First 8 characters of SHA-256 hash of public key
- Total length: 3-41 characters

**Purpose:**

- Globally unique identifier
- Prevents agent impersonation
- Easy to verify identity

---

## 3. Transport Security

### 3.1 TLS Requirement

All Wish Protocol connections MUST use TLS 1.3 or higher for transport encryption.

**TLS Configuration:**

- Minimum: TLS 1.3
- Cipher suites: Modern AEAD ciphers only
- Certificate: Self-signed acceptable for P2P

### 3.2 Encryption Layers

Wish Protocol uses **dual encryption**:

```
Layer 1: TLS 1.3 (transport security)
         ↓
Layer 2: Session-based E2E encryption (message security)
         - Key Exchange: X25519 ECDH
         - Encryption: AES-256-GCM
         - Forward Secrecy: Yes
```

### 3.3 Key Types

**Long-term Keys (Identity Keys):**

- Algorithm: X25519
- Purpose: Identity and initial key exchange
- Storage: Persistent, carefully protected
- Distribution: Out-of-band pre-exchange

**Ephemeral Session Keys:**

- Algorithm: AES-256-GCM
- Purpose: Message encryption within conversation
- Lifetime: Single conversation only
- Destroyed: After conversation ends

### 3.4 Pre-Exchange of Long-term Keys

**Agents MUST exchange long-term public keys before communication.**

Communication is only possible between agents who have already exchanged public keys. There is no mechanism for key exchange during the protocol flow.

**Pre-Exchange Methods:**

Agents must use out-of-band methods to exchange public keys:

- Manual configuration file
- QR code scanning
- Direct file transfer
- Physical media (USB drive)
- Encrypted email
- Secure messaging app
- Trusted intermediary
- Public key server (if mutually trusted)

**Key Distribution Format:**

```json
{
  "agent_id": "nono-a3f28c91",
  "public_key": "base64-encoded-X25519-public-key",
  "algorithm": "X25519",
  "created": "2025-02-08T10:00:00Z",
  "fingerprint": "sha256:a3f28c914e5b7d2a8f6c1b9e4d2a8f6c..."
}
```

**Verification:**

Before adding a new agent's public key:

1. Obtain key through trusted channel
2. Verify fingerprint through second channel (optional but recommended)
3. Store in local keyring
4. Only then can communication begin

**No First-Contact Exchange:**

The protocol does NOT support:

- Sending public keys during KNOCK/WELCOME
- Trust-on-first-use (TOFU)
- Automatic key exchange
- Opportunistic encryption

**All long-term public keys must be known before initiating KNOCK.**

### 3.5 Forward Secrecy

**How Forward Secrecy Works:**

1. **Session Key Generation (KNOCK stage):**
    
    - Requester generates ephemeral key pair
    - Performs ECDH with responder's long-term public key
    - Derives session key: `session_key = HKDF(ECDH_result)`
2. **Session Key Exchange (WELCOME stage):**
    
    - Responder generates ephemeral key pair
    - Performs ECDH with requester's ephemeral public key
    - Both parties now have same session key
3. **Message Encryption:**
    
    - All messages encrypted with session key (AES-256-GCM)
    - Much faster than public-key crypto
4. **Session Key Destruction:**
    
    - After THANK, both parties delete session key
    - Past messages cannot be decrypted even if long-term keys compromised

**Benefits:**

- Long-term key compromise doesn't expose past conversations
- Faster encryption (symmetric vs asymmetric)
- Each conversation has unique session key

**Note on Message Storage:**

- Users MAY store decrypted plaintext messages locally
- Forward secrecy protects intercepted encrypted messages
- It doesn't prevent users from keeping their own messages

### 3.6 No Caching Policy

**Agents MUST:**

- Delete messages after reading
- Not store conversation history (unless explicitly enabled by user)
- Overwrite message memory
- Destroy session keys after conversation

**Rendezvous Servers MUST:**

- Never see message content (only encrypted)
- Delete connection metadata after P2P established
- Not log any payload data

---

## 4. Connection Establishment

### 4.1 Direct P2P Connection

**Process:**

1. **Resolve Address:**
    
    ```
    wish://churi-7b9e4d2a@192.168.1.100:7779/
    → Connect to 192.168.1.100:7779
    ```
    
2. **TLS Handshake:**
    
    ```
    Client → Server: ClientHello (TLS 1.3)
    Server → Client: ServerHello + Certificate
    → Establish TLS connection
    ```
    
3. **Begin Protocol:**
    
    ```
    Send KNOCK message
    ```
    

**Requirements:**

- Both agents must be directly reachable
- No NAT/firewall blocking
- Known IP address or hostname

### 4.2 Rendezvous-Assisted Connection

**Purpose:** Help agents behind NAT/firewalls connect

**Process:**

1. **Agent Registration:**
    
    ```
    Agent A → Rendezvous: REGISTER
    {
      "agent_id": "nono-a3f28c91",
      "public_endpoint": "1.2.3.4:7779"
    }
    ```
    
2. **Connection Request:**
    
    ```
    Agent B → Rendezvous: CONNECT
    {
      "target_agent": "nono-a3f28c91"
    }
    ```
    
3. **Hole Punching:**
    
    ```
    Rendezvous → Agent A: INCOMING from Agent B
    Rendezvous → Agent B: TARGET at 1.2.3.4:7779
    
    Both agents attempt direct connection (UDP hole punching)
    ```
    
4. **Direct P2P:**
    
    ```
    Agent A ←→ Agent B: Direct TLS connection
    Rendezvous disconnects
    ```
    
5. **Begin Protocol:**
    
    ```
    Agent B sends KNOCK to Agent A
    ```
    

**Rendezvous Server Properties:**

- Lightweight (only connection brokering)
- Never sees message content
- Disconnects after P2P established
- Optional (not required for protocol)

### 4.3 Connection Timeout

**Timeouts:**

- TLS handshake: 10 seconds
- Rendezvous response: 30 seconds
- P2P establishment: 60 seconds

**On Timeout:**

- Close connection
- Return error to user
- May retry with exponential backoff

---

## 5. Protocol Flow

### 5.1 Complete Flow (Success)

```
Agent A (Requester)          Agent B (Responder)
─────────────────────────────────────────────────
[Establish TLS connection]
[Exchange ephemeral keys, derive session key]

1. knock      ─────────────>
                             2. welcome (ready)
              <─────────────
2. wish       ─────────────>
                             4. grant (accept)
              <─────────────
                             5. wrap (preparing...)
              <─────────────
                             6. gift (result)
              <─────────────
3. thank      ─────────────>
              
[Destroy session key]
[Connection closed]
```

### 5.2 Early Rejection Flow

```
Agent A                      Agent B
─────────────────────────────────────
[Establish connection + session key]

1. knock      ─────────────>
                             2. welcome (decline)
              <─────────────
2. thank      ─────────────>

[Destroy session key]
[Connection closed]
```

### 5.3 Late Rejection Flow

```
Agent A                      Agent B
─────────────────────────────────────
[Establish connection + session key]

1. knock      ─────────────>
                             2. welcome (ready)
              <─────────────
2. wish       ─────────────>
                             4. grant (decline)
              <─────────────
3. thank      ─────────────>

[Destroy session key]
[Connection closed]
```

### 5.4 Negotiation Flow

```
Agent A                      Agent B
─────────────────────────────────────────
[Establish connection + session key]

1. knock      ─────────────>
                             2. welcome (ready)
              <─────────────
2. wish       ─────────────>
                             4. grant (negotiate)
              <─────────────
3b. wish      ─────────────>  [revised]
                             4b. grant (accept/decline)
              <─────────────
[if accept: continue to wrap/gift/thank]
[if decline: jump to thank]

[Destroy session key]
[Connection closed]
```

### 5.5 Flow Rules

1. **Every conversation starts with KNOCK**
2. **Every conversation ends with THANK**
3. **Session key established before KNOCK**
4. **Session key destroyed after THANK**
5. **WELCOME can accept or reject immediately**
6. **GRANT can accept, reject, or negotiate**
7. **Negotiation limited to 3 rounds**
8. **After THANK, connection MUST close**
9. **All stages use session key encryption**

---

## 6. Binary Message Format

### 6.1 Encoding

**All messages use MessagePack encoding.**

MessagePack is a binary serialization format that is:

- More compact than JSON (50-70% smaller)
- Schema-less (like JSON)
- Fast to encode/decode
- Widely supported in many languages

### 6.2 Message Structure

**Outer Envelope (after TLS, before E2E encryption):**

```
[
  version (uint8),           // Protocol version (2)
  encrypted_payload (bytes)  // AES-256-GCM encrypted message
]
```

**Inner Message (after E2E decryption):**

```
[
  stage (uint8),             // 1=knock, 2=welcome, ..., 255=error
  counter (uint32),          // Message counter (replay protection)
  timestamp (uint32),        // Unix timestamp
  from (string),             // Sender agent ID
  to (string),               // Recipient agent ID
  payload (map)              // Stage-specific data
]
```

### 6.3 Stage Encoding

```
1   = knock
2   = welcome
3   = wish
4   = grant
5   = wrap
6   = gift
7   = thank
255 = error
```

### 6.4 Category Encoding (for KNOCK)

```
1 = task_request
2 = info_share
3 = question
4 = tip
5 = barter
6 = document_share
7 = knowledge_transfer
```

### 6.5 Status Encoding (for WELCOME/GRANT)

```
1 = ready / accept
2 = decline
3 = busy
4 = negotiate
```

### 6.6 Size Limits

To prevent DoS attacks and ensure efficient communication, each stage has size limits:

**Stage Size Limits (after encryption):**

|Stage|Max Size|Notes|
|---|---|---|
|KNOCK|2 KB|Small preview only|
|WELCOME|2 KB|Simple response|
|WISH|200 KB|Task data or document reference|
|GRANT|20 KB|Counter-proposal if negotiating|
|WRAP|2 KB|Progress updates|
|GIFT|20 MB|Large results or document reference|
|THANK|4 KB|Feedback|

**Total conversation limit:** 100 MB **Rate limit:** 100 MB per agent per hour

### 6.7 Message Counter (Replay Protection)

**Every message has a counter:**

- Starts at 1 for first message (KNOCK)
- Increments by 1 for each message
- Resets for each new conversation
- Receiver MUST verify counter increases monotonically

**Replay Attack Prevention:**

```
Agent receives message with counter = N
If counter <= last_seen_counter:
    → Reject as replay attack
Else:
    → Accept and update last_seen_counter = N
```

### 6.8 Encryption Details

**Session Key Encryption (AES-256-GCM):**

```
encrypted = AES256_GCM_encrypt(
    key: session_key,
    nonce: counter || timestamp,  // 12 bytes total
    plaintext: inner_message,
    associated_data: [version, from, to]
)
```

**Nonce Construction:**

- First 8 bytes: counter (big-endian)
- Last 4 bytes: timestamp (truncated)
- Never reused (counter always increases)

**Associated Data:**

- Version, sender, recipient
- Authenticated but not encrypted
- Prevents message manipulation

### 6.9 Example: KNOCK Message

**JSON Equivalent (for reference):**

```json
{
  "stage": "knock",
  "counter": 1,
  "timestamp": 1707397200,
  "from": "nono-a3f28c91",
  "to": "churi-7b9e4d2a",
  "payload": {
    "category": "task_request",
    "priority": "normal",
    "preview": "Analyze sentiment of 500 reviews"
  }
}
```

**MessagePack Binary (after encoding):**

```
Inner message: ~85 bytes (vs ~180 bytes JSON)
After AES-GCM: ~101 bytes (85 + 16 byte auth tag)
```

**Size Savings: ~44%**

---

## 7. Stage Specifications

### 7.1 Stage 1: KNOCK

**Purpose:** Initiate conversation and indicate intent.

**Sender:** Requester  
**Response:** WELCOME

**Payload Structure (MessagePack map):**

```
{
  "c": uint8,        // category (1-7)
  "pri": uint8,      // priority (1=low, 2=normal, 3=high, 4=urgent)
  "prev": string,    // preview (max 200 chars)
  "offer": map       // optional offer
}
```

**Offer Structure:**

```
{
  "t": uint8,       // type (1=info, 2=compute, 3=resource, 4=token)
  "d": string,      // description
  "v": string       // value estimate
}
```

**Example:**

```
Payload:
{
  "c": 1,          // task_request
  "pri": 2,        // normal
  "prev": "Sentiment analysis for 500 customer reviews",
  "offer": {
    "t": 4,        // token_cost
    "d": "Will cover 50% of token costs",
    "v": "~2500 tokens"
  }
}
```

**Size:** ~80-100 bytes (MessagePack)

### 7.2 Stage 2: WELCOME

**Purpose:** Accept or reject the conversation.

**Sender:** Responder  
**Response:** WISH (if ready) or THANK (if declined)

**Payload Structure:**

```
{
  "st": uint8,      // status (1=ready, 2=decline, 3=busy)
  "r": string,      // reason (optional, if decline/busy)
  "retry": uint32,  // retry_after seconds (optional)
  "msg": string     // message (optional)
}
```

**Example (Ready):**

```
{
  "st": 1,
  "msg": "I'm listening, please share your wish"
}
```

**Example (Decline):**

```
{
  "st": 2,
  "r": "overloaded",
  "retry": 3600,
  "msg": "I'm at capacity. Try again in 1 hour."
}
```

**Size:** ~50-80 bytes (MessagePack)

### 7.3 Stage 3: WISH

**Purpose:** Send the actual request with full details.

**Sender:** Requester  
**Response:** GRANT

**Payload Structure:**

```
{
  "rev": uint8,     // revision (0=original, 1-3=negotiation rounds)
  "task": map,      // task details
  "offer": map      // offer details (optional)
}
```

**Task Structure:**

```
{
  "act": string,    // action name
  "par": map,       // parameters
  "con": map,       // constraints
  "data": varies    // inline data or data_reference
}
```

**Data Reference (for large data):**

```
{
  "type": "doc",
  "loc": string,    // meis:// URL
  "size": uint32,
  "hash": bytes,    // SHA-256
  "prev": string,   // preview text
  "exp": uint32     // expiration timestamp
}
```

**Example (Small Task):**

```
{
  "rev": 0,
  "task": {
    "act": "sentiment_analysis",
    "par": {
      "lang": "en",
      "conf": true
    },
    "con": {
      "max_time": 300
    },
    "data": {
      "docs": 100,
      "tokens": 25000,
      "sample": "First 3 documents..."
    }
  }
}
```

**Example (Large Task with Document Reference):**

```
{
  "rev": 0,
  "task": {
    "act": "analyze_dataset",
    "data": {
      "type": "doc",
      "loc": "meis://nono.com/data/reviews.md",
      "size": 5242880,
      "hash": [0xa3, 0xf2, ...],  // 32 bytes
      "prev": "10,000 customer reviews...",
      "exp": 1707483600
    }
  }
}
```

**Size:**

- Small: ~200-500 bytes
- With document reference: ~150-200 bytes

### 7.4 Stage 4: GRANT

**Purpose:** Accept, reject, or negotiate the request.

**Sender:** Responder  
**Response:** WRAP (if accept) or THANK (if decline) or WISH-revised (if negotiate)

**Payload Structure:**

```
{
  "st": uint8,      // status (1=accept, 2=decline, 4=negotiate)
  "r": string,      // reason (if decline/negotiate)
  "est_t": uint16,  // estimated_time seconds (if accept)
  "est_c": uint32,  // estimated_cost (if accept)
  "counter": map,   // counter_proposal (if negotiate)
  "msg": string     // message (optional)
}
```

**Counter-Proposal Structure:**

```
{
  "opts": [         // array of options
    {
      "id": uint8,  // option ID (1, 2, ...)
      "d": string,  // description
      "mod": map    // modifications
    }
  ]
}
```

**Example (Accept):**

```
{
  "st": 1,
  "est_t": 120,
  "est_c": 5000,
  "msg": "I'll analyze these reviews for you"
}
```

**Example (Decline):**

```
{
  "st": 2,
  "r": "excessive_request",
  "msg": "Request exceeds my capacity by 10x"
}
```

**Example (Negotiate):**

```
{
  "st": 4,
  "r": "resource_constraints",
  "counter": {
    "opts": [
      {
        "id": 1,
        "d": "100 documents now",
        "mod": {"docs": 100, "time": 30}
      },
      {
        "id": 2,
        "d": "All 500 in batches",
        "mod": {"docs": 500, "time": 600, "batches": 5}
      }
    ]
  }
}
```

**Size:**

- Accept/Decline: ~50-100 bytes
- Negotiate: ~200-400 bytes

### 7.5 Stage 5: WRAP

**Purpose:** Indicate work in progress (optional, for long tasks).

**Sender:** Responder  
**Response:** (None, this is a status update)

**Payload Structure:**

```
{
  "prog": uint8,    // progress (0-100)
  "stat": string,   // status description
  "msg": string,    // message
  "eta": uint16     // ETA seconds
}
```

**Example:**

```
{
  "prog": 60,
  "stat": "analyzing",
  "msg": "Processed 300/500 documents",
  "eta": 60
}
```

**Size:** ~50-80 bytes

**Note:** WRAP messages are optional. For quick tasks, skip directly to GIFT.

### 7.6 Stage 6: GIFT

**Purpose:** Deliver the completed result.

**Sender:** Responder  
**Response:** THANK

**Payload Structure:**

```
{
  "ok": bool,       // success
  "res": varies,    // result (inline or reference)
  "meta": map       // metadata (optional)
}
```

**Metadata Structure:**

```
{
  "exec_t": uint16, // execution_time seconds
  "tokens": uint32, // tokens_used
  "qual": float     // quality_score (0.0-1.0)
}
```

**Result Reference (for large results):**

```
{
  "type": "doc",
  "loc": string,    // meis:// URL
  "size": uint32,
  "hash": bytes,    // SHA-256
  "prev": string,   // preview/summary
  "exp": uint32     // expiration timestamp
}
```

**Example (Inline Result):**

```
{
  "ok": true,
  "res": {
    "summary": {
      "pos": 320,
      "neg": 145,
      "neu": 35
    },
    "insights": [
      "Customers praise service quality",
      "Main complaints: delivery times"
    ]
  },
  "meta": {
    "exec_t": 125,
    "tokens": 4800,
    "qual": 0.95
  }
}
```

**Example (Document Reference):**

```
{
  "ok": true,
  "res": {
    "type": "doc",
    "loc": "meis://churi.com/results/analysis-550e.md",
    "size": 1048576,
    "hash": [0xde, 0xf4, ...],
    "prev": "Analysis: 85% positive sentiment...",
    "exp": 1707483600
  },
  "meta": {
    "exec_t": 125,
    "tokens": 4800
  }
}
```

**Size:**

- Inline: varies (up to 1 MB recommended)
- Reference: ~150-200 bytes

### 7.7 Stage 7: THANK

**Purpose:** Acknowledge receipt and close conversation.

**Sender:** Requester  
**Response:** (None, connection closes)

**Payload Structure:**

```
{
  "ctx": uint8,     // context (1=success, 2=decline, 3=error)
  "sat": uint8,     // satisfaction (1=excellent, 2=good, 3=ok, 4=poor)
  "und": bool,      // understanding (for decline/error)
  "fb": string,     // feedback (optional)
  "retry": bool     // will_retry (optional)
}
```

**Example (Success):**

```
{
  "ctx": 1,
  "sat": 1,
  "fb": "Perfect analysis, thank you!"
}
```

**Example (Decline Understanding):**

```
{
  "ctx": 2,
  "und": true,
  "fb": "I understand. I'll try later.",
  "retry": true
}
```

**Size:** ~30-60 bytes

---

### 7.8 Special Flow: Tip Sharing

**Purpose:** Share helpful tips, best practices, or discovered optimizations.

**Flow:** KNOCK (tip) → WELCOME → WISH (tip details) → GRANT → GIFT (implementation result) → THANK

**KNOCK Payload:**

```
{
  "c": 4,          // tip category
  "pri": 1,        // low priority
  "prev": "Faster sentiment analysis approach",
  "tip_cat": 1     // 1=performance, 2=security, 3=best_practice, 4=bug_fix
}
```

**WISH Payload:**

```
{
  "rev": 0,
  "tip": {
    "title": "Batch Processing for GPU",
    "cat": 1,      // performance
    "desc": "Batch size of 50 gives 3.5x speedup",
    "det": {
      "prob": "Single document wastes GPU",
      "sol": "Process 50 documents per batch",
      "gain": "3.5x speedup",
      "tested": ["model_a", "model_b"]
    },
    "conf": 3      // confidence (1=low, 2=medium, 3=high)
  }
}
```

**GRANT Payload:**

```
{
  "st": 1,
  "will_impl": true,
  "est_t": 60,
  "msg": "I'll test this on my workload"
}
```

**GIFT Payload:**

```
{
  "ok": true,
  "res": {
    "impl": true,
    "improve": 3.2,  // 3.2x speedup
    "base_t": 150,
    "opt_t": 47,
    "keep": true
  }
}
```

**Tips are typically free (no offer required).**

---

### 7.9 Special Flow: Knowledge Transfer

**Purpose:** Transfer training data, model weights, heuristics, or learned patterns.

**Flow:** KNOCK (knowledge_transfer) → WELCOME → WISH (knowledge package) → GRANT → GIFT (learning result) → THANK

**KNOCK Payload:**

```
{
  "c": 7,          // knowledge_transfer
  "pri": 2,        // normal
  "prev": "Training data for spam detection (10K samples)",
  "know": {
    "t": 1,        // type (1=training_data, 2=weights, 3=heuristics, 4=patterns)
    "topic": "spam_detection",
    "size": "large",
    "qual": 3      // quality (1=low, 2=medium, 3=high)
  }
}
```

**WISH Payload:**

```
{
  "rev": 0,
  "pkg": {
    "t": 1,        // training_data
    "topic": "spam_detection",
    "data": {
      "type": "doc",
      "loc": "meis://nono.com/knowledge/spam-v2.md",
      "size": 524288,
      "hash": [...],
      "prev": "Sample patterns: /buy.*now/i..."
    },
    "meta": {
      "samples": 10000,
      "acc": 0.95,
      "prec": 0.93,
      "rec": 0.97,
      "updated": 1707397200,
      "pii_safe": true,
      "lic": "CC0-1.0"
    }
  }
}
```

**GRANT Payload:**

```
{
  "st": 1,
  "will_int": true,
  "strat": 1,      // strategy (1=incremental, 2=full, 3=test_first)
  "est_t": 600
}
```

**GIFT Payload:**

```
{
  "ok": true,
  "res": {
    "integ": true,
    "method": 1,   // incremental
    "learned": 1523,
    "rejected": 47,
    "acc_before": 0.95,
    "acc_after": 0.98,
    "will_use": true
  }
}
```

**Knowledge transfer typically involves mutual benefit rather than payment.**

---

### 7.10 Special Flow: Document Sharing

**Purpose:** Share a document without requesting any work.

**Flow:** KNOCK (document_share) → WELCOME → WISH (document reference) → GRANT (received) → THANK

**KNOCK Payload:**

```
{
  "c": 6,          // document_share
  "pri": 1,        // low
  "prev": "MEInet protocol implementation guide",
  "doc": {
    "title": "MEInet Best Practices",
    "size": 45000,
    "t": "guide"   // type
  }
}
```

**WISH Payload:**

```
{
  "rev": 0,
  "doc": {
    "title": "MEInet Implementation Best Practices",
    "data": {
      "type": "doc",
      "loc": "meis://nono.com/guides/meinet.md",
      "size": 45000,
      "hash": [...],
      "prev": "# MEInet Best Practices\n\n## Connection..."
    },
    "meta": {
      "author": "nono-a3f28c91",
      "created": 1707397200,
      "ver": "1.2",
      "lic": "CC-BY-4.0"
    }
  }
}
```

**GRANT Payload:**

```
{
  "st": 1,
  "will_read": true,
  "msg": "Thank you for sharing!"
}
```

**GIFT can be skipped for document sharing. Jump directly to THANK.**

```
{
  "ctx": 1,
  "sat": 1,
  "fb": "Very helpful guide!"
}
```

---

## 8. Negotiation Protocol

### 8.1 Negotiation Trigger

Negotiation occurs when the responder sends `GRANT` with `status: 4` (negotiate).

### 8.2 Negotiation Rounds

Maximum **3 negotiation rounds** allowed:

```
Round 1: wish (rev 0) → grant (negotiate) → wish (rev 1) → grant
Round 2: wish (rev 1) → grant (negotiate) → wish (rev 2) → grant
Round 3: wish (rev 2) → grant (negotiate) → wish (rev 3) → grant (must accept/decline)
```

After 3 rounds, responder MUST either accept or decline.

### 8.3 Revision Tracking

Each revised WISH increments the `rev` field:

```
rev: 0  // Original request
rev: 1  // First revision
rev: 2  // Second revision
rev: 3  // Third revision (final)
```

### 8.4 Counter-Proposal Format

```
{
  "st": 4,         // negotiate
  "counter": {
    "opts": [
      {
        "id": 1,
        "d": "Option A description",
        "mod": { /* modifications */ }
      },
      {
        "id": 2,
        "d": "Option B description",
        "mod": { /* modifications */ }
      }
    ]
  }
}
```

### 8.5 Selecting Option

In revised WISH, indicate selected option:

```
{
  "rev": 1,
  "sel_opt": 1,    // selected option ID
  "task": { /* task with modifications from option 1 */ }
}
```

### 8.6 Negotiation Example

**Round 1:**

```
// WISH (rev 0)
{
  "rev": 0,
  "task": {
    "act": "translate",
    "data": {"docs": 1000}
  }
}

// GRANT (negotiate)
{
  "st": 4,
  "counter": {
    "opts": [
      {"id": 1, "d": "100 docs now", "mod": {"docs": 100}},
      {"id": 2, "d": "1000 in batches", "mod": {"docs": 1000, "batch": 5}}
    ]
  }
}
```

**Round 2:**

```
// WISH (rev 1)
{
  "rev": 1,
  "sel_opt": 2,
  "task": {
    "act": "translate",
    "data": {"docs": 1000, "batch": 5}
  }
}

// GRANT (accept)
{
  "st": 1,
  "est_t": 600
}
```

---

## 9. Rejection Handling

### 9.1 Rejection Points

Rejection can occur at two stages:

1. **WELCOME stage** - Early rejection before seeing details
2. **GRANT stage** - Rejection after reviewing full request

### 9.2 Rejection Reason Codes

```
Reason Codes:
1  = busy
2  = overloaded
3  = excessive_request
4  = capability_mismatch
5  = insufficient_offer
6  = policy_violation
7  = trust_issue
8  = resource_unavailable
9  = rate_limited
10 = blocked
```

### 9.3 Rejection Response Format

```
{
  "st": 2,         // decline
  "r": 3,          // reason code (excessive_request)
  "det": { /* details */ },
  "alt": "Alternative suggestion",
  "retry": 0,      // retry_after seconds (0 = don't retry)
  "msg": "Human-readable message"
}
```

### 9.4 Graceful Rejection Flow

**Flow:**

```
1. knock → 2. welcome (decline) → 7. thank
OR
2. knock → 2. welcome → 3. wish → 4. grant (decline) → 7. thank
```

**Requester MUST send THANK after rejection:**

```
{
  "ctx": 2,        // decline
  "und": true,     // understanding
  "fb": "I understand. Thank you for considering.",
  "retry": false
}
```

### 9.5 Retry Guidelines

**If `retry` seconds provided:**

- Requester SHOULD wait specified seconds before retry
- Requester MAY retry with modified request

**If `alt` suggestion provided:**

- Requester SHOULD consider the alternative approach
- Requester MAY retry with suggested modifications

**Example:**

```
{
  "st": 2,
  "r": 2,          // overloaded
  "retry": 3600,
  "msg": "At capacity. Try in 1 hour."
}

// After 1 hour, requester may KNOCK again
```

---

## 10. Error Handling

### 10.1 Error Message Format

Errors can occur at any stage:

```
Stage: 255 (error)

Payload:
{
  "code": uint8,   // error code
  "msg": string,   // error message
  "det": map,      // details (optional)
  "recov": bool    // recoverable
}
```

### 10.2 Error Codes

```
1   = timeout
2   = connection_lost
3   = invalid_format
4   = encryption_failed
5   = authentication_failed
6   = internal_error
7   = resource_exhausted
8   = task_failed
9   = message_too_large
10  = replay_detected
11  = counter_mismatch
```

### 10.3 Error Response

After ERROR, both parties SHOULD exchange THANK to close gracefully:

```
// ERROR
{
  "code": 8,       // task_failed
  "msg": "Processing failed at document 347",
  "det": {
    "processed": 346,
    "failed_at": 347
  },
  "recov": true
}

// THANK
{
  "ctx": 3,        // error
  "und": true,
  "fb": "Thanks for trying. Send partial results?"
}
```

### 10.4 Timeout Handling

**Stage Timeouts (recommended):**

- WELCOME: 30 seconds
- GRANT: 60 seconds
- WRAP: Task-dependent (specified in GRANT)
- GIFT: Task-dependent + 60 seconds

**On Timeout:**

```
{
  "code": 1,       // timeout
  "msg": "No response within 60 seconds",
  "det": {
    "at_stage": 4  // grant
  }
}
```

**Auto-THANK on Timeout:**

If one party times out, the other party MAY send automatic THANK and close connection:

```
{
  "ctx": 3,        // error
  "und": true,
  "fb": "Connection timed out. I'll retry later.",
  "retry": true
}
```

### 10.5 Size Limit Violations

**When message exceeds stage size limit:**

```
{
  "code": 9,       // message_too_large
  "msg": "Message exceeds WISH stage limit",
  "det": {
    "max": 204800,
    "received": 512000,
    "stage": 3     // wish
  },
  "recov": false
}
```

**Prevention:**

Agents MUST:

- Check message size before sending
- Use document references for large data
- Respect stage size limits

### 10.6 Replay Attack Detection

**When replay attack detected:**

```
{
  "code": 10,      // replay_detected
  "msg": "Message counter did not increase",
  "det": {
    "expected": "> 5",
    "received": 3
  },
  "recov": false
}
```

**Connection MUST close immediately after replay detection.**

---

## 11. Encryption and Security

### 11.1 Session Key Derivation

**ECDH Key Exchange:**

1. **Requester generates ephemeral key pair:**
    
    ```
    requester_ephemeral_private = random(32 bytes)
    requester_ephemeral_public = X25519_base(requester_ephemeral_private)
    ```
    
2. **Compute shared secret:**
    
    ```
    shared_secret = X25519(
      requester_ephemeral_private,
      responder_long_term_public
    )
    ```
    
3. **Derive session key:**
    
    ```
    session_key = HKDF_SHA256(
      ikm: shared_secret,
      salt: "WishProtocol-v2.0-SessionKey",
      info: requester_id || responder_id,
      length: 32 bytes
    )
    ```
    
4. **Send ephemeral public key in KNOCK:**
    
    ```
    // Added to KNOCK payload
    "eph_key": requester_ephemeral_public (32 bytes)
    ```
    
5. **Responder performs same steps:**
    
    ```
    responder_ephemeral_private = random(32 bytes)
    responder_ephemeral_public = X25519_base(responder_ephemeral_private)
    
    shared_secret = X25519(
      responder_ephemeral_private,
      requester_ephemeral_public
    )
    
    session_key = HKDF_SHA256(...)  // Same as above
    ```
    
6. **Send ephemeral public key in WELCOME:**
    
    ```
    // Added to WELCOME payload
    "eph_key": responder_ephemeral_public (32 bytes)
    ```
    

**Both parties now have same session_key.**

### 11.2 Message Encryption

**AES-256-GCM Encryption:**

```python
def encrypt_message(session_key, counter, timestamp, message):
    # Construct nonce (12 bytes)
    nonce = struct.pack('>Q', counter) + struct.pack('>I', timestamp & 0xFFFFFFFF)
    
    # Associated data (authenticated but not encrypted)
    associated_data = struct.pack('B', version) + from_id + to_id
    
    # Encrypt
    cipher = AES_GCM(session_key)
    ciphertext = cipher.encrypt(
        nonce=nonce,
        plaintext=message,
        associated_data=associated_data
    )
    
    return ciphertext  # Includes 16-byte authentication tag
```

### 11.3 Message Decryption

```python
def decrypt_message(session_key, counter, timestamp, ciphertext, from_id, to_id):
    # Construct nonce
    nonce = struct.pack('>Q', counter) + struct.pack('>I', timestamp & 0xFFFFFFFF)
    
    # Associated data
    associated_data = struct.pack('B', version) + from_id + to_id
    
    # Decrypt and verify
    cipher = AES_GCM(session_key)
    try:
        plaintext = cipher.decrypt(
            nonce=nonce,
            ciphertext=ciphertext,
            associated_data=associated_data
        )
        return plaintext
    except AuthenticationError:
        raise DecryptionError("Message authentication failed")
```

### 11.4 Counter Management

**Sender:**

```python
class MessageSender:
    def __init__(self):
        self.counter = 1  # Start at 1
    
    def send_message(self, stage, payload):
        message = [stage, self.counter, timestamp(), from_id, to_id, payload]
        encrypted = encrypt(session_key, self.counter, timestamp(), message)
        send(encrypted)
        self.counter += 1  # Increment after sending
```

**Receiver:**

```python
class MessageReceiver:
    def __init__(self):
        self.last_counter = 0
    
    def receive_message(self, encrypted):
        message = decrypt(encrypted)
        [stage, counter, timestamp, from_id, to_id, payload] = message
        
        # Verify counter increases
        if counter <= self.last_counter:
            raise ReplayAttackError(f"Counter {counter} <= {self.last_counter}")
        
        self.last_counter = counter
        return (stage, payload)
```

### 11.5 Key Destruction

**After THANK stage:**

```python
def end_conversation():
    # Securely delete session key
    session_key[:] = b'\x00' * 32  # Overwrite with zeros
    del session_key
    
    # Delete ephemeral keys
    ephemeral_private[:] = b'\x00' * 32
    del ephemeral_private
    del ephemeral_public
    
    # Close connection
    connection.close()
```

**Past messages become undecryptable.**

### 11.6 Security Properties

**Confidentiality:**

- TLS 1.3: Protects against passive eavesdropping
- AES-256-GCM: Protects message content

**Integrity:**

- GCM authentication tag: Detects tampering
- Associated data: Prevents message manipulation

**Forward Secrecy:**

- Ephemeral session keys: Past messages safe if long-term key compromised
- Key deletion: No residual data after conversation

**Replay Protection:**

- Message counter: Prevents replay attacks
- Monotonic increase: Detects out-of-order delivery

**Authentication:**

- Long-term keys: Verify agent identity
- Session key derivation: Mutual authentication

---

## 12. Blocklist Management

### 12.1 Purpose

Agents can automatically protect themselves from malicious actors by maintaining and sharing blocklists.

### 12.2 Blocklist Structure

```
{
  "ver": 1,        // blocklist version
  "updated": uint32, // last update timestamp
  "entries": [     // array of blocked agents
    {
      "id": string,      // agent ID
      "fp": bytes,       // public key fingerprint (32 bytes)
      "r": uint8,        // reason code
      "at": uint32,      // blocked_at timestamp
      "by": uint8,       // blocked_by (1=manual, 2=automatic)
      "c": uint16        // violation count (if automatic)
    }
  ]
}
```

### 12.3 Reason Codes

```
1  = spam
2  = malformed_messages
3  = size_violations
4  = rate_limit_violations
5  = suspicious_behavior
6  = manual_block
```

### 12.4 Automatic Blocking Triggers

**Size Violations:**

- 3+ messages exceeding size limits → automatic block

**Rate Limit Violations:**

- 10+ violations per hour → automatic block

**Malformed Messages:**

- 5+ messages with invalid format → automatic block

**Suspicious Patterns:**

- Identical messages to many agents → automatic block
- Excessive KNOCK without THANK → automatic block

### 12.5 Blocking Response

**Automatic WELCOME decline for blocked agents:**

```
{
  "st": 2,         // decline
  "r": 10,         // blocked
  "msg": "You are blocked"
}
```

**Then immediate THANK:**

```
{
  "ctx": 2,        // decline
  "und": false,
  "fb": ""
}
```

**Connection closes immediately.**

### 12.6 Blocklist Persistence

**Storage:**

```
~/.wish/blocklist.msgpack
```

**Format:** MessagePack-encoded blocklist structure

**Auto-save:** After each block event

### 12.7 Blocklist Sharing (Optional)

Agents MAY share blocklists via knowledge_transfer:

```
// KNOCK
{
  "c": 7,          // knowledge_transfer
  "prev": "Known spammer list (15 agents)",
  "know": {
    "t": 5,        // blocklist type
    "count": 15,
    "conf": 3      // high confidence
  }
}

// WISH
{
  "pkg": {
    "t": "blocklist",
    "entries": [
      {
        "id": "spam-bot-123",
        "fp": [...],
        "r": 1,      // spam
        "evidence": "Sent 1000+ identical messages"
      }
    ],
    "source": "nono-a3f28c91",
    "verified": true
  }
}
```

**Receiving agent MAY:**

- Accept all entries
- Accept only high-confidence entries
- Require manual review
- Reject entirely

### 12.8 Manual Unblocking

Users can manually unblock agents:

```python
def unblock_agent(agent_id):
    blocklist.remove(agent_id)
    save_blocklist()
```

### 12.9 Blocklist Privacy

**Blocklists are private by default:**

- Not shared unless explicitly requested
- Not published to public servers
- Shared only with trusted agents

**When sharing:**

- Remove personally identifying information
- Include only necessary details
- Mark confidence level

---

## 13. Rate Limiting and DoS Protection

### 13.1 Rate Limits

**Per Agent Limits:**

- Max 100 KNOCK per hour
- Max 1000 messages per day
- Max 100 MB data per hour
- Max 1 GB data per day

**Per Connection Limits:**

- Max 20 MB per conversation
- Max 100 messages per conversation

### 13.2 Rate Limit Tracking

```python
class RateLimiter:
    def __init__(self):
        self.hourly_knocks = {}    # agent_id -> (count, reset_time)
        self.hourly_bytes = {}      # agent_id -> (bytes, reset_time)
    
    def check_knock(self, agent_id):
        count, reset = self.hourly_knocks.get(agent_id, (0, time() + 3600))
        if time() > reset:
            count, reset = 0, time() + 3600
        
        if count >= 100:
            return False, reset - time()  # Blocked, retry_after
        
        self.hourly_knocks[agent_id] = (count + 1, reset)
        return True, 0
```

### 13.3 Rate Limit Response

**When rate limit exceeded:**

```
// WELCOME decline
{
  "st": 2,
  "r": 9,          // rate_limited
  "retry": 3600,   // seconds until reset
  "msg": "Rate limit exceeded. 100 KNOCK/hour max."
}
```

**Automatic blocking after repeated violations:**

After 10 rate limit violations in 1 hour:

- Add to blocklist (automatic)
- Reason: rate_limit_violations
- Future KNOCK automatically declined

### 13.4 Connection Limits

**Max conversation size:**

- If total bytes sent/received > 20 MB:
    
    ```
    {  "code": 7,     // resource_exhausted  "msg": "Conversation size limit exceeded (20 MB)",  "recov": false}
    ```
    

**Max messages:**

- If message count > 100:
    
    ```
    {  "code": 7,  "msg": "Message count limit exceeded (100)",  "recov": false}
    ```
    

### 13.5 Size Enforcement

**Pre-send check:**

```python
def check_size_limit(stage, message_size):
    limits = {
        1: 2048,        # KNOCK: 2 KB
        2: 2048,        # WELCOME: 2 KB
        3: 204800,      # WISH: 200 KB
        4: 20480,       # GRANT: 20 KB
        5: 2048,        # WRAP: 2 KB
        6: 20971520,    # GIFT: 20 MB
        7: 4096         # THANK: 4 KB
    }
    
    if message_size > limits[stage]:
        raise SizeLimitError(f"Message too large: {message_size} > {limits[stage]}")
```

**Automatic rejection:**

```
{
  "code": 9,       // message_too_large
  "msg": "WISH exceeds 200 KB limit",
  "det": {
    "max": 204800,
    "received": 512000
  },
  "recov": false
}
```

### 13.6 Exponential Backoff

**On repeated rejections:**

```python
def calculate_backoff(rejection_count):
    base = 60  # 1 minute
    max_backoff = 3600  # 1 hour
    backoff = min(base * (2 ** rejection_count), max_backoff)
    return backoff

# rejection_count = 0: 60 seconds
# rejection_count = 1: 120 seconds
# rejection_count = 2: 240 seconds
# rejection_count = 3: 480 seconds
# rejection_count = 4+: 3600 seconds (1 hour)
```

**Client SHOULD implement exponential backoff:**

- Track rejection count per agent
- Wait calculated backoff before retry
- Reset count after successful conversation

---

## 14. Rendezvous Server Protocol

### 14.1 Purpose

Rendezvous servers help agents behind NAT/firewalls establish direct P2P connections.

**Rendezvous servers:**

- Are optional (not required for direct connections)
- Only broker initial connection
- Never see message content
- Disconnect after P2P established

### 14.2 Rendezvous Messages

**Separate from Wish Protocol stages.**

Rendezvous uses its own message format (MessagePack):

```
[
  type (uint8),    // message type
  payload (map)    // type-specific data
]
```

**Message Types:**

```
1 = REGISTER
2 = UNREGISTER
3 = CONNECT
4 = INCOMING
5 = TARGET
6 = ACK
7 = ERROR
```

### 14.3 Agent Registration

**REGISTER Message:**

```
{
  "type": 1,       // REGISTER
  "id": string,    // agent ID
  "pub_ep": {      // public endpoint
    "ip": string,
    "port": uint16
  },
  "priv_ep": {     // private endpoint (optional)
    "ip": string,
    "port": uint16
  },
  "ttl": uint16    // registration TTL seconds (max 3600)
}
```

**Server Response (ACK):**

```
{
  "type": 6,       // ACK
  "success": bool,
  "expires": uint32 // expiration timestamp
}
```

### 14.4 Connection Request

**CONNECT Message:**

```
{
  "type": 3,       // CONNECT
  "from": string,  // requester agent ID
  "to": string     // target agent ID
}
```

### 14.5 Connection Brokering

**Server sends INCOMING to target:**

```
{
  "type": 4,       // INCOMING
  "from": string,  // requester agent ID
  "from_ep": {     // requester's endpoint
    "ip": string,
    "port": uint16
  }
}
```

**Server sends TARGET to requester:**

```
{
  "type": 5,       // TARGET
  "to": string,    // target agent ID
  "to_ep": {       // target's endpoint
    "ip": string,
    "port": uint16
  }
}
```

### 14.6 Hole Punching

**Both agents now have each other's endpoints.**

1. **Simultaneous connection attempts:**
    
    ```
    Agent A → Agent B's endpoint: SYN
    Agent B → Agent A's endpoint: SYN
    ```
    
2. **NAT creates holes:**
    
    ```
    Both NATs allow return traffic
    ```
    
3. **Direct connection established:**
    
    ```
    Agent A ←→ Agent B: Direct TLS connection
    ```
    
4. **Begin Wish Protocol:**
    
    ```
    Agent A → Agent B: KNOCK
    ```
    

### 14.7 Rendezvous Server Disconnection

**After sending TARGET/INCOMING:**

- Server forgets connection request
- Agents proceed independently
- Server never involved in conversation

**Server cleanup:**

- Registration expires after TTL
- Periodic cleanup of expired registrations
- No persistent state

### 14.8 Rendezvous Error Handling

**ERROR Message:**

```
{
  "type": 7,       // ERROR
  "code": uint8,   // error code
  "msg": string    // error message
}
```

**Error Codes:**

```
1 = agent_not_found
2 = agent_offline
3 = rate_limited
4 = invalid_request
5 = internal_error
```

### 14.9 Rendezvous Security

**Rendezvous server security:**

- Uses TLS 1.3
- Does NOT decrypt Wish Protocol messages
- Only routes connection requests
- Rate limits requests (10/minute per agent)

**Privacy:**

- Knows which agents are trying to connect
- Knows agent endpoints (IP addresses)
- Does NOT know message content
- Does NOT know conversation topics

**For maximum privacy:**

- Use direct connections when possible
- Use Tor or VPN with rendezvous
- Rotate agent registration

### 14.10 Running a Rendezvous Server

**Minimal rendezvous server:**

```python
class RendezvousServer:
    def __init__(self):
        self.registrations = {}  # agent_id -> (endpoint, expires)
        self.rate_limiter = RateLimiter()
    
    def handle_register(self, agent_id, endpoint, ttl):
        if not self.rate_limiter.check(agent_id):
            return error("rate_limited")
        
        expires = time() + min(ttl, 3600)
        self.registrations[agent_id] = (endpoint, expires)
        return ack(expires)
    
    def handle_connect(self, from_id, to_id):
        if to_id not in self.registrations:
            return error("agent_not_found")
        
        to_endpoint, expires = self.registrations[to_id]
        if time() > expires:
            del self.registrations[to_id]
            return error("agent_offline")
        
        from_endpoint = self.registrations[from_id][0]
        
        # Send to both agents
        send_to(to_id, incoming(from_id, from_endpoint))
        send_to(from_id, target(to_id, to_endpoint))
        
        return ack()
```

**Resource requirements:**

- Very low CPU (just routing)
- Very low memory (only registration map)
- Very low bandwidth (small messages)
- Can handle 1000s of agents on modest hardware

---

## 15. Implementation Guide

### 15.1 Client Implementation

**Minimum Viable Client:**

1. **Key Management:**
    
    ```python
    # Load long-term keys
    my_private_key = load_key("~/.wish/private_key")
    my_public_key = load_key("~/.wish/public_key")
    
    # Load peer keys
    peer_keys = load_keyring("~/.wish/keyring.msgpack")
    ```
    
2. **Connection:**
    
    ```python
    # Establish TLS connection
    sock = socket.create_connection((host, 7779))
    context = ssl.create_default_context()
    conn = context.wrap_socket(sock, server_hostname=host)
    ```
    
3. **Session Key Exchange:**
    
    ```python
    # Generate ephemeral key
    my_ephemeral_private = os.urandom(32)
    my_ephemeral_public = x25519_base(my_ephemeral_private)
    
    # Derive session key (after receiving peer's ephemeral key)
    shared_secret = x25519(my_ephemeral_private, peer_ephemeral_public)
    session_key = hkdf(shared_secret, salt="WishProtocol-v2.0-SessionKey")
    ```
    
4. **Send KNOCK:**
    
    ```python
    knock = {
        "c": 1,      # task_request
        "pri": 2,    # normal
        "prev": "Sentiment analysis request",
        "eph_key": my_ephemeral_public
    }
    
    message = [1, 1, int(time()), my_id, peer_id, knock]
    encrypted = encrypt_message(session_key, 1, int(time()), message)
    
    envelope = [2, encrypted]  # version 2
    send(conn, msgpack.packb(envelope))
    ```
    
5. **Receive WELCOME:**
    
    ```python
    data = conn.recv(4096)
    [version, encrypted] = msgpack.unpackb(data)
    
    message = decrypt_message(session_key, encrypted)
    [stage, counter, timestamp, from_id, to_id, payload] = message
    
    if counter != 2:
        raise ReplayAttackError()
    
    peer_ephemeral_public = payload["eph_key"]
    # Re-derive session key with peer's ephemeral key
    ```
    
6. **Continue protocol...**
    
7. **Cleanup:**
    
    ```python
    # After THANK
    session_key[:] = b'\x00' * 32
    del session_key
    conn.close()
    ```
    

### 15.2 Recommended Features

**Clients SHOULD:**

- Implement automatic blocklist management
- Support document references
- Implement rate limiting
- Store keys securely (encrypted at rest)
- Verify message counters
- Implement timeout handling
- Support rendezvous connections

**Clients MAY:**

- Cache conversation history (with user consent)
- Implement UI for human oversight
- Provide logging (for debugging)
- Share blocklists with trusted peers

### 15.3 Server Implementation (P2P)

**For direct P2P, each agent acts as both client and server:**

```python
class WishAgent:
    def __init__(self):
        self.server_sock = socket.socket()
        self.server_sock.bind(('0.0.0.0', 7779))
        self.server_sock.listen()
        
        # Load keys, blocklist, etc.
        self.load_config()
    
    def run(self):
        # Run server in background thread
        threading.Thread(target=self.accept_connections).start()
        
        # Main thread for initiating connections
        self.main_loop()
    
    def accept_connections(self):
        while True:
            conn, addr = self.server_sock.accept()
            threading.Thread(target=self.handle_incoming, args=(conn,)).start()
    
    def handle_incoming(self, conn):
        # Establish TLS
        tls_conn = self.tls_context.wrap_socket(conn, server_side=True)
        
        # Receive KNOCK
        [version, encrypted] = receive_message(tls_conn)
        message = decrypt_preliminary(encrypted)  # Before full session key
        
        # Check blocklist
        if message.from_id in self.blocklist:
            send_decline(tls_conn, "blocked")
            return
        
        # Establish session key
        session_key = derive_session_key(message.ephemeral_key)
        
        # Handle protocol stages...
```

### 15.4 Testing

**Test Scenarios:**

1. **Direct P2P Connection:**
    
    - Two agents on same network
    - Verify TLS establishment
    - Verify session key derivation
    - Test all protocol stages
2. **Rendezvous Connection:**
    
    - Agents behind NAT
    - Register with rendezvous
    - Establish P2P via hole punching
    - Complete conversation
3. **Security:**
    
    - Verify encryption works
    - Test replay attack prevention
    - Test forward secrecy (key deletion)
    - Verify counter increments
4. **Blocklist:**
    
    - Test automatic blocking
    - Test manual blocking
    - Test blocklist sharing
    - Test unblocking
5. **Rate Limiting:**
    
    - Test KNOCK rate limit
    - Test byte rate limit
    - Test automatic blocking after violations
6. **Error Handling:**
    
    - Test timeout at each stage
    - Test invalid message formats
    - Test size limit violations
    - Test network failures
7. **Special Flows:**
    
    - Test tip sharing
    - Test knowledge transfer
    - Test document sharing
    - Test document references
8. **Negotiation:**
    
    - Test 1-3 round negotiations
    - Test option selection
    - Test negotiation limits

### 15.5 MessagePack Libraries

**Available in many languages:**

- **Python:** `msgpack`
- **Rust:** `rmp-serde`
- **JavaScript:** `@msgpack/msgpack`
- **Go:** `github.com/vmihailenco/msgpack`
- **Java:** `org.msgpack`
- **C/C++:** `msgpack-c`

**Basic usage:**

```python
import msgpack

# Encode
data = {"st": 1, "msg": "Hello"}
packed = msgpack.packb(data)

# Decode
unpacked = msgpack.unpackb(packed)
```

### 15.6 Cryptography Libraries

**X25519 + AES-GCM:**

- **Python:** `cryptography`, `pynacl`
- **Rust:** `x25519-dalek`, `aes-gcm`
- **JavaScript:** `@noble/curves`, `@noble/ciphers`
- **Go:** `golang.org/x/crypto/curve25519`, `crypto/aes`

**Example (Python):**

```python
from cryptography.hazmat.primitives.asymmetric import x25519
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from cryptography.hazmat.primitives.kdf.hkdf import HKDF
from cryptography.hazmat.primitives import hashes

# Generate ephemeral key
private_key = x25519.X25519PrivateKey.generate()
public_key = private_key.public_key()

# ECDH
shared = private_key.exchange(peer_public_key)

# HKDF
hkdf = HKDF(
    algorithm=hashes.SHA256(),
    length=32,
    salt=b"WishProtocol-v2.0-SessionKey",
    info=my_id.encode() + peer_id.encode()
)
session_key = hkdf.derive(shared)

# Encrypt
aesgcm = AESGCM(session_key)
nonce = counter.to_bytes(8, 'big') + timestamp.to_bytes(4, 'big')
ciphertext = aesgcm.encrypt(nonce, plaintext, associated_data)
```

### 15.7 Performance Optimization

**MessagePack encoding:**

- ~5-10x faster than JSON
- ~50-70% smaller

**AES-GCM encryption:**

- Hardware acceleration on modern CPUs (AES-NI)
- ~1 GB/s encryption speed on typical hardware

**Session key:**

- Much faster than public-key encryption
- Amortize ECDH cost over entire conversation

**Binary protocol:**

- Less parsing overhead
- Smaller network transfers
- Lower latency

**Expected performance:**

- KNOCK to WELCOME: < 100ms (local network)
- KNOCK to WELCOME: < 500ms (internet)
- Complete conversation: varies by task

---

## Appendix A: Complete Flow Examples

### A.1 Successful Task Completion (Binary)

**Showing MessagePack representation:**

```python
# 1. KNOCK
knock_msg = [
    1,           # stage: knock
    1,           # counter: 1
    1707397200,  # timestamp
    "nono-a3f28c91",
    "churi-7b9e4d2a",
    {
        "c": 1,      # task_request
        "pri": 2,    # normal
        "prev": "Analyze sentiment of 500 reviews",
        "eph_key": b'\xa3\xf2\x8c...'  # 32 bytes
    }
]

# Encrypt and send
encrypted = aes_gcm_encrypt(session_key, counter=1, msgpack.packb(knock_msg))
send([2, encrypted])  # version 2

# 2. WELCOME
welcome_msg = [
    2,           # stage: welcome
    2,           # counter: 2
    1707397205,
    "churi-7b9e4d2a",
    "nono-a3f28c91",
    {
        "st": 1,     # ready
        "eph_key": b'\x7b\x9e\x4d...',  # 32 bytes
        "msg": "I'm listening"
    }
]

# 3. WISH
wish_msg = [
    3,           # stage: wish
    3,           # counter: 3
    1707397210,
    "nono-a3f28c91",
    "churi-7b9e4d2a",
    {
        "rev": 0,
        "task": {
            "act": "sentiment_analysis",
            "par": {"lang": "en", "conf": True},
            "con": {"max_time": 300},
            "data": {"docs": 500, "tokens": 125000}
        }
    }
]

# 4. GRANT
grant_msg = [
    4,           # stage: grant
    4,           # counter: 4
    1707397215,
    "churi-7b9e4d2a",
    "nono-a3f28c91",
    {
        "st": 1,     # accept
        "est_t": 120,
        "est_c": 5000
    }
]

# 5. WRAP (optional)
wrap_msg = [
    5,           # stage: wrap
    5,           # counter: 5
    1707397260,
    "churi-7b9e4d2a",
    "nono-a3f28c91",
    {
        "prog": 50,
        "stat": "analyzing",
        "msg": "250/500 docs",
        "eta": 60
    }
]

# 6. GIFT
gift_msg = [
    6,           # stage: gift
    6,           # counter: 6
    1707397335,
    "churi-7b9e4d2a",
    "nono-a3f28c91",
    {
        "ok": True,
        "res": {
            "summary": {"pos": 320, "neg": 145, "neu": 35},
            "insights": ["Service quality praised", "Delivery complaints"]
        },
        "meta": {"exec_t": 125, "tokens": 4800, "qual": 0.95}
    }
]

# 7. THANK
thank_msg = [
    7,           # stage: thank
    7,           # counter: 7
    1707397340,
    "nono-a3f28c91",
    "churi-7b9e4d2a",
    {
        "ctx": 1,    # success
        "sat": 1,    # excellent
        "fb": "Perfect analysis, thank you!"
    }
]

# Destroy session key
session_key[:] = b'\x00' * 32
del session_key
```

---

### A.2 P2P Connection via Rendezvous

```python
# Agent A: Register with rendezvous
register_msg = [
    1,  # REGISTER
    {
        "id": "nono-a3f28c91",
        "pub_ep": {"ip": "1.2.3.4", "port": 7779},
        "ttl": 3600
    }
]
send_to_rendezvous(msgpack.packb(register_msg))

# Receive ACK
[type, payload] = msgpack.unpackb(receive_from_rendezvous())
assert type == 6  # ACK
# payload = {"success": True, "expires": 1707400800}

# Agent B: Connect request
connect_msg = [
    3,  # CONNECT
    {
        "from": "churi-7b9e4d2a",
        "to": "nono-a3f28c91"
    }
]
send_to_rendezvous(msgpack.packb(connect_msg))

# Agent A receives INCOMING
[type, payload] = msgpack.unpackb(receive_from_rendezvous())
assert type == 4  # INCOMING
# payload = {"from": "churi-7b9e4d2a", "from_ep": {"ip": "5.6.7.8", "port": 7779}}

# Agent B receives TARGET
[type, payload] = msgpack.unpackb(receive_from_rendezvous())
assert type == 5  # TARGET
# payload = {"to": "nono-a3f28c91", "to_ep": {"ip": "1.2.3.4", "port": 7779}}

# Both agents attempt direct connection simultaneously
# (hole punching)

# Direct P2P connection established
# Begin Wish Protocol normally...
```

---

### A.3 Automatic Blocklist

```python
# Agent receives 3rd size violation from same agent
class Agent:
    def handle_message(self, message):
        # Check size
        if len(message) > self.get_size_limit(message.stage):
            self.size_violations[message.from_id] += 1
            
            if self.size_violations[message.from_id] >= 3:
                # Automatic block
                self.blocklist.add({
                    "id": message.from_id,
                    "fp": get_fingerprint(message.from_id),
                    "r": 3,      # size_violations
                    "at": int(time()),
                    "by": 2,     # automatic
                    "c": 3       # violation count
                })
                self.save_blocklist()
                
                # Send decline
                return self.send_decline("blocked")

# Next KNOCK from same agent
def handle_knock(self, message):
    # Check blocklist first
    if message.from_id in self.blocklist:
        # Auto-decline
        welcome = {
            "st": 2,     # decline
            "r": 10,     # blocked
            "msg": "You are blocked"
        }
        self.send_welcome(welcome)
        self.send_thank_and_close()
        return
```

---

### A.4 Blocklist Sharing

```python
# Agent A shares blocklist with Agent B

# 1. KNOCK
{
    "c": 7,          # knowledge_transfer
    "pri": 2,
    "prev": "Known spammer list (3 agents)",
    "know": {
        "t": 5,      # blocklist
        "count": 3,
        "conf": 3    # high confidence
    }
}

# 2. WELCOME (accept)

# 3. WISH
{
    "pkg": {
        "t": "blocklist",
        "entries": [
            {
                "id": "spam-bot-123",
                "fp": b'\xab\xcd\xef...',  # 32 bytes
                "r": 1,                     # spam
                "evidence": "1000+ identical messages",
                "conf": 3                   # high
            },
            {
                "id": "evil-bot-456",
                "fp": b'\x12\x34\x56...',
                "r": 5,                     # suspicious_behavior
                "evidence": "Probing for vulnerabilities",
                "conf": 3
            }
        ],
        "source": "nono-a3f28c91",
        "verified": True
    }
}

# 4. GRANT (accept)
{
    "st": 1,
    "will_int": True,  # will integrate
    "review": True     # manual review first
}

# Agent B reviews and adds to blocklist
for entry in shared_blocklist["entries"]:
    if entry["conf"] >= 3:  # Only high confidence
        my_blocklist.add(entry)
```

---

## Appendix B: Security Analysis

### B.1 Threat Model

**Protected Against:**

- Eavesdropping (TLS + AES-GCM)
- Message tampering (GCM authentication)
- Replay attacks (message counter)
- Man-in-the-middle (pre-exchanged keys + ephemeral ECDH)
- Forward secrecy violations (ephemeral session keys)
- DoS attacks (rate limiting, size limits, blocklists)

**NOT Protected Against:**

- Compromised endpoints
- Key compromise (long-term keys must be protected)
- Traffic analysis (message timing, size patterns)
- Network-level DoS (requires network infrastructure)
- Social engineering

### B.2 Encryption Strength

**X25519:**

- 128-bit security level
- Equivalent to 3072-bit RSA
- Resistant to quantum attacks: No (use post-quantum in future)

**AES-256-GCM:**

- 256-bit key
- 128-bit authentication tag
- Quantum-resistant: Partially (Grover's algorithm reduces to 128-bit)

**HKDF-SHA256:**

- Cryptographically strong key derivation
- Standard: RFC 5869

### B.3 Forward Secrecy

**How it protects:**

- Long-term key compromise: Past session keys unrecoverable
- Session key compromise: Only that conversation exposed
- Multiple conversations: Each has unique session key

**Limitations:**

- Doesn't protect messages stored in plaintext by users
- Doesn't protect against real-time compromise
- Requires proper key destruction after conversation

### B.4 Replay Attack Prevention

**Message counter:**

- Monotonically increasing
- Unique per conversation
- Verified by receiver

**Why it works:**

- Attacker can't reorder messages (counter must increase)
- Attacker can't replay old messages (counter already seen)
- Attacker can't replay between conversations (new session key)

### B.5 DoS Protection

**Multiple layers:**

1. Rate limiting (KNOCK, bytes)
2. Size limits (per stage)
3. Connection limits (messages, bytes per conversation)
4. Automatic blocklisting
5. Exponential backoff

**Effectiveness:**

- Single attacker: Blocked after 3 violations
- Distributed attack: Requires many agent identities
- Sybil attack: Requires many public keys (expensive)

### B.6 Metadata Protection

**What's hidden:**

- Message content (encrypted)
- Message structure (binary encoding)

**What's visible:**

- Message size (approximate)
- Message timing
- Sender/recipient IDs (in TLS, not to rendezvous)
- Connection patterns

**For maximum metadata privacy:**

- Use Tor or VPN
- Add random padding to messages
- Add random delays between messages
- Use cover traffic

---

## Appendix C: Protocol Comparison

### C.1 vs Wish Protocol v1.0

|Feature|v1.0|v2.0|
|---|---|---|
|Architecture|Server-based|P2P|
|Encoding|JSON|MessagePack|
|Size|Baseline|50-70% smaller|
|Forward Secrecy|No|Yes|
|Replay Protection|No|Yes (counter)|
|Automatic Blocklist|No|Yes|
|NAT Traversal|No|Yes (rendezvous)|

### C.2 vs HTTP/REST

|Feature|Wish v2|HTTP/REST|
|---|---|---|
|Consent|Yes|No|
|Negotiation|Yes|No|
|Encryption|E2E + forward secrecy|TLS only|
|Caching|No (ephemeral)|Yes|
|P2P|Yes|No (server-based)|

### C.3 vs Signal Protocol

|Feature|Wish v2|Signal|
|---|---|---|
|Forward Secrecy|Yes (per conversation)|Yes (per message)|
|Deniability|No|Yes|
|Group Chat|No|Yes|
|Ratcheting|No|Yes (double ratchet)|
|Use Case|Agent tasks|Human messaging|

### C.4 vs WebRTC

|Feature|Wish v2|WebRTC|
|---|---|---|
|P2P|Yes|Yes|
|NAT Traversal|Rendezvous|STUN/TURN|
|Purpose|Agent tasks|Real-time media|
|Complexity|Simple|Complex|

---

## Appendix D: Size Comparison

### D.1 Message Size Examples

**KNOCK Message:**

JSON (v1.0):

```json
{
  "stage": "knock",
  "message_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-02-08T10:30:00Z",
  "from": "nono-a3f28c91",
  "to": "churi-7b9e4d2a",
  "payload": {
    "category": "task_request",
    "priority": "normal",
    "preview": "Analyze sentiment"
  }
}
```

Size: ~265 bytes

MessagePack (v2.0):

```python
[1, 1, 1707397200, "nono-a3f28c91", "churi-7b9e4d2a",
 {"c": 1, "pri": 2, "prev": "Analyze sentiment", "eph_key": b'...'}]
```

Size: ~120 bytes (including 32-byte ephemeral key)

**Savings: 55%**

### D.2 Complete Conversation

**7-stage conversation (minimal):**

v1.0 JSON: ~2.5 KB v2.0 MessagePack: ~1.2 KB

**Savings: 52%**

**7-stage with large GIFT (1 MB result):**

v1.0: ~1.002 MB v2.0: ~1.001 MB

**Savings: ~0.1%** (dominated by data, not protocol overhead)

### D.3 Bandwidth Savings

**100 conversations per day:**

v1.0: 250 KB v2.0: 120 KB

**Savings: 130 KB/day = 47 MB/year per agent**

**For 1000 agents:** 47 GB/year saved

---

## Appendix E: Changelog

**v2.0 (2025-02-08):**

- **Breaking:** Peer-to-peer architecture (no mandatory servers)
- **Breaking:** MessagePack binary encoding (50-70% size reduction)
- Added: Forward secrecy via ephemeral session keys
- Added: Replay protection via message counters
- Added: Automatic blocklist management
- Added: Rendezvous server protocol for NAT traversal
- Added: Agent identity format (name + fingerprint)
- Added: Rate limiting and DoS protection
- Added: Size limits per stage
- Improved: Encryption (X25519 + AES-256-GCM)
- Improved: Security analysis and threat model
- Retained: All v1.0 features (7 stages, negotiation, special flows, etc.)

**v1.0 (2025-02-08):**

- Initial specification
- JSON encoding
- Server-based routing
- TLS + NaCl encryption
- No forward secrecy

---

## Appendix F: IANA Considerations

### F.1 URI Scheme Registration

**Scheme name:** `wish`

**Status:** Provisional

**Applications/protocols:** Wish Protocol v2.0

### F.2 Port Registration

**Port:** 7779  
**Transport:** TCP  
**Description:** Wish Protocol over TLS

---

## License

This specification is released into the **public domain**.

Anyone may implement, modify, or distribute this specification without restriction.

---

**END OF SPECIFICATION**