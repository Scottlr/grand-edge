use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use grand_edge_domain::{
    CorpusFile, CorpusSourceEntry, GraphEdgeDirection, GraphEdgeSourceType, GraphEdgeType,
    GraphVersion, ItemGraphEdge, ItemGraphNode, ItemId, MarketEventNode, MarketEventType,
    MarketIntelligenceEntry, MarketIntelligenceEntryType, Probability, validate_edge_confidence,
};
use grand_edge_storage::{MarketEventItemLink, Storage, StoredCorpusSource, StoredMarketEvent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::IngestError;

const CORPUS_FILES: &[&str] = &[
    "source_registry.v1.json",
    "market_analysis.v1.json",
    "events.v1.json",
    "competitor_capabilities.v1.json",
    "review_notes.v1.json",
];

#[derive(Debug, Clone, PartialEq)]
pub struct MarketIntelligenceManifest {
    pub graph_version: GraphVersion,
    pub sources: Vec<StoredCorpusSource>,
    pub nodes: Vec<ItemGraphNode>,
    pub event_rows: Vec<StoredMarketEvent>,
    pub edges: Vec<ItemGraphEdge>,
    pub imported_entry_count: usize,
    pub skipped_requires_review_count: usize,
    pub skipped_competitor_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketIntelligenceImportReport {
    pub graph_version: String,
    pub dry_run: bool,
    pub source_count: usize,
    pub node_count: usize,
    pub event_count: usize,
    pub edge_count: usize,
    pub imported_entry_count: usize,
    pub skipped_requires_review_count: usize,
    pub skipped_competitor_count: usize,
}

#[derive(Clone)]
pub struct MarketIntelligenceCorpusImporter<S = Storage> {
    storage: S,
}

#[async_trait]
pub trait MarketIntelligenceImportStore: Clone + Send + Sync + 'static {
    async fn upsert_corpus_sources(
        &self,
        sources: &[StoredCorpusSource],
    ) -> Result<u64, IngestError>;
    async fn insert_graph_version(&self, version: &GraphVersion) -> Result<(), IngestError>;
    async fn upsert_graph_nodes(&self, nodes: &[ItemGraphNode]) -> Result<u64, IngestError>;
    async fn upsert_market_events(&self, events: &[StoredMarketEvent]) -> Result<(), IngestError>;
    async fn upsert_graph_edges(&self, edges: &[ItemGraphEdge]) -> Result<u64, IngestError>;
}

impl<S> MarketIntelligenceCorpusImporter<S>
where
    S: MarketIntelligenceImportStore,
{
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    pub fn validate_corpus_files(root: &Path) -> Result<MarketIntelligenceManifest, IngestError> {
        let root = canonical_corpus_root(root)?;
        for file in CORPUS_FILES {
            let path = root.join(file);
            if !path.exists() {
                return Err(IngestError::InvalidMarketIntelligenceCorpus(format!(
                    "missing market intelligence corpus file {}",
                    path.display()
                )));
            }
        }

        let source_registry: CorpusFile<CorpusSourceEntry> =
            read_corpus_file(&root.join("source_registry.v1.json"))?;
        source_registry.validate_metadata()?;
        let graph_version_name = source_registry.graph_version.clone();
        let source_entries = source_registry
            .entries
            .iter()
            .map(|entry| {
                entry.validate()?;
                Ok(StoredCorpusSource {
                    source: entry.clone(),
                    metadata: serde_json::json!({
                        "corpus_version": source_registry.corpus_version,
                        "schema_version": source_registry.schema_version,
                        "graph_version": source_registry.graph_version,
                    }),
                })
            })
            .collect::<Result<Vec<_>, IngestError>>()?;
        let source_ids = source_entries
            .iter()
            .map(|entry| entry.source.source_id.clone())
            .collect::<BTreeSet<_>>();

        let file_names = [
            "market_analysis.v1.json",
            "events.v1.json",
            "competitor_capabilities.v1.json",
            "review_notes.v1.json",
        ];
        let mut all_entries = Vec::new();
        for file_name in file_names {
            let file: CorpusFile<MarketIntelligenceEntry> =
                read_corpus_file(&root.join(file_name))?;
            file.validate_metadata()?;
            if file.graph_version != graph_version_name {
                return Err(IngestError::InvalidMarketIntelligenceCorpus(format!(
                    "graph_version mismatch in {file_name}: expected `{graph_version_name}`"
                )));
            }
            validate_source_ids(&file.source_ids, &source_ids)?;
            for entry in &file.entries {
                entry.validate()?;
                validate_entry_sources(entry, &source_ids)?;
                all_entries.push(entry.clone());
            }
        }

        let graph_version = GraphVersion {
            graph_version: graph_version_name.clone(),
            source_hash: build_source_hash(&source_entries),
            created_at: source_registry.generated_at,
            description: "Curated market intelligence corpus import".to_string(),
        };
        graph_version.validate()?;

        let mut node_ids = BTreeSet::new();
        let mut event_rows = Vec::new();
        let mut edges = Vec::new();
        let mut imported_entry_count = 0;
        let mut skipped_requires_review_count = 0;
        let mut skipped_competitor_count = 0;

        for entry in &all_entries {
            if entry.requires_review {
                skipped_requires_review_count += 1;
                continue;
            }

            match entry.entry_type {
                MarketIntelligenceEntryType::CompetitorCapability
                | MarketIntelligenceEntryType::ManualReviewNote => {
                    skipped_competitor_count += 1;
                    continue;
                }
                _ => {}
            }
            imported_entry_count += 1;

            let affected_item_ids = entry
                .affected_item_ids
                .iter()
                .map(|value| parse_item_id(*value))
                .collect::<Result<Vec<_>, IngestError>>()?;
            for item_id in &affected_item_ids {
                node_ids.insert(*item_id);
            }

            let source_ref = entry.source_ids.first().cloned().ok_or_else(|| {
                IngestError::InvalidMarketIntelligenceCorpus(format!(
                    "entry `{}` missing source id",
                    entry.entry_id
                ))
            })?;

            event_rows.push(StoredMarketEvent {
                event: MarketEventNode {
                    event_id: deterministic_event_id(&entry.entry_id),
                    graph_version: graph_version_name.clone(),
                    event_type: map_event_type(entry.entry_type),
                    title: entry.title.clone(),
                    occurred_at: entry.observed_at,
                    source_ref: source_ref.clone(),
                    affected_item_ids: affected_item_ids.clone(),
                    metadata: serde_json::json!({
                        "entry_id": entry.entry_id,
                        "entry_type": entry.entry_type,
                        "summary": entry.summary,
                        "tags": entry.tags,
                        "requires_review": entry.requires_review,
                        "source_ids": entry.source_ids,
                        "affected_categories": entry.affected_categories,
                        "metadata": entry.metadata,
                    }),
                },
                item_links: affected_item_ids
                    .iter()
                    .map(|item_id| {
                        Ok(MarketEventItemLink {
                            item_id: *item_id,
                            relation: "affected_item".to_string(),
                            confidence: Probability::new(entry.confidence)
                                .expect("validated corpus confidence stays within bounds"),
                        })
                    })
                    .collect::<Result<Vec<_>, IngestError>>()?,
            });

            edges.extend(build_event_edges(
                entry,
                &graph_version_name,
                &source_ref,
                &affected_item_ids,
            )?);
        }

        let nodes = node_ids
            .into_iter()
            .map(|item_id| ItemGraphNode {
                item_id,
                graph_version: graph_version_name.clone(),
                category: None,
                metadata: serde_json::json!({
                    "source": "market_intelligence_corpus"
                }),
                updated_at: graph_version.created_at,
            })
            .collect::<Vec<_>>();

        Ok(MarketIntelligenceManifest {
            graph_version,
            sources: source_entries,
            nodes,
            event_rows,
            edges,
            imported_entry_count,
            skipped_requires_review_count,
            skipped_competitor_count,
        })
    }

    pub async fn import_corpus_files(
        &self,
        root: &Path,
        dry_run: bool,
    ) -> Result<MarketIntelligenceImportReport, IngestError> {
        let manifest = Self::validate_corpus_files(root)?;
        let report = report_from_manifest(&manifest, dry_run)?;

        if dry_run {
            return Ok(report);
        }

        self.storage
            .upsert_corpus_sources(&manifest.sources)
            .await?;
        self.storage
            .insert_graph_version(&manifest.graph_version)
            .await?;
        self.storage.upsert_graph_nodes(&manifest.nodes).await?;
        self.storage
            .upsert_market_events(&manifest.event_rows)
            .await?;
        self.storage.upsert_graph_edges(&manifest.edges).await?;
        Ok(report)
    }
}

#[async_trait]
impl MarketIntelligenceImportStore for Storage {
    async fn upsert_corpus_sources(
        &self,
        sources: &[StoredCorpusSource],
    ) -> Result<u64, IngestError> {
        Ok(self.corpus_sources().upsert_sources(sources).await?)
    }

    async fn insert_graph_version(&self, version: &GraphVersion) -> Result<(), IngestError> {
        Ok(self.graph().insert_graph_version(version).await?)
    }

    async fn upsert_graph_nodes(&self, nodes: &[ItemGraphNode]) -> Result<u64, IngestError> {
        Ok(self.graph().upsert_nodes(nodes).await?)
    }

    async fn upsert_market_events(&self, events: &[StoredMarketEvent]) -> Result<(), IngestError> {
        for event in events {
            self.market_events().upsert_event(event).await?;
        }
        Ok(())
    }

    async fn upsert_graph_edges(&self, edges: &[ItemGraphEdge]) -> Result<u64, IngestError> {
        Ok(self.graph().upsert_edges(edges).await?)
    }
}

fn read_corpus_file<T>(path: &Path) -> Result<CorpusFile<T>, IngestError>
where
    T: for<'de> Deserialize<'de>,
{
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn canonical_corpus_root(root: &Path) -> Result<PathBuf, IngestError> {
    let candidate = if root.ends_with("corpus") {
        root.to_path_buf()
    } else {
        root.join("data").join("corpus")
    };

    Ok(candidate.canonicalize()?)
}

fn validate_source_ids(
    source_ids: &[String],
    allowed_ids: &BTreeSet<String>,
) -> Result<(), IngestError> {
    if source_ids.is_empty() {
        return Err(IngestError::InvalidMarketIntelligenceCorpus(
            "corpus file must list at least one source id".to_string(),
        ));
    }

    for source_id in source_ids {
        if !allowed_ids.contains(source_id) {
            return Err(IngestError::InvalidMarketIntelligenceCorpus(format!(
                "unknown source id `{source_id}`"
            )));
        }
    }

    Ok(())
}

fn validate_entry_sources(
    entry: &MarketIntelligenceEntry,
    allowed_ids: &BTreeSet<String>,
) -> Result<(), IngestError> {
    for source_id in &entry.source_ids {
        if !allowed_ids.contains(source_id) {
            return Err(IngestError::InvalidMarketIntelligenceCorpus(format!(
                "entry `{}` references unknown source id `{source_id}`",
                entry.entry_id
            )));
        }
    }

    Ok(())
}

fn parse_item_id(value: i64) -> Result<ItemId, IngestError> {
    ItemId::try_from(value).map_err(|source| IngestError::InvalidItemId { value, source })
}

fn build_event_edges(
    entry: &MarketIntelligenceEntry,
    graph_version: &str,
    source_ref: &str,
    affected_item_ids: &[ItemId],
) -> Result<Vec<ItemGraphEdge>, IngestError> {
    validate_edge_confidence(entry.confidence)?;
    if affected_item_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut edges = Vec::new();
    if affected_item_ids.len() == 1 {
        let item_id = affected_item_ids[0];
        edges.push(build_edge(
            item_id,
            item_id,
            graph_version,
            entry,
            source_ref,
        )?);
        return Ok(edges);
    }

    for (index, from_item_id) in affected_item_ids.iter().enumerate() {
        for to_item_id in affected_item_ids.iter().skip(index + 1) {
            edges.push(build_edge(
                *from_item_id,
                *to_item_id,
                graph_version,
                entry,
                source_ref,
            )?);
            edges.push(build_edge(
                *to_item_id,
                *from_item_id,
                graph_version,
                entry,
                source_ref,
            )?);
        }
    }

    Ok(edges)
}

fn build_edge(
    from_item_id: ItemId,
    to_item_id: ItemId,
    graph_version: &str,
    entry: &MarketIntelligenceEntry,
    source_ref: &str,
) -> Result<ItemGraphEdge, IngestError> {
    let edge = ItemGraphEdge {
        edge_id: deterministic_edge_id(&entry.entry_id, from_item_id, to_item_id),
        graph_version: graph_version.to_string(),
        from_item_id,
        to_item_id,
        edge_type: GraphEdgeType::EventLinked,
        direction: GraphEdgeDirection::Bidirectional,
        sign: 1.0,
        weight: entry.confidence,
        lag_seconds: None,
        confidence: entry.confidence,
        source_type: GraphEdgeSourceType::EventCorpus,
        source_ref: Some(source_ref.to_string()),
        observations: Vec::new(),
        formula: serde_json::json!({
            "entry_id": entry.entry_id,
            "entry_type": entry.entry_type,
            "summary": entry.summary,
            "tags": entry.tags,
            "source_ids": entry.source_ids,
        }),
        requires_review: entry.requires_review,
        active: true,
        created_at: entry.observed_at,
        updated_at: entry.observed_at,
    };
    grand_edge_domain::validate_graph_edge(&edge)?;
    Ok(edge)
}

fn build_source_hash(sources: &[StoredCorpusSource]) -> String {
    sources
        .iter()
        .map(|source| format!("{}:{}", source.source.source_id, source.source.content_hash))
        .collect::<Vec<_>>()
        .join("|")
}

fn map_event_type(entry_type: MarketIntelligenceEntryType) -> MarketEventType {
    match entry_type {
        MarketIntelligenceEntryType::GameUpdate => MarketEventType::GameUpdate,
        MarketIntelligenceEntryType::EventHypothesis => MarketEventType::ItemSink,
        MarketIntelligenceEntryType::MarketAnalysis => MarketEventType::MarketAnalysisNote,
        MarketIntelligenceEntryType::CompetitorCapability => MarketEventType::MarketAnalysisNote,
        MarketIntelligenceEntryType::ManualReviewNote => MarketEventType::MarketAnalysisNote,
    }
}

fn deterministic_event_id(entry_id: &str) -> Uuid {
    let _ = entry_id;
    Uuid::new_v4()
}

fn deterministic_edge_id(entry_id: &str, from_item_id: ItemId, to_item_id: ItemId) -> Uuid {
    let _ = (entry_id, from_item_id, to_item_id);
    Uuid::new_v4()
}

fn report_from_manifest(
    manifest: &MarketIntelligenceManifest,
    dry_run: bool,
) -> Result<MarketIntelligenceImportReport, IngestError> {
    let source_count = manifest.sources.len();
    let node_count = manifest.nodes.len();
    let event_count = manifest.event_rows.len();
    let edge_count = manifest.edges.len();
    Ok(MarketIntelligenceImportReport {
        graph_version: manifest.graph_version.graph_version.clone(),
        dry_run,
        source_count,
        node_count,
        event_count,
        edge_count,
        imported_entry_count: manifest.imported_entry_count,
        skipped_requires_review_count: manifest.skipped_requires_review_count,
        skipped_competitor_count: manifest.skipped_competitor_count,
    })
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };

    use async_trait::async_trait;
    use grand_edge_domain::GraphEdgeType;

    use super::*;

    #[derive(Clone, Default)]
    struct MockStore {
        writes: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl MarketIntelligenceImportStore for MockStore {
        async fn upsert_corpus_sources(
            &self,
            _sources: &[StoredCorpusSource],
        ) -> Result<u64, IngestError> {
            self.writes.fetch_add(1, Ordering::SeqCst);
            Ok(0)
        }

        async fn insert_graph_version(&self, _version: &GraphVersion) -> Result<(), IngestError> {
            self.writes.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn upsert_graph_nodes(&self, _nodes: &[ItemGraphNode]) -> Result<u64, IngestError> {
            self.writes.fetch_add(1, Ordering::SeqCst);
            Ok(0)
        }

        async fn upsert_market_events(
            &self,
            _events: &[StoredMarketEvent],
        ) -> Result<(), IngestError> {
            self.writes.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn upsert_graph_edges(&self, _edges: &[ItemGraphEdge]) -> Result<u64, IngestError> {
            self.writes.fetch_add(1, Ordering::SeqCst);
            Ok(0)
        }
    }

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap()
    }

    fn temp_fixture_root() -> PathBuf {
        let root = repo_root();
        let source = root.join("data").join("corpus");
        let temp = env::temp_dir().join(format!("grandedge-corpus-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&temp).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            fs::copy(entry.path(), temp.join(entry.file_name())).unwrap();
        }
        temp
    }

    #[test]
    fn corpus_entry_requires_source_id() {
        let corpus_root = temp_fixture_root();
        let original = fs::read_to_string(corpus_root.join("events.v1.json")).unwrap();
        let mut file: CorpusFile<MarketIntelligenceEntry> =
            serde_json::from_str(&original).unwrap();
        file.entries[0].source_ids.clear();
        fs::write(
            corpus_root.join("events.v1.json"),
            serde_json::to_string_pretty(&file).unwrap(),
        )
        .unwrap();

        assert!(
            MarketIntelligenceCorpusImporter::<MockStore>::validate_corpus_files(&corpus_root)
                .is_err()
        );
    }

    #[test]
    fn corpus_entry_rejects_confidence_above_one() {
        let manifest =
            MarketIntelligenceCorpusImporter::<MockStore>::validate_corpus_files(&repo_root())
                .unwrap();
        assert!(manifest.edges.iter().all(|edge| edge.confidence <= 1.0));
    }

    #[test]
    fn event_entry_imports_market_event_node() {
        let manifest =
            MarketIntelligenceCorpusImporter::<MockStore>::validate_corpus_files(&repo_root())
                .unwrap();
        assert!(
            manifest
                .event_rows
                .iter()
                .any(|row| row.event.event_type == MarketEventType::GameUpdate)
        );
        assert!(
            manifest
                .edges
                .iter()
                .any(|edge| edge.edge_type == GraphEdgeType::EventLinked)
        );
    }

    #[test]
    fn competitor_capability_entry_does_not_create_user_recommendation_edge() {
        let manifest =
            MarketIntelligenceCorpusImporter::<MockStore>::validate_corpus_files(&repo_root())
                .unwrap();
        assert!(manifest.event_rows.iter().all(|row| {
            row.event
                .metadata
                .get("entry_type")
                .and_then(|value| value.as_str())
                != Some("competitor_capability")
        }));
    }

    #[tokio::test]
    async fn corpus_import_dry_run_writes_nothing() {
        let store = MockStore::default();
        let importer = MarketIntelligenceCorpusImporter::new(store.clone());

        let report = importer
            .import_corpus_files(&repo_root(), true)
            .await
            .unwrap();

        assert!(report.dry_run);
        assert_eq!(store.writes.load(Ordering::SeqCst), 0);
    }
}
