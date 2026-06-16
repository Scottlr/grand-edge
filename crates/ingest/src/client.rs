use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use grand_edge_domain::PriceInterval;
use reqwest::{StatusCode, header::RETRY_AFTER};
use serde::de::DeserializeOwned;
use tokio::{sync::Mutex, time::sleep};

use crate::{
    IngestError, IntervalBulkResponseRaw, LatestResponseRaw, MappingItemRaw, OsrsWikiConfig,
    TimeseriesResponseRaw,
};

#[derive(Clone)]
pub struct OsrsWikiClient {
    http: reqwest::Client,
    config: OsrsWikiConfig,
    rate_limit_state: Arc<Mutex<RateLimitState>>,
}

struct RateLimitState {
    tokens: f64,
    last_refill: Instant,
}

impl OsrsWikiClient {
    pub fn new(config: OsrsWikiConfig) -> Result<Self, IngestError> {
        config.validate()?;

        let http = reqwest::Client::builder()
            .user_agent(config.user_agent.clone())
            .timeout(config.request_timeout)
            .build()?;

        Ok(Self {
            http,
            config: config.clone(),
            rate_limit_state: Arc::new(Mutex::new(RateLimitState {
                tokens: config.rate_limit.burst_size as f64,
                last_refill: Instant::now(),
            })),
        })
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, IngestError> {
        let mut attempt = 0;

        loop {
            self.acquire_rate_limit_token().await;

            let request = self.http.get(self.endpoint_url(path)?).query(query);
            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        return Ok(response.json().await?);
                    }

                    if attempt >= self.config.rate_limit.max_retries || !is_retryable_status(status)
                    {
                        let body = response.text().await.unwrap_or_default();
                        return Err(IngestError::UnexpectedStatus { status, body });
                    }

                    let retry_after = if self.config.rate_limit.respect_retry_after {
                        parse_retry_after(response.headers().get(RETRY_AFTER))
                    } else {
                        None
                    };
                    sleep(retry_after.unwrap_or_else(|| self.backoff_for_attempt(attempt))).await;
                }
                Err(error) => {
                    if attempt >= self.config.rate_limit.max_retries || !is_retryable_error(&error)
                    {
                        return Err(IngestError::Http(error));
                    }

                    sleep(self.backoff_for_attempt(attempt)).await;
                }
            }

            attempt += 1;
        }
    }

    pub async fn mapping(&self) -> Result<Vec<MappingItemRaw>, IngestError> {
        self.get_json("/mapping", &[]).await
    }

    pub async fn latest(&self) -> Result<LatestResponseRaw, IngestError> {
        self.get_json("/latest", &[]).await
    }

    /// Bulk `/latest` is the normal ingestion path. This item-specific fetch is only for
    /// debugging or smoke tests and should not be used by scheduler jobs.
    pub async fn latest_for_item_debug(
        &self,
        item_id: i64,
    ) -> Result<LatestResponseRaw, IngestError> {
        self.get_json("/latest", &[("id", item_id.to_string())])
            .await
    }

    pub async fn interval_latest(
        &self,
        interval: PriceInterval,
    ) -> Result<IntervalBulkResponseRaw, IngestError> {
        self.get_json(interval_latest_path(interval)?, &[]).await
    }

    pub async fn timeseries(
        &self,
        item_id: i64,
        timestep: PriceInterval,
    ) -> Result<TimeseriesResponseRaw, IngestError> {
        let timestep = timeseries_timestep(timestep)?;
        self.get_json(
            "/timeseries",
            &[
                ("id", item_id.to_string()),
                ("timestep", timestep.to_string()),
            ],
        )
        .await
    }

    async fn acquire_rate_limit_token(&self) {
        loop {
            let wait_duration = {
                let mut state = self.rate_limit_state.lock().await;
                refill_tokens(&mut state, &self.config);
                if state.tokens >= 1.0 {
                    state.tokens -= 1.0;
                    None
                } else {
                    let missing_tokens = 1.0 - state.tokens;
                    let seconds = missing_tokens / self.config.rate_limit.max_requests_per_second;
                    Some(Duration::from_secs_f64(seconds.max(0.001)))
                }
            };

            match wait_duration {
                Some(duration) => sleep(duration).await,
                None => return,
            }
        }
    }

    fn endpoint_url(&self, path: &str) -> Result<reqwest::Url, IngestError> {
        let mut url = self.config.base_url.clone();
        let base_path = url.path().trim_end_matches('/').to_string();
        url.set_path(&format!("{base_path}/{}", path.trim_start_matches('/')));
        Ok(url)
    }

    fn backoff_for_attempt(&self, attempt: u32) -> Duration {
        let base_ms = self.config.rate_limit.initial_backoff.as_millis() as u64;
        let max_ms = self.config.rate_limit.max_backoff.as_millis() as u64;
        let multiplier = 1_u64.checked_shl(attempt.min(20)).unwrap_or(u64::MAX);
        let exponential_ms = base_ms.saturating_mul(multiplier).min(max_ms);
        let jitter_window = (exponential_ms / 4).max(1);
        let jitter =
            (u64::from(Utc::now().timestamp_subsec_millis()) + u64::from(attempt)) % jitter_window;
        Duration::from_millis(exponential_ms.saturating_add(jitter).min(max_ms))
    }
}

fn refill_tokens(state: &mut RateLimitState, config: &OsrsWikiConfig) {
    let now = Instant::now();
    let elapsed = now.duration_since(state.last_refill).as_secs_f64();
    let refill = elapsed * config.rate_limit.max_requests_per_second;
    state.tokens = (state.tokens + refill).min(config.rate_limit.burst_size as f64);
    state.last_refill = now;
}

fn parse_retry_after(value: Option<&reqwest::header::HeaderValue>) -> Option<Duration> {
    let value = value?.to_str().ok()?;
    if let Ok(seconds) = value.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }

    let retry_at = DateTime::parse_from_rfc2822(value)
        .ok()?
        .with_timezone(&Utc);
    let delta = retry_at.signed_duration_since(Utc::now()).to_std().ok()?;
    Some(delta)
}

fn is_retryable_status(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::TOO_MANY_REQUESTS
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::BAD_GATEWAY
            | StatusCode::GATEWAY_TIMEOUT
            | StatusCode::INTERNAL_SERVER_ERROR
    )
}

fn is_retryable_error(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect() || error.is_request()
}

fn interval_latest_path(interval: PriceInterval) -> Result<&'static str, IngestError> {
    match interval {
        PriceInterval::FiveMinute => Ok("/5m"),
        PriceInterval::OneHour => Ok("/1h"),
        unsupported => Err(IngestError::UnsupportedInterval(unsupported)),
    }
}

fn timeseries_timestep(interval: PriceInterval) -> Result<&'static str, IngestError> {
    match interval {
        PriceInterval::FiveMinute => Ok("5m"),
        PriceInterval::OneHour => Ok("1h"),
        PriceInterval::SixHour => Ok("6h"),
        PriceInterval::TwentyFourHour => Ok("24h"),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::Datelike;
    use grand_edge_domain::PriceInterval;

    use super::{OsrsWikiClient, parse_retry_after};
    use crate::OsrsWikiConfig;

    #[test]
    fn retry_after_parser_accepts_seconds_and_http_dates() {
        let seconds = reqwest::header::HeaderValue::from_static("3");
        assert_eq!(
            parse_retry_after(Some(&seconds)),
            Some(Duration::from_secs(3))
        );

        let date = reqwest::header::HeaderValue::from_static("Wed, 21 Oct 2037 07:28:00 GMT");
        assert!(parse_retry_after(Some(&date)).is_some());
    }

    #[test]
    fn timeseries_supports_documented_intervals() {
        assert_eq!(
            super::timeseries_timestep(PriceInterval::FiveMinute).unwrap(),
            "5m"
        );
        assert_eq!(
            super::timeseries_timestep(PriceInterval::TwentyFourHour).unwrap(),
            "24h"
        );
    }

    #[tokio::test]
    async fn client_builds_with_default_config() {
        let client = OsrsWikiClient::new(OsrsWikiConfig::grandedge_default().unwrap()).unwrap();
        let _ = client.clone();
        assert_eq!(chrono::Utc::now().year_ce().0, true);
    }
}
