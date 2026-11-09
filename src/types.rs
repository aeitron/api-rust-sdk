use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Multi-objective intelligent routing strategies for AI model execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RoutingStrategy {
    #[default]
    LatencyOptimal,
    CostMinimal,
    CarbonNeutral,
    SovereignPinned,
}

/// Geopolitical sovereign jurisdiction boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum Jurisdiction {
    #[serde(rename = "BD")]
    Bangladesh,
    #[serde(rename = "EU")]
    EuropeanUnion,
    #[serde(rename = "US")]
    UnitedStates,
    #[serde(rename = "GLOBAL")]
    #[default]
    Global,
}

/// An individual conversational turn in an LLM dialog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".into(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".into(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".into(),
            content: content.into(),
        }
    }
}

/// Request parameters for AI chat completion with edge routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing_strategy: Option<RoutingStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jurisdiction: Option<Jurisdiction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

impl ChatCompletionRequest {
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            routing_strategy: None,
            jurisdiction: None,
            temperature: None,
            max_tokens: None,
        }
    }

    pub fn with_strategy(mut self, strategy: RoutingStrategy) -> Self {
        self.routing_strategy = Some(strategy);
        self
    }

    pub fn with_jurisdiction(mut self, jurisdiction: Jurisdiction) -> Self {
        self.jurisdiction = Some(jurisdiction);
        self
    }
}

/// Completion candidate returned by the inference gateway.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

/// Edge telemetry emitted by the multi-objective scheduler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionTelemetry {
    pub provider: Option<String>,
    pub cost: Option<f64>,
    pub ttft_ms: Option<f64>,
    pub region: Option<String>,
}

/// Token usage details.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompletionUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Full chat completion response payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: Option<String>,
    pub object: Option<String>,
    pub created: Option<i64>,
    pub model: String,
    pub choices: Vec<CompletionChoice>,
    #[serde(default)]
    pub usage: CompletionUsage,
    pub telemetry: CompletionTelemetry,
}

/// Custom edge domain record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRecord {
    pub id: String,
    pub project_id: String,
    pub hostname: String,
    pub cname_target: Option<String>,
    pub origin_address: Option<String>,
    pub origin_port: Option<u16>,
    pub origin_protocol: Option<String>,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
}

/// Parameters for onboarding a custom domain to the Anycast edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardDomainRequest {
    pub project_id: String,
    pub hostname: String,
    pub origin_address: String,
    pub origin_port: u16,
    pub origin_protocol: Option<String>,
}

/// Verification result for domain DNS validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyDomainResult {
    pub verified: bool,
    pub status: String,
    pub message: Option<String>,
}

/// Authoritative DNS zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsZone {
    pub id: String,
    pub name: String,
    pub status: String,
    pub ttl: u32,
    pub created_at: Option<DateTime<Utc>>,
}

/// DNS resource record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub id: String,
    pub zone_id: String,
    pub name: String,
    pub r#type: String,
    pub content: String,
    pub ttl: u32,
}

/// Parameters for creating a DNS resource record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDnsRecordRequest {
    pub zone_id: String,
    pub name: String,
    pub r#type: String,
    pub content: String,
    pub ttl: Option<u32>,
}

/// Global platform SLA and traffic summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySummary {
    pub global_requests_per_sec: f64,
    pub edge_cache_hit_ratio: f64,
    pub p99_latency_ms: f64,
    pub total_blocked_attacks: i64,
}

/// Health and connectivity status of an edge Point of Presence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopHealthRecord {
    pub pop_id: String,
    pub metro: String,
    pub country: String,
    pub status: String,
    pub latency_ms: f64,
    pub packet_loss: f64,
}

/// Serverless microVM instance record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroVmInstance {
    pub id: String,
    pub project_id: String,
    pub vcpus: u32,
    pub memory_mb: u32,
    pub status: String,
    pub ip_address: Option<String>,
}

/// Distributed storage bucket metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketRecord {
    pub name: String,
    pub region: String,
    pub created_at: DateTime<Utc>,
}

/// Cross-metro WAN latency measurement between edge PoPs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMatrixResult {
    pub source_pop: String,
    pub dest_pop: String,
    pub latency_ms: f64,
    pub jitter_ms: f64,
}
