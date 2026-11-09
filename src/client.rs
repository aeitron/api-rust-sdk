use crate::errors::AeitronError;
use crate::types::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

pub const DEFAULT_BASE_URL: &str = "https://dashboard.aeitron.com";
pub const DEFAULT_API_VERSION: &str = "2026-11-01";
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
pub const DEFAULT_MAX_RETRIES: u32 = 3;
pub const SDK_VERSION: &str = "0.1.0";

/// Builder for constructing an [`AeitronClient`] with custom configuration.
pub struct ClientBuilder {
    api_key: String,
    base_url: String,
    api_version: String,
    timeout: Duration,
    max_retries: u32,
    project_id: Option<String>,
}

impl ClientBuilder {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.into(),
            api_version: DEFAULT_API_VERSION.into(),
            timeout: DEFAULT_TIMEOUT,
            max_retries: DEFAULT_MAX_RETRIES,
            project_id: None,
        }
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    pub fn api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = version.into();
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn project_id(mut self, id: impl Into<String>) -> Self {
        self.project_id = Some(id.into());
        self
    }

    pub fn build(self) -> Result<AeitronClient, AeitronError> {
        let key = self.api_key.trim().to_string();
        if key.is_empty() {
            return Err(AeitronError::InvalidApiKey(
                "API key cannot be empty (must provide nke_live_... or atk_live_...)".into(),
            ));
        }

        let http = reqwest::Client::builder().timeout(self.timeout).build()?;

        let inner = ClientInner {
            http,
            api_key: key,
            base_url: self.base_url.trim_end_matches('/').to_string(),
            api_version: self.api_version,
            max_retries: self.max_retries,
            project_id: self.project_id,
        };

        Ok(AeitronClient {
            inner: Arc::new(inner),
        })
    }
}

pub(crate) struct ClientInner {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
    api_version: String,
    max_retries: u32,
    project_id: Option<String>,
}

impl ClientInner {
    pub async fn request<T: DeserializeOwned, B: Serialize>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, AeitronError> {
        let url = format!("{}{}", self.base_url, path);
        let max_attempts = 1 + self.max_retries;
        let mut last_error = None;

        for attempt in 0..max_attempts {
            if attempt > 0 {
                let backoff = Duration::from_millis(100 * (1 << (attempt - 1)));
                tokio::time::sleep(backoff).await;
            }

            let mut req = self
                .http
                .request(method.clone(), &url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header(
                    "User-Agent",
                    format!("aeitron-rust-sdk/{} (Rust 2021)", SDK_VERSION),
                )
                .header("Accept", "application/json");

            if !self.api_version.is_empty() {
                req = req.header("X-API-Version", &self.api_version);
            }
            if let Some(ref pid) = self.project_id {
                req = req.header("X-Project-ID", pid);
            }
            if let Some(b) = body {
                req = req.json(b);
            }

            let resp = match req.send().await {
                Ok(r) => r,
                Err(e) => {
                    last_error = Some(AeitronError::NetworkError(e));
                    continue;
                }
            };

            let status = resp.status();
            if status.is_success() {
                let val = resp.json::<T>().await?;
                return Ok(val);
            }

            // Parse error details
            let request_id = resp
                .headers()
                .get("x-request-id")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let text = resp.text().await.unwrap_or_default();
            let parsed_msg = serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|v| {
                    v.get("message")
                        .or_else(|| v.get("error"))
                        .and_then(|m| m.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or(text);

            let err = match status.as_u16() {
                401 => AeitronError::AuthenticationError {
                    message: parsed_msg,
                    request_id,
                },
                403 => AeitronError::PermissionDeniedError {
                    message: parsed_msg,
                    request_id,
                },
                404 => AeitronError::NotFoundError {
                    message: parsed_msg,
                    request_id,
                },
                409 => AeitronError::ConflictError {
                    message: parsed_msg,
                    request_id,
                },
                429 => AeitronError::RateLimitError {
                    message: parsed_msg,
                    retry_after: Some(Duration::from_secs(1)),
                    request_id,
                },
                code => AeitronError::ServerError {
                    status: code,
                    message: parsed_msg,
                    request_id,
                },
            };

            if (status.as_u16() == 429 || status.is_server_error()) && attempt < max_attempts - 1 {
                last_error = Some(err);
                continue;
            }

            return Err(err);
        }

        Err(last_error.unwrap_or_else(|| AeitronError::ServerError {
            status: 500,
            message: "request failed after retries".into(),
            request_id: None,
        }))
    }
}

/// Official client for the Aeitron Intelligent Edge Platform.
#[derive(Clone)]
pub struct AeitronClient {
    pub(crate) inner: Arc<ClientInner>,
}

impl AeitronClient {
    /// Creates a new client with default options and the specified API key.
    pub fn new(api_key: impl Into<String>) -> Result<Self, AeitronError> {
        ClientBuilder::new(api_key).build()
    }

    /// Returns a [`ClientBuilder`] to configure custom settings.
    pub fn builder(api_key: impl Into<String>) -> ClientBuilder {
        ClientBuilder::new(api_key)
    }

    /// Access multi-objective AI routing and LLM inference endpoints.
    pub fn inference(&self) -> InferenceClient {
        InferenceClient {
            client: self.clone(),
        }
    }

    /// Access custom edge domain onboarding and verification.
    pub fn domains(&self) -> DomainsClient {
        DomainsClient {
            client: self.clone(),
        }
    }

    /// Access Anycast DNS zones and resource records.
    pub fn dns(&self) -> DnsClient {
        DnsClient {
            client: self.clone(),
        }
    }

    /// Access global edge performance and SLA telemetry.
    pub fn telemetry(&self) -> TelemetryClient {
        TelemetryClient {
            client: self.clone(),
        }
    }

    /// Access edge microVM compute instances.
    pub fn compute(&self) -> ComputeClient {
        ComputeClient {
            client: self.clone(),
        }
    }

    /// Access distributed edge object storage.
    pub fn storage(&self) -> StorageClient {
        StorageClient {
            client: self.clone(),
        }
    }

    /// Access international WAN subsea latency matrix.
    pub fn network(&self) -> NetworkClient {
        NetworkClient {
            client: self.clone(),
        }
    }
}

/// Client for multi-objective AI model execution.
#[derive(Clone)]
pub struct InferenceClient {
    client: AeitronClient,
}

impl InferenceClient {
    /// Dispatches a chat completion prompt through the intelligent edge router.
    pub async fn create_chat_completion(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AeitronError> {
        self.client
            .inner
            .request(reqwest::Method::POST, "/v1/chat/completions", Some(request))
            .await
    }
}

/// Client for managing custom edge domains.
#[derive(Clone)]
pub struct DomainsClient {
    client: AeitronClient,
}

impl DomainsClient {
    /// Lists all onboarded domains.
    pub async fn list(&self) -> Result<Vec<DomainRecord>, AeitronError> {
        #[derive(Deserialize)]
        struct Resp {
            domains: Vec<DomainRecord>,
        }
        let resp: Resp = self
            .client
            .inner
            .request(reqwest::Method::GET, "/v1/domains", None::<&()>)
            .await?;
        Ok(resp.domains)
    }

    /// Onboards a new domain to the Anycast edge.
    pub async fn onboard(
        &self,
        request: &OnboardDomainRequest,
    ) -> Result<DomainRecord, AeitronError> {
        self.client
            .inner
            .request(reqwest::Method::POST, "/v1/domains", Some(request))
            .await
    }

    /// Verifies DNS propagation for an edge domain.
    pub async fn verify(&self, domain_id: &str) -> Result<VerifyDomainResult, AeitronError> {
        let path = format!("/v1/domains/{}/verify", domain_id);
        self.client
            .inner
            .request(reqwest::Method::POST, &path, None::<&()>)
            .await
    }
}

/// Client for Anycast DNS management.
#[derive(Clone)]
pub struct DnsClient {
    client: AeitronClient,
}

impl DnsClient {
    /// Lists all authoritative DNS zones.
    pub async fn list_zones(&self) -> Result<Vec<DnsZone>, AeitronError> {
        #[derive(Deserialize)]
        struct Resp {
            zones: Vec<DnsZone>,
        }
        let resp: Resp = self
            .client
            .inner
            .request(reqwest::Method::GET, "/v1/dns/zones", None::<&()>)
            .await?;
        Ok(resp.zones)
    }

    /// Creates a DNS record in a zone.
    pub async fn create_record(
        &self,
        request: &CreateDnsRecordRequest,
    ) -> Result<DnsRecord, AeitronError> {
        let path = format!("/v1/dns/zones/{}/records", request.zone_id);
        self.client
            .inner
            .request(reqwest::Method::POST, &path, Some(request))
            .await
    }
}

/// Client for platform telemetry and SLA metrics.
#[derive(Clone)]
pub struct TelemetryClient {
    client: AeitronClient,
}

impl TelemetryClient {
    /// Returns global platform traffic summary.
    pub async fn get_summary(&self) -> Result<TelemetrySummary, AeitronError> {
        self.client
            .inner
            .request(reqwest::Method::GET, "/v1/telemetry/summary", None::<&()>)
            .await
    }

    /// Returns health telemetry for edge Points of Presence.
    pub async fn get_pop_health(&self) -> Result<Vec<PopHealthRecord>, AeitronError> {
        #[derive(Deserialize)]
        struct Resp {
            pops: Vec<PopHealthRecord>,
        }
        let resp: Resp = self
            .client
            .inner
            .request(reqwest::Method::GET, "/v1/status/pops", None::<&()>)
            .await?;
        Ok(resp.pops)
    }
}

/// Client for edge microVM compute.
#[derive(Clone)]
pub struct ComputeClient {
    client: AeitronClient,
}

impl ComputeClient {
    /// Lists active microVM instances.
    pub async fn list_instances(&self) -> Result<Vec<MicroVmInstance>, AeitronError> {
        #[derive(Deserialize)]
        struct Resp {
            instances: Vec<MicroVmInstance>,
        }
        let resp: Resp = self
            .client
            .inner
            .request(reqwest::Method::GET, "/v1/compute/instances", None::<&()>)
            .await?;
        Ok(resp.instances)
    }
}

/// Client for edge object storage.
#[derive(Clone)]
pub struct StorageClient {
    client: AeitronClient,
}

impl StorageClient {
    /// Lists object storage buckets.
    pub async fn list_buckets(&self) -> Result<Vec<BucketRecord>, AeitronError> {
        #[derive(Deserialize)]
        struct Resp {
            buckets: Vec<BucketRecord>,
        }
        let resp: Resp = self
            .client
            .inner
            .request(reqwest::Method::GET, "/v1/storage/buckets", None::<&()>)
            .await?;
        Ok(resp.buckets)
    }
}

/// Client for WAN network routes and subsea telemetry.
#[derive(Clone)]
pub struct NetworkClient {
    client: AeitronClient,
}

impl NetworkClient {
    /// Returns inter-PoP latency matrix.
    pub async fn get_latency_matrix(&self) -> Result<Vec<LatencyMatrixResult>, AeitronError> {
        #[derive(Deserialize)]
        struct Resp {
            matrix: Vec<LatencyMatrixResult>,
        }
        let resp: Resp = self
            .client
            .inner
            .request(
                reqwest::Method::GET,
                "/v1/network/latency-matrix",
                None::<&()>,
            )
            .await?;
        Ok(resp.matrix)
    }
}
