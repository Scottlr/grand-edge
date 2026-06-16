use std::time::Duration;

use grand_edge_ingest::{LatestResponseRaw, OsrsWikiClient, OsrsWikiConfig};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path},
};

fn client_config(base_url: &str) -> OsrsWikiConfig {
    let mut config = OsrsWikiConfig::grandedge_default().unwrap();
    config.base_url = reqwest::Url::parse(base_url).unwrap();
    config.request_timeout = Duration::from_secs(2);
    config.rate_limit.max_requests_per_second = 100.0;
    config.rate_limit.burst_size = 100;
    config.rate_limit.initial_backoff = Duration::from_millis(1);
    config.rate_limit.max_backoff = Duration::from_millis(5);
    config
}

#[tokio::test]
async fn latest_request_sends_custom_user_agent() {
    let server = MockServer::start().await;
    let config = client_config(&server.uri());

    Mock::given(method("GET"))
        .and(path("/latest"))
        .and(header("user-agent", config.user_agent.as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            include_str!("../../../tests/fixtures/osrs/latest_4151.json"),
            "application/json",
        ))
        .mount(&server)
        .await;

    let client = OsrsWikiClient::new(config).unwrap();
    let latest = client.latest().await.unwrap();

    assert_eq!(latest.data.len(), 1);
}

#[tokio::test]
async fn latest_bulk_response_handles_missing_fields_without_panicking() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/latest"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(r#"{"data":{"4151":{"low":2895000}}}"#, "application/json"),
        )
        .mount(&server)
        .await;

    let client = OsrsWikiClient::new(client_config(&server.uri())).unwrap();
    let latest: LatestResponseRaw = client.latest().await.unwrap();

    assert_eq!(latest.data["4151"].high, None);
    assert_eq!(latest.data["4151"].low, Some(2_895_000));
}

#[tokio::test]
async fn latest_retries_on_server_errors_then_returns_typed_error() {
    let server = MockServer::start().await;
    let mut config = client_config(&server.uri());
    config.rate_limit.max_retries = 1;

    Mock::given(method("GET"))
        .and(path("/latest"))
        .respond_with(ResponseTemplate::new(500).set_body_string("temporary outage"))
        .mount(&server)
        .await;

    let client = OsrsWikiClient::new(config).unwrap();
    let error = client.latest().await.unwrap_err();
    let requests = server.received_requests().await.unwrap();

    assert_eq!(requests.len(), 2);
    assert!(matches!(
        error,
        grand_edge_ingest::IngestError::UnexpectedStatus { .. }
    ));
}
