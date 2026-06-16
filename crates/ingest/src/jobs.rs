use std::{future::Future, time::Duration};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use grand_edge_domain::{IntervalPrice, Item, LatestPrice, PriceInterval};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::{
    IngestError, IntervalBulkResponseRaw, LatestResponseRaw, MappingItemRaw, OsrsWikiClient,
    OsrsWikiConfig, PollingSchedule, TimeseriesCheckpoint, TimeseriesResponseRaw,
    normalize_interval_bulk, normalize_latest, normalize_mapping, normalize_timeseries,
    timeseries_checkpoint_key,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackfillKind {
    Mapping,
    Latest,
    Interval(PriceInterval),
    Timeseries { interval: PriceInterval },
}

#[derive(Debug, Clone)]
pub struct IngestionJobConfig {
    pub poll_latest_seconds: u64,
    pub poll_5m_seconds: u64,
    pub poll_1h_seconds: u64,
    pub sync_mapping_seconds: u64,
    pub max_timeseries_items_per_run: usize,
    pub max_timeseries_requests_per_minute: u32,
    pub item_allowlist: Option<Vec<i64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobReport {
    pub job_name: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub fetched_rows: usize,
    pub written_rows: u64,
    pub skipped_rows: usize,
    pub errors: Vec<String>,
}

#[derive(Clone)]
pub struct IngestionJobs<C = OsrsWikiClient, S = grand_edge_storage::Storage> {
    client: C,
    storage: S,
    config: IngestionJobConfig,
}

#[async_trait]
pub trait WikiClientApi: Clone + Send + Sync + 'static {
    async fn mapping(&self) -> Result<Vec<MappingItemRaw>, IngestError>;
    async fn latest(&self) -> Result<LatestResponseRaw, IngestError>;
    async fn interval_latest(
        &self,
        interval: PriceInterval,
    ) -> Result<IntervalBulkResponseRaw, IngestError>;
    async fn timeseries(
        &self,
        item_id: i64,
        interval: PriceInterval,
    ) -> Result<TimeseriesResponseRaw, IngestError>;
    fn config(&self) -> &OsrsWikiConfig;
}

#[async_trait]
pub trait IngestionStore: Clone + Send + Sync + 'static {
    async fn upsert_items(&self, items: &[Item]) -> Result<u64, IngestError>;
    async fn insert_latest_prices(&self, prices: &[LatestPrice]) -> Result<u64, IngestError>;
    async fn upsert_interval_prices(&self, rows: &[IntervalPrice]) -> Result<u64, IngestError>;
    async fn get_timeseries_checkpoint(
        &self,
        item_id: i64,
        interval: PriceInterval,
    ) -> Result<Option<TimeseriesCheckpoint>, IngestError>;
    async fn set_timeseries_checkpoint(
        &self,
        checkpoint: &TimeseriesCheckpoint,
    ) -> Result<u64, IngestError>;
}

impl IngestionJobConfig {
    pub fn validate(&self, wiki: &OsrsWikiConfig) -> Result<(), IngestError> {
        if self.poll_latest_seconds < 60 {
            return Err(IngestError::InvalidConfig(
                "poll_latest_seconds must be >= 60".to_string(),
            ));
        }
        if self.poll_5m_seconds < 300 {
            return Err(IngestError::InvalidConfig(
                "poll_5m_seconds must be >= 300".to_string(),
            ));
        }
        if self.poll_1h_seconds < 3600 {
            return Err(IngestError::InvalidConfig(
                "poll_1h_seconds must be >= 3600".to_string(),
            ));
        }
        if self.sync_mapping_seconds < 3600 {
            return Err(IngestError::InvalidConfig(
                "sync_mapping_seconds must be >= 3600".to_string(),
            ));
        }
        if self.max_timeseries_items_per_run == 0 {
            return Err(IngestError::InvalidConfig(
                "max_timeseries_items_per_run must be > 0".to_string(),
            ));
        }
        if self.max_timeseries_requests_per_minute == 0 {
            return Err(IngestError::InvalidConfig(
                "max_timeseries_requests_per_minute must be > 0".to_string(),
            ));
        }

        let live_average_rps = (1.0 / self.poll_latest_seconds as f64)
            + (1.0 / self.poll_5m_seconds as f64)
            + (1.0 / self.poll_1h_seconds as f64)
            + (1.0 / self.sync_mapping_seconds as f64);
        let timeseries_rps = self.max_timeseries_requests_per_minute as f64 / 60.0;

        if live_average_rps > wiki.rate_limit.max_requests_per_second {
            return Err(IngestError::InvalidConfig(
                "polling cadence exceeds max_requests_per_second".to_string(),
            ));
        }

        if timeseries_rps > wiki.rate_limit.max_requests_per_second {
            return Err(IngestError::InvalidConfig(
                "timeseries request budget exceeds max_requests_per_second".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for IngestionJobConfig {
    fn default() -> Self {
        Self {
            poll_latest_seconds: 60,
            poll_5m_seconds: 300,
            poll_1h_seconds: 3600,
            sync_mapping_seconds: 86_400,
            max_timeseries_items_per_run: 100,
            max_timeseries_requests_per_minute: 30,
            item_allowlist: None,
        }
    }
}

impl<C, S> IngestionJobs<C, S>
where
    C: WikiClientApi,
    S: IngestionStore,
{
    pub fn new(client: C, storage: S, config: IngestionJobConfig) -> Result<Self, IngestError> {
        config.validate(client.config())?;
        Ok(Self {
            client,
            storage,
            config,
        })
    }

    pub async fn sync_mapping(&self) -> Result<JobReport, IngestError> {
        let started_at = Utc::now();
        let mapping = self.client.mapping().await?;
        let fetched_rows = mapping.len();
        let items = normalize_mapping(mapping, started_at)?;
        let written_rows = self.storage.upsert_items(&items).await?;

        Ok(JobReport {
            job_name: "sync_mapping".to_string(),
            started_at,
            finished_at: Utc::now(),
            fetched_rows,
            written_rows,
            skipped_rows: fetched_rows.saturating_sub(written_rows as usize),
            errors: Vec::new(),
        })
    }

    pub async fn ingest_latest_snapshot(&self) -> Result<JobReport, IngestError> {
        let started_at = Utc::now();
        let response = self.client.latest().await?;
        let fetched_rows = response.data.len();
        let prices = normalize_latest(response, started_at)?;
        let written_rows = self.storage.insert_latest_prices(&prices).await?;

        Ok(JobReport {
            job_name: "ingest_latest".to_string(),
            started_at,
            finished_at: Utc::now(),
            fetched_rows,
            written_rows,
            skipped_rows: fetched_rows.saturating_sub(written_rows as usize),
            errors: Vec::new(),
        })
    }

    pub async fn ingest_interval_snapshot(
        &self,
        interval: PriceInterval,
    ) -> Result<JobReport, IngestError> {
        let started_at = Utc::now();
        let response = self.client.interval_latest(interval).await?;
        let fetched_rows = response.data.len();
        let rows = normalize_interval_bulk(response, interval)?;
        let written_rows = self.storage.upsert_interval_prices(&rows).await?;

        Ok(JobReport {
            job_name: format!("ingest_interval:{interval:?}"),
            started_at,
            finished_at: Utc::now(),
            fetched_rows,
            written_rows,
            skipped_rows: fetched_rows.saturating_sub(written_rows as usize),
            errors: Vec::new(),
        })
    }

    pub async fn backfill_timeseries(
        &self,
        item_ids: &[i64],
        interval: PriceInterval,
    ) -> Result<JobReport, IngestError> {
        let started_at = Utc::now();
        let mut fetched_rows = 0;
        let mut written_rows = 0;
        let mut skipped_rows = 0;
        let mut errors = Vec::new();

        let selected_ids = self.resolve_item_ids(item_ids);
        let max_items = self.config.max_timeseries_items_per_run;
        let request_delay =
            Duration::from_secs_f64(60.0 / self.config.max_timeseries_requests_per_minute as f64);

        for (index, item_id) in selected_ids.iter().copied().enumerate() {
            if index >= max_items {
                skipped_rows += 1;
                continue;
            }

            if self
                .storage
                .get_timeseries_checkpoint(item_id, interval)
                .await?
                .is_some()
            {
                skipped_rows += 1;
                continue;
            }

            match self.client.timeseries(item_id, interval).await {
                Ok(response) => {
                    fetched_rows += response.data.len();
                    let rows = normalize_timeseries(item_id, response, interval)?;
                    written_rows += self.storage.upsert_interval_prices(&rows).await?;
                    self.storage
                        .set_timeseries_checkpoint(&TimeseriesCheckpoint {
                            item_id,
                            interval,
                            completed_at: Utc::now(),
                        })
                        .await?;
                }
                Err(error) => {
                    skipped_rows += 1;
                    errors.push(format!("item {item_id}: {error}"));
                }
            }

            if index + 1 < selected_ids.len().min(max_items) {
                sleep(request_delay).await;
            }
        }

        Ok(JobReport {
            job_name: format!("backfill_timeseries:{interval:?}"),
            started_at,
            finished_at: Utc::now(),
            fetched_rows,
            written_rows,
            skipped_rows,
            errors,
        })
    }

    pub async fn run_polling_loop<F>(&self, shutdown: F) -> Result<(), IngestError>
    where
        F: Future<Output = ()> + Send,
    {
        let mut shutdown = Box::pin(shutdown);
        let mut schedule = PollingSchedule::new(&self.config);

        if let Err(error) = self.sync_mapping().await {
            tracing::warn!("initial mapping sync failed: {error}");
        }

        loop {
            tokio::select! {
                _ = &mut shutdown => return Ok(()),
                _ = schedule.latest.tick() => {
                    self.log_job_result(self.ingest_latest_snapshot().await);
                }
                _ = schedule.five_minute.tick() => {
                    self.log_job_result(self.ingest_interval_snapshot(PriceInterval::FiveMinute).await);
                }
                _ = schedule.one_hour.tick() => {
                    self.log_job_result(self.ingest_interval_snapshot(PriceInterval::OneHour).await);
                }
                _ = schedule.sync_mapping.tick() => {
                    self.log_job_result(self.sync_mapping().await);
                }
            }
        }
    }

    fn resolve_item_ids(&self, item_ids: &[i64]) -> Vec<i64> {
        if item_ids.is_empty() {
            return self.config.item_allowlist.clone().unwrap_or_default();
        }

        item_ids.to_vec()
    }

    fn log_job_result(&self, result: Result<JobReport, IngestError>) {
        match result {
            Ok(report) => tracing::info!(
                job = report.job_name,
                fetched_rows = report.fetched_rows,
                written_rows = report.written_rows,
                skipped_rows = report.skipped_rows,
                "ingestion job completed"
            ),
            Err(error) => tracing::warn!("ingestion job failed: {error}"),
        }
    }
}

#[async_trait]
impl WikiClientApi for OsrsWikiClient {
    async fn mapping(&self) -> Result<Vec<MappingItemRaw>, IngestError> {
        OsrsWikiClient::mapping(self).await
    }

    async fn latest(&self) -> Result<LatestResponseRaw, IngestError> {
        OsrsWikiClient::latest(self).await
    }

    async fn interval_latest(
        &self,
        interval: PriceInterval,
    ) -> Result<IntervalBulkResponseRaw, IngestError> {
        OsrsWikiClient::interval_latest(self, interval).await
    }

    async fn timeseries(
        &self,
        item_id: i64,
        interval: PriceInterval,
    ) -> Result<TimeseriesResponseRaw, IngestError> {
        OsrsWikiClient::timeseries(self, item_id, interval).await
    }

    fn config(&self) -> &OsrsWikiConfig {
        self.config()
    }
}

#[async_trait]
impl IngestionStore for grand_edge_storage::Storage {
    async fn upsert_items(&self, items: &[Item]) -> Result<u64, IngestError> {
        Ok(self.items().upsert_items(items).await?)
    }

    async fn insert_latest_prices(&self, prices: &[LatestPrice]) -> Result<u64, IngestError> {
        Ok(self.prices().insert_latest_prices(prices).await?)
    }

    async fn upsert_interval_prices(&self, rows: &[IntervalPrice]) -> Result<u64, IngestError> {
        Ok(self.prices().upsert_interval_prices(rows).await?)
    }

    async fn get_timeseries_checkpoint(
        &self,
        item_id: i64,
        interval: PriceInterval,
    ) -> Result<Option<TimeseriesCheckpoint>, IngestError> {
        Ok(self
            .checkpoints()
            .get_json(&timeseries_checkpoint_key(item_id, interval))
            .await?
            .map(|stored| stored.value))
    }

    async fn set_timeseries_checkpoint(
        &self,
        checkpoint: &TimeseriesCheckpoint,
    ) -> Result<u64, IngestError> {
        Ok(self
            .checkpoints()
            .set_json(
                &timeseries_checkpoint_key(checkpoint.item_id, checkpoint.interval),
                checkpoint,
            )
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use async_trait::async_trait;
    use chrono::Utc;
    use grand_edge_domain::PriceInterval;
    use tokio::sync::RwLock;

    use super::{IngestionJobConfig, IngestionJobs, IngestionStore, WikiClientApi};
    use crate::{
        IngestError, IntervalBulkResponseRaw, IntervalPriceRaw, LatestPriceRaw, LatestResponseRaw,
        MappingItemRaw, OsrsWikiConfig, TimeseriesCheckpoint, TimeseriesPriceRaw,
        TimeseriesResponseRaw,
    };

    #[derive(Clone)]
    struct MockClient {
        config: OsrsWikiConfig,
        state: Arc<RwLock<MockClientState>>,
    }

    #[derive(Default)]
    struct MockClientState {
        mapping_calls: usize,
        latest_calls: usize,
        interval_calls: Vec<PriceInterval>,
        timeseries_calls: Vec<(i64, PriceInterval)>,
    }

    impl MockClient {
        fn new() -> Self {
            Self {
                config: OsrsWikiConfig::grandedge_default().unwrap(),
                state: Arc::new(RwLock::new(MockClientState::default())),
            }
        }
    }

    #[async_trait]
    impl WikiClientApi for MockClient {
        async fn mapping(&self) -> Result<Vec<MappingItemRaw>, IngestError> {
            self.state.write().await.mapping_calls += 1;
            Ok(vec![MappingItemRaw {
                examine: Some("Abyssal whip".to_string()),
                id: 4151,
                members: true,
                lowalch: Some(72_000),
                limit: Some(8),
                value: Some(120_001),
                highalch: Some(108_001),
                icon: Some("Abyssal whip.png".to_string()),
                name: "Abyssal whip".to_string(),
            }])
        }

        async fn latest(&self) -> Result<LatestResponseRaw, IngestError> {
            self.state.write().await.latest_calls += 1;
            let mut data = HashMap::new();
            data.insert(
                "4151".to_string(),
                LatestPriceRaw {
                    high: Some(2_000_000),
                    high_time: Some(1_700_000_000),
                    low: Some(1_990_000),
                    low_time: Some(1_700_000_001),
                },
            );
            data.insert(
                "1127".to_string(),
                LatestPriceRaw {
                    high: Some(50_000),
                    high_time: Some(1_700_000_000),
                    low: Some(49_000),
                    low_time: Some(1_700_000_002),
                },
            );
            Ok(LatestResponseRaw { data })
        }

        async fn interval_latest(
            &self,
            interval: PriceInterval,
        ) -> Result<IntervalBulkResponseRaw, IngestError> {
            self.state.write().await.interval_calls.push(interval);
            let mut data = HashMap::new();
            data.insert(
                "4151".to_string(),
                IntervalPriceRaw {
                    avg_high_price: Some(2_010_000),
                    high_price_volume: Some(11),
                    avg_low_price: Some(1_980_000),
                    low_price_volume: Some(9),
                },
            );
            Ok(IntervalBulkResponseRaw {
                data,
                timestamp: 1_700_000_300,
            })
        }

        async fn timeseries(
            &self,
            item_id: i64,
            interval: PriceInterval,
        ) -> Result<TimeseriesResponseRaw, IngestError> {
            self.state
                .write()
                .await
                .timeseries_calls
                .push((item_id, interval));
            Ok(TimeseriesResponseRaw {
                data: vec![TimeseriesPriceRaw {
                    timestamp: 1_700_000_000,
                    avg_high_price: Some(100),
                    avg_low_price: Some(90),
                    high_price_volume: Some(10),
                    low_price_volume: Some(8),
                }],
            })
        }

        fn config(&self) -> &OsrsWikiConfig {
            &self.config
        }
    }

    #[derive(Clone, Default)]
    struct MockStore {
        state: Arc<RwLock<MockStoreState>>,
    }

    #[derive(Default)]
    struct MockStoreState {
        items: Vec<grand_edge_domain::Item>,
        latest_batches: Vec<Vec<grand_edge_domain::LatestPrice>>,
        interval_batches: Vec<Vec<grand_edge_domain::IntervalPrice>>,
        checkpoints: HashMap<String, TimeseriesCheckpoint>,
    }

    #[async_trait]
    impl IngestionStore for MockStore {
        async fn upsert_items(
            &self,
            items: &[grand_edge_domain::Item],
        ) -> Result<u64, IngestError> {
            self.state.write().await.items.extend_from_slice(items);
            Ok(items.len() as u64)
        }

        async fn insert_latest_prices(
            &self,
            prices: &[grand_edge_domain::LatestPrice],
        ) -> Result<u64, IngestError> {
            self.state
                .write()
                .await
                .latest_batches
                .push(prices.to_vec());
            Ok(prices.len() as u64)
        }

        async fn upsert_interval_prices(
            &self,
            rows: &[grand_edge_domain::IntervalPrice],
        ) -> Result<u64, IngestError> {
            self.state
                .write()
                .await
                .interval_batches
                .push(rows.to_vec());
            Ok(rows.len() as u64)
        }

        async fn get_timeseries_checkpoint(
            &self,
            item_id: i64,
            interval: PriceInterval,
        ) -> Result<Option<TimeseriesCheckpoint>, IngestError> {
            Ok(self
                .state
                .read()
                .await
                .checkpoints
                .get(&crate::timeseries_checkpoint_key(item_id, interval))
                .cloned())
        }

        async fn set_timeseries_checkpoint(
            &self,
            checkpoint: &TimeseriesCheckpoint,
        ) -> Result<u64, IngestError> {
            self.state.write().await.checkpoints.insert(
                crate::timeseries_checkpoint_key(checkpoint.item_id, checkpoint.interval),
                checkpoint.clone(),
            );
            Ok(1)
        }
    }

    #[test]
    fn default_scheduler_cadence_is_conservative() {
        let config = IngestionJobConfig::default();
        assert_eq!(config.poll_latest_seconds, 60);
        assert_eq!(config.poll_5m_seconds, 300);
        assert_eq!(config.poll_1h_seconds, 3600);
        assert_eq!(config.sync_mapping_seconds, 86_400);
    }

    #[test]
    fn rejects_poll_latest_below_sixty_seconds() {
        let mut config = IngestionJobConfig::default();
        config.poll_latest_seconds = 59;
        let wiki = OsrsWikiConfig::grandedge_default().unwrap();
        assert!(config.validate(&wiki).is_err());
    }

    #[tokio::test]
    async fn sync_mapping_writes_items_once() {
        let jobs = IngestionJobs::new(
            MockClient::new(),
            MockStore::default(),
            IngestionJobConfig::default(),
        )
        .unwrap();

        let report = jobs.sync_mapping().await.unwrap();
        let state = jobs.storage.state.read().await;

        assert_eq!(report.fetched_rows, 1);
        assert_eq!(report.written_rows, 1);
        assert_eq!(state.items.len(), 1);
    }

    #[tokio::test]
    async fn ingest_latest_uses_single_observed_at() {
        let jobs = IngestionJobs::new(
            MockClient::new(),
            MockStore::default(),
            IngestionJobConfig::default(),
        )
        .unwrap();

        let report = jobs.ingest_latest_snapshot().await.unwrap();
        let state = jobs.storage.state.read().await;
        let batch = &state.latest_batches[0];

        assert_eq!(report.fetched_rows, 2);
        assert!(
            batch
                .windows(2)
                .all(|window| window[0].observed_at == window[1].observed_at)
        );
    }

    #[tokio::test]
    async fn backfill_timeseries_reports_skipped_duplicates() {
        let store = MockStore::default();
        store.state.write().await.checkpoints.insert(
            crate::timeseries_checkpoint_key(4151, PriceInterval::OneHour),
            TimeseriesCheckpoint {
                item_id: 4151,
                interval: PriceInterval::OneHour,
                completed_at: Utc::now(),
            },
        );
        let jobs =
            IngestionJobs::new(MockClient::new(), store, IngestionJobConfig::default()).unwrap();

        let report = jobs
            .backfill_timeseries(&[4151, 1127], PriceInterval::OneHour)
            .await
            .unwrap();

        assert_eq!(report.skipped_rows, 1);
        assert_eq!(report.written_rows, 1);
    }

    #[tokio::test]
    async fn timeseries_backfill_uses_request_budget() {
        let mut config = IngestionJobConfig::default();
        config.max_timeseries_items_per_run = 1;
        let jobs = IngestionJobs::new(MockClient::new(), MockStore::default(), config).unwrap();

        let report = jobs
            .backfill_timeseries(&[4151, 1127], PriceInterval::FiveMinute)
            .await
            .unwrap();
        let state = jobs.client.state.read().await;

        assert_eq!(state.timeseries_calls.len(), 1);
        assert_eq!(report.skipped_rows, 1);
    }

    #[tokio::test]
    async fn polling_loop_exits_on_shutdown_signal() {
        let jobs = IngestionJobs::new(
            MockClient::new(),
            MockStore::default(),
            IngestionJobConfig::default(),
        )
        .unwrap();

        jobs.run_polling_loop(async {}).await.unwrap();
    }
}
