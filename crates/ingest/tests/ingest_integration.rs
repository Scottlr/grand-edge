use std::collections::HashMap;

use async_trait::async_trait;
use grand_edge_domain::PriceInterval;
use grand_edge_ingest::{
    IngestError, IngestionJobConfig, IngestionJobs, IntervalBulkResponseRaw, LatestResponseRaw,
    MappingItemRaw, OsrsWikiConfig, TimeseriesResponseRaw, WikiClientApi,
};
use grand_edge_storage::Storage;

fn database_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

#[derive(Clone)]
struct FixtureClient {
    config: OsrsWikiConfig,
}

#[async_trait]
impl WikiClientApi for FixtureClient {
    async fn mapping(&self) -> Result<Vec<MappingItemRaw>, IngestError> {
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
        Ok(LatestResponseRaw {
            data: HashMap::new(),
        })
    }

    async fn interval_latest(
        &self,
        _interval: PriceInterval,
    ) -> Result<IntervalBulkResponseRaw, IngestError> {
        Ok(IntervalBulkResponseRaw {
            data: HashMap::new(),
            timestamp: 1_700_000_000,
        })
    }

    async fn timeseries(
        &self,
        _item_id: i64,
        _interval: PriceInterval,
    ) -> Result<TimeseriesResponseRaw, IngestError> {
        Ok(TimeseriesResponseRaw { data: Vec::new() })
    }

    fn config(&self) -> &OsrsWikiConfig {
        &self.config
    }
}

#[tokio::test]
#[ignore]
async fn sync_mapping_is_idempotent_against_storage() {
    let Some(database_url) = database_url() else {
        return;
    };

    let storage = Storage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();

    let jobs = IngestionJobs::new(
        FixtureClient {
            config: OsrsWikiConfig::grandedge_default().unwrap(),
        },
        storage.clone(),
        IngestionJobConfig::default(),
    )
    .unwrap();

    let first = jobs.sync_mapping().await.unwrap();
    let second = jobs.sync_mapping().await.unwrap();
    let stored = storage
        .items()
        .get_item(grand_edge_domain::ItemId(4151))
        .await
        .unwrap();

    assert_eq!(first.fetched_rows, 1);
    assert_eq!(second.fetched_rows, 1);
    assert!(stored.is_some());
}
