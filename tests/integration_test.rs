use aeitron::{
    AeitronClient, ChatCompletionRequest, ChatMessage, Jurisdiction, OnboardDomainRequest,
    RoutingStrategy, WebhookVerifier,
};
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use ring::hmac;
use std::convert::Infallible;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;

#[test]
fn test_client_initialization() {
    // Empty key fails
    assert!(AeitronClient::new("").is_err());
    assert!(AeitronClient::new("   ").is_err());

    // Valid initialization with builder
    let client = AeitronClient::builder("atk_live_test_12345")
        .base_url("https://api-stage.aeitron.com")
        .api_version("2026-11-01")
        .timeout(Duration::from_secs(12))
        .max_retries(2)
        .project_id("proj_test")
        .build();

    assert!(client.is_ok());
}

#[test]
fn test_webhook_signature_verification() {
    let secret = "whsec_test_secret_98765";
    let payload = b"{\"event\":\"domain.verified\",\"domain_id\":\"dom_dhaka_1\"}";

    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let signed_payload = format!("{}.{}", now_unix, std::str::from_utf8(payload).unwrap());
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());
    let tag = hmac::sign(&key, signed_payload.as_bytes());
    let valid_sig_hex: String = tag.as_ref().iter().map(|b| format!("{:02x}", b)).collect();

    let valid_header = format!("t={},v1={}", now_unix, valid_sig_hex);

    // 1. Valid signature passes
    let res = WebhookVerifier::verify(payload, &valid_header, secret, Duration::from_secs(300));
    assert!(res.is_ok(), "Valid signature must pass");

    // 2. Tampered payload fails
    let tampered_payload = b"{\"event\":\"domain.verified\",\"domain_id\":\"dom_HACKED\"}";
    let res = WebhookVerifier::verify(
        tampered_payload,
        &valid_header,
        secret,
        Duration::from_secs(300),
    );
    assert!(res.is_err(), "Tampered payload must fail");

    // 3. Expired timestamp fails
    let old_unix = now_unix - 3600;
    let old_signed = format!("{}.{}", old_unix, std::str::from_utf8(payload).unwrap());
    let old_tag = hmac::sign(&key, old_signed.as_bytes());
    let old_sig_hex: String = old_tag
        .as_ref()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    let expired_header = format!("t={},v1={}", old_unix, old_sig_hex);

    let res = WebhookVerifier::verify(payload, &expired_header, secret, Duration::from_secs(300));
    assert!(res.is_err(), "Expired timestamp must fail");

    // 4. Wrong secret fails
    let res = WebhookVerifier::verify(
        payload,
        &valid_header,
        "whsec_wrong_key",
        Duration::from_secs(300),
    );
    assert!(res.is_err(), "Wrong secret must fail");
}

#[tokio::test]
async fn test_inference_chat_completion_mock() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);
        let service = service_fn(|req: Request<hyper::body::Incoming>| async move {
            assert_eq!(req.uri().path(), "/v1/chat/completions");
            assert_eq!(
                req.headers().get("authorization").unwrap(),
                "Bearer nke_live_tokio_client"
            );
            assert!(req
                .headers()
                .get("user-agent")
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("aeitron-rust-sdk/"));

            let resp_json = r#"{
                "id": "cmpl_rust_01",
                "model": "aeitron-fast-l4",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Low-latency response from Dhaka Sovereign Edge"
                    },
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": 10,
                    "completion_tokens": 8,
                    "total_tokens": 18
                },
                "telemetry": {
                    "provider": "aeitron-bd-dac-edge",
                    "cost": 0.00008,
                    "ttft_ms": 11.4,
                    "region": "bd-dac"
                }
            }"#;

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from(resp_json)))
                .unwrap();
            Ok::<_, Infallible>(response)
        });

        hyper_util::server::conn::auto::Builder::new(hyper_util::rt::TokioExecutor::new())
            .serve_connection(io, service)
            .await
            .unwrap();
    });

    let client = AeitronClient::builder("nke_live_tokio_client")
        .base_url(format!("http://{}", addr))
        .build()
        .unwrap();

    let req = ChatCompletionRequest::new(
        "aeitron-fast-l4",
        vec![ChatMessage::user("Analyze BDIX route")],
    )
    .with_strategy(RoutingStrategy::LatencyOptimal)
    .with_jurisdiction(Jurisdiction::Bangladesh);

    let resp = client
        .inference()
        .create_chat_completion(&req)
        .await
        .unwrap();

    assert_eq!(resp.id.as_deref(), Some("cmpl_rust_01"));
    assert_eq!(
        resp.choices[0].message.content,
        "Low-latency response from Dhaka Sovereign Edge"
    );
    assert_eq!(resp.telemetry.region.as_deref(), Some("bd-dac"));
    assert_eq!(resp.telemetry.ttft_ms, Some(11.4));
}

#[tokio::test]
async fn test_domains_and_automatic_retry() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let attempts = Arc::new(AtomicU32::new(0));

    let attempts_clone = attempts.clone();
    tokio::spawn(async move {
        loop {
            let (stream, _) = match listener.accept().await {
                Ok(s) => s,
                Err(_) => break,
            };
            let io = TokioIo::new(stream);
            let att = attempts_clone.clone();

            let service = service_fn(move |req: Request<hyper::body::Incoming>| {
                let att = att.clone();
                async move {
                    if req.uri().path() == "/v1/domains" {
                        let cur = att.fetch_add(1, Ordering::SeqCst);
                        if cur == 0 {
                            // Transient 503 error on first attempt
                            let resp = Response::builder()
                                .status(StatusCode::SERVICE_UNAVAILABLE)
                                .body(Full::new(Bytes::from(r#"{"error":"temporary overload"}"#)))
                                .unwrap();
                            return Ok::<_, Infallible>(resp);
                        }

                        // Success on retry
                        let resp_json = r#"{
                            "id": "dom_rust_99",
                            "project_id": "proj_sovereign",
                            "hostname": "rust.aeitron.com",
                            "status": "ACTIVE"
                        }"#;
                        let resp = Response::builder()
                            .status(StatusCode::OK)
                            .header("content-type", "application/json")
                            .body(Full::new(Bytes::from(resp_json)))
                            .unwrap();
                        return Ok::<_, Infallible>(resp);
                    }

                    let resp = Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Full::new(Bytes::from("not found")))
                        .unwrap();
                    Ok::<_, Infallible>(resp)
                }
            });

            tokio::spawn(async move {
                let _ = hyper_util::server::conn::auto::Builder::new(
                    hyper_util::rt::TokioExecutor::new(),
                )
                .serve_connection(io, service)
                .await;
            });
        }
    });

    let client = AeitronClient::builder("test_key")
        .base_url(format!("http://{}", addr))
        .max_retries(2)
        .build()
        .unwrap();

    let onboard_req = OnboardDomainRequest {
        project_id: "proj_sovereign".into(),
        hostname: "rust.aeitron.com".into(),
        origin_address: "10.0.0.9".into(),
        origin_port: 443,
        origin_protocol: Some("https".into()),
    };

    let domain = client.domains().onboard(&onboard_req).await.unwrap();
    assert_eq!(domain.id, "dom_rust_99");
    assert_eq!(domain.status, "ACTIVE");
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        2,
        "Must retry once and succeed on 2nd attempt"
    );
}
