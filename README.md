# Aeitron Official Rust SDK (`aeitron`)

[![crates.io](https://img.shields.io/crates/v/aeitron.svg)](https://crates.io/crates/aeitron)
[![docs.rs](https://docs.rs/aeitron/badge.svg)](https://docs.rs/aeitron)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Official, asynchronous, memory-safe client library for the **Aeitron Intelligent Global Edge & Sovereign AI Platform**.

---

## Features

- **Asynchronous & Fast:** Built natively with Tokio, Hyper, and Reqwest for ultra-low latency line-rate invocation.
- **Multi-Objective AI Orchestration:** Query completions with explicit multi-variable routing (`LATENCY_OPTIMAL`, `COST_MINIMAL`, `CARBON_NEUTRAL`, `SOVEREIGN_PINNED`).
- **Domain & Edge Lifecycle:** Programmatic onboarding, DNS verification, and origin pool management.
- **Enterprise Webhook Verification:** Constant-time HMAC-SHA256 signature verification powered by `ring` with anti-replay timestamp tolerance.
- **Resilient Reliability:** Automatic exponential backoff with jitter on HTTP 429 rate limits and 5xx transient gateway errors.
- **Strictly Typed:** Zero guesswork with compile-time checked domain models.

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
aeitron = "0.1"
tokio = { version = "1", features = ["full"] }
```

---

## Quickstart

### 1. Multi-Objective AI Chat Completion

```rust
use aeitron::{AeitronClient, ChatCompletionRequest, ChatMessage, RoutingStrategy, Jurisdiction};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AeitronClient::new("atk_live_your_api_key_here")?;

    let request = ChatCompletionRequest::new(
        "aeitron-fast-l4",
        vec![ChatMessage::user("Analyze BDIX route latency in Dhaka.")],
    )
    .with_strategy(RoutingStrategy::LatencyOptimal)
    .with_jurisdiction(Jurisdiction::Bangladesh);

    let response = client.inference().create_chat_completion(&request).await?;

    println!("Model: {}", response.choices[0].message.content);
    println!("Telemetry: Region={:?}, TTFT={:?}ms, Cost=${:?}",
        response.telemetry.region,
        response.telemetry.ttft_ms,
        response.telemetry.cost
    );

    Ok(())
}
```

---

### 2. Verify Enterprise Webhook Signatures

```rust
use aeitron::WebhookVerifier;
use std::time::Duration;

fn handle_webhook(payload: &[u8], signature_header: &str) -> Result<(), Box<dyn std::error::Error>> {
    let secret = "whsec_your_signing_secret";

    WebhookVerifier::verify(
        payload,
        signature_header,
        secret,
        Duration::from_secs(300), // 5-minute replay tolerance
    )?;

    println!("Webhook cryptographically verified.");
    Ok(())
}
```

---

## Testing

```bash
cargo test --package aeitron
```
