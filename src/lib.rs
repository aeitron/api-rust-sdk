//! Official Rust SDK for the **Aeitron Intelligent Global Edge & Sovereign AI Platform**.
//!
//! Provides strictly-typed, asynchronous, ergonomic bindings for multi-objective AI routing,
//! custom edge domains, Anycast DNS, microVM compute, and enterprise webhook verification.
//!
//! # Quickstart
//! ```no_run
//! use aeitron::{AeitronClient, ChatCompletionRequest, ChatMessage, RoutingStrategy};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = AeitronClient::new("atk_live_your_api_key")?;
//!
//!     let req = ChatCompletionRequest::new(
//!         "aeitron-fast-l4",
//!         vec![ChatMessage::user("Ping edge in Dhaka")]
//!     ).with_strategy(RoutingStrategy::LatencyOptimal);
//!
//!     let resp = client.inference().create_chat_completion(&req).await?;
//!     println!("Response: {}", resp.choices[0].message.content);
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod errors;
pub mod types;
pub mod webhooks;

pub use client::{
    AeitronClient, ClientBuilder, DEFAULT_API_VERSION, DEFAULT_BASE_URL, SDK_VERSION,
};
pub use errors::{AeitronError, WebhookError};
pub use types::*;
pub use webhooks::WebhookVerifier;
