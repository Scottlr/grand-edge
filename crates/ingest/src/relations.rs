use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use chrono::Utc;
use grand_edge_domain::{
    CorpusSourceEntry, GraphEdgeDirection, GraphEdgeSourceType, GraphEdgeType, GraphVersion,
    ItemGraphEdge, ItemGraphNode, ItemId, RelationFile, validate_edge_confidence,
};
use grand_edge_storage::{Storage, StoredCorpusSource};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::IngestError;

const RELATION_FILES: &[&str] = &[
    "source_registry.v1.json",
    "item_sets.v1.json",
    "recipes.v1.json",
    "repairs.v1.json",
    "alchemy.v1.json",
    "dose_decant.v1.json",
    "charge_links.v1.json",
    "degrade_links.v1.json",
    "categories.v1.json",
    "substitutes.v1.json",
    "market_analysis_sources.v1.json",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemSetRelationEntry {
    pub set_item_id: i64,
    pub component_item_ids: Vec<i64>,
    pub pack_unpack_free: bool,
    pub pack_unpack_instant: bool,
    pub confidence: f64,
    pub source_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeRelationEntry {
    pub output_item_id: i64,
    pub component_item_ids: Vec<i64>,
    pub pack_unpack_free: bool,
    pub pack_unpack_instant: bool,
    pub confidence: f64,
    pub source_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepairRelationEntry {
    pub broken_item_id: i64,
    pub repaired_item_id: i64,
    pub repair_cost_gp: i64,
    pub repair_method: String,
    pub confidence: f64,
    pub source_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlchemyRelationEntry {
    pub item_id: i64,
    pub high_alch_gp: Option<i64>,
    pub low_alch_gp: Option<i64>,
    pub nature_rune_item_id: Option<i64>,
    pub source_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryRelationEntry {
    pub category: String,
    pub item_ids: Vec<i64>,
    pub confidence: f64,
    pub source_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubstituteRelationEntry {
    pub primary_item_id: i64,
    pub substitute_item_ids: Vec<i64>,
    pub confidence: f64,
    pub source_ref: String,
    #[serde(default)]
    pub requires_review: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelationCorpusManifest {
    pub graph_version: GraphVersion,
    pub sources: Vec<StoredCorpusSource>,
    pub nodes: Vec<ItemGraphNode>,
    pub edges: Vec<ItemGraphEdge>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationImportReport {
    pub graph_version: String,
    pub dry_run: bool,
    pub source_count: usize,
    pub node_count: usize,
    pub edge_count: usize,
    pub edge_type_counts: BTreeMap<String, usize>,
    pub source_type_counts: BTreeMap<String, usize>,
}

#[derive(Clone)]
pub struct RelationCorpusImporter<S = Storage> {
    storage: S,
}

#[async_trait]
pub trait RelationImportStore: Clone + Send + Sync + 'static {
    async fn upsert_corpus_sources(
        &self,
        sources: &[StoredCorpusSource],
    ) -> Result<u64, IngestError>;
    async fn insert_graph_version(&self, version: &GraphVersion) -> Result<(), IngestError>;
    async fn upsert_graph_nodes(&self, nodes: &[ItemGraphNode]) -> Result<u64, IngestError>;
    async fn upsert_graph_edges(&self, edges: &[ItemGraphEdge]) -> Result<u64, IngestError>;
}

impl<S> RelationCorpusImporter<S>
where
    S: RelationImportStore,
{
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    pub fn validate_relation_files(root: &Path) -> Result<RelationCorpusManifest, IngestError> {
        let root = canonical_relations_root(root)?;
        for file in RELATION_FILES {
            let path = root.join(file);
            if !path.exists() {
                return Err(IngestError::InvalidRelationCorpus(format!(
                    "missing relation corpus file {}",
                    path.display()
                )));
            }
        }

        let source_registry: RelationFile<CorpusSourceEntry> =
            read_relation_file(&root.join("source_registry.v1.json"))?;
        source_registry.validate_metadata()?;
        let source_entries = source_registry
            .entries
            .iter()
            .map(|entry| {
                entry.validate()?;
                Ok(StoredCorpusSource {
                    source: entry.clone(),
                    metadata: serde_json::json!({
                        "relation_version": source_registry.relation_version,
                        "schema_version": source_registry.schema_version
                    }),
                })
            })
            .collect::<Result<Vec<_>, IngestError>>()?;
        let source_ids = source_entries
            .iter()
            .map(|entry| entry.source.source_id.clone())
            .collect::<BTreeSet<_>>();

        let mut node_categories = BTreeMap::<ItemId, String>::new();
        let mut node_ids = BTreeSet::<ItemId>::new();
        let mut edges = Vec::new();

        let item_sets: RelationFile<ItemSetRelationEntry> =
            read_relation_file(&root.join("item_sets.v1.json"))?;
        validate_source_ids(&item_sets.source_ids, &source_ids)?;
        for entry in &item_sets.entries {
            let set_item_id = parse_item_id(entry.set_item_id)?;
            validate_ref_confidence(&entry.source_ref, entry.confidence, &source_ids)?;
            node_ids.insert(set_item_id);
            for component in &entry.component_item_ids {
                let component_id = parse_item_id(*component)?;
                node_ids.insert(component_id);
                edges.push(build_edge(
                    component_id,
                    set_item_id,
                    GraphEdgeType::ComponentOfSet,
                    GraphEdgeSourceType::Mechanical,
                    GraphEdgeDirection::Upstream,
                    entry.confidence,
                    &entry.source_ref,
                    false,
                    serde_json::json!({
                        "formulaType": "item_set_pack",
                        "componentIds": entry.component_item_ids,
                        "setItemId": entry.set_item_id,
                        "packUnpackFree": entry.pack_unpack_free,
                        "packUnpackInstant": entry.pack_unpack_instant
                    }),
                )?);
            }
        }

        let recipes: RelationFile<RecipeRelationEntry> =
            read_relation_file(&root.join("recipes.v1.json"))?;
        validate_source_ids(&recipes.source_ids, &source_ids)?;
        for entry in &recipes.entries {
            let output_item_id = parse_item_id(entry.output_item_id)?;
            validate_ref_confidence(&entry.source_ref, entry.confidence, &source_ids)?;
            node_ids.insert(output_item_id);
            for component in &entry.component_item_ids {
                let component_id = parse_item_id(*component)?;
                node_ids.insert(component_id);
                edges.push(build_edge(
                    component_id,
                    output_item_id,
                    GraphEdgeType::IngredientOf,
                    GraphEdgeSourceType::Mechanical,
                    GraphEdgeDirection::Upstream,
                    entry.confidence,
                    &entry.source_ref,
                    false,
                    serde_json::json!({
                        "formulaType": "recipe",
                        "componentIds": entry.component_item_ids,
                        "outputItemId": entry.output_item_id
                    }),
                )?);
            }
        }

        let repairs: RelationFile<RepairRelationEntry> =
            read_relation_file(&root.join("repairs.v1.json"))?;
        validate_source_ids(&repairs.source_ids, &source_ids)?;
        for entry in &repairs.entries {
            let broken_item_id = parse_item_id(entry.broken_item_id)?;
            let repaired_item_id = parse_item_id(entry.repaired_item_id)?;
            validate_ref_confidence(&entry.source_ref, entry.confidence, &source_ids)?;
            node_ids.insert(broken_item_id);
            node_ids.insert(repaired_item_id);
            edges.push(build_edge(
                broken_item_id,
                repaired_item_id,
                GraphEdgeType::RepairConversion,
                GraphEdgeSourceType::Mechanical,
                GraphEdgeDirection::Upstream,
                entry.confidence,
                &entry.source_ref,
                false,
                serde_json::json!({
                    "formulaType": "repair_conversion",
                    "brokenItemId": entry.broken_item_id,
                    "repairedItemId": entry.repaired_item_id,
                    "repairCostGp": entry.repair_cost_gp,
                    "repairMethod": entry.repair_method
                }),
            )?);
        }

        let dose_decant: RelationFile<ItemSetRelationEntry> =
            read_relation_file(&root.join("dose_decant.v1.json"))?;
        validate_source_ids(&dose_decant.source_ids, &source_ids)?;
        for entry in &dose_decant.entries {
            let output_item_id = parse_item_id(entry.set_item_id)?;
            validate_ref_confidence(&entry.source_ref, entry.confidence, &source_ids)?;
            node_ids.insert(output_item_id);
            for component in &entry.component_item_ids {
                let component_id = parse_item_id(*component)?;
                node_ids.insert(component_id);
                edges.push(build_edge(
                    component_id,
                    output_item_id,
                    GraphEdgeType::DoseConversion,
                    GraphEdgeSourceType::Mechanical,
                    GraphEdgeDirection::Bidirectional,
                    entry.confidence,
                    &entry.source_ref,
                    false,
                    serde_json::json!({
                        "formulaType": "dose_decant",
                        "componentIds": entry.component_item_ids,
                        "outputItemId": entry.set_item_id
                    }),
                )?);
            }
        }

        let charge_links: RelationFile<ItemSetRelationEntry> =
            read_relation_file(&root.join("charge_links.v1.json"))?;
        validate_source_ids(&charge_links.source_ids, &source_ids)?;
        for entry in &charge_links.entries {
            let output_item_id = parse_item_id(entry.set_item_id)?;
            validate_ref_confidence(&entry.source_ref, entry.confidence, &source_ids)?;
            node_ids.insert(output_item_id);
            for component in &entry.component_item_ids {
                let component_id = parse_item_id(*component)?;
                node_ids.insert(component_id);
                edges.push(build_edge(
                    component_id,
                    output_item_id,
                    GraphEdgeType::ChargeConversion,
                    GraphEdgeSourceType::Mechanical,
                    GraphEdgeDirection::Upstream,
                    entry.confidence,
                    &entry.source_ref,
                    false,
                    serde_json::json!({
                        "formulaType": "charge_link",
                        "componentIds": entry.component_item_ids,
                        "outputItemId": entry.set_item_id
                    }),
                )?);
            }
        }

        let degrade_links: RelationFile<RepairRelationEntry> =
            read_relation_file(&root.join("degrade_links.v1.json"))?;
        validate_source_ids(&degrade_links.source_ids, &source_ids)?;
        for entry in &degrade_links.entries {
            let stable_item_id = parse_item_id(entry.broken_item_id)?;
            let degraded_item_id = parse_item_id(entry.repaired_item_id)?;
            validate_ref_confidence(&entry.source_ref, entry.confidence, &source_ids)?;
            node_ids.insert(stable_item_id);
            node_ids.insert(degraded_item_id);
            edges.push(build_edge(
                stable_item_id,
                degraded_item_id,
                GraphEdgeType::DegradeConversion,
                GraphEdgeSourceType::Mechanical,
                GraphEdgeDirection::Downstream,
                entry.confidence,
                &entry.source_ref,
                false,
                serde_json::json!({
                    "formulaType": "degrade_link",
                    "stableItemId": entry.broken_item_id,
                    "degradedItemId": entry.repaired_item_id,
                    "method": entry.repair_method
                }),
            )?);
        }

        let alchemy: RelationFile<AlchemyRelationEntry> =
            read_relation_file(&root.join("alchemy.v1.json"))?;
        validate_source_ids(&alchemy.source_ids, &source_ids)?;
        for entry in &alchemy.entries {
            let item_id = parse_item_id(entry.item_id)?;
            validate_ref_confidence(&entry.source_ref, 0.9, &source_ids)?;
            node_ids.insert(item_id);
            edges.push(build_edge(
                item_id,
                item_id,
                GraphEdgeType::AlchFloor,
                GraphEdgeSourceType::Mechanical,
                GraphEdgeDirection::Bidirectional,
                0.9,
                &entry.source_ref,
                false,
                serde_json::json!({
                    "formulaType": "alchemy_floor",
                    "itemId": entry.item_id,
                    "highAlchGp": entry.high_alch_gp,
                    "lowAlchGp": entry.low_alch_gp,
                    "natureRuneItemId": entry.nature_rune_item_id
                }),
            )?);
        }

        let categories: RelationFile<CategoryRelationEntry> =
            read_relation_file(&root.join("categories.v1.json"))?;
        validate_source_ids(&categories.source_ids, &source_ids)?;
        for entry in &categories.entries {
            validate_ref_confidence(&entry.source_ref, entry.confidence, &source_ids)?;
            for item_id in &entry.item_ids {
                let item_id = parse_item_id(*item_id)?;
                node_ids.insert(item_id);
                node_categories.insert(item_id, entry.category.clone());
            }
            for pair in entry.item_ids.windows(2) {
                let left = parse_item_id(pair[0])?;
                let right = parse_item_id(pair[1])?;
                edges.push(build_edge(
                    left,
                    right,
                    GraphEdgeType::SameCategory,
                    GraphEdgeSourceType::Curated,
                    GraphEdgeDirection::Bidirectional,
                    entry.confidence,
                    &entry.source_ref,
                    false,
                    serde_json::json!({
                        "formulaType": "category_link",
                        "category": entry.category
                    }),
                )?);
            }
        }

        let substitutes: RelationFile<SubstituteRelationEntry> =
            read_relation_file(&root.join("substitutes.v1.json"))?;
        validate_source_ids(&substitutes.source_ids, &source_ids)?;
        for entry in &substitutes.entries {
            let primary_item_id = parse_item_id(entry.primary_item_id)?;
            validate_ref_confidence(&entry.source_ref, entry.confidence, &source_ids)?;
            node_ids.insert(primary_item_id);
            for substitute in &entry.substitute_item_ids {
                let substitute_id = parse_item_id(*substitute)?;
                node_ids.insert(substitute_id);
                edges.push(build_edge(
                    primary_item_id,
                    substitute_id,
                    GraphEdgeType::Substitute,
                    GraphEdgeSourceType::Curated,
                    GraphEdgeDirection::Bidirectional,
                    entry.confidence,
                    &entry.source_ref,
                    entry.requires_review,
                    serde_json::json!({
                        "formulaType": "substitute_link",
                        "primaryItemId": entry.primary_item_id,
                        "substituteItemId": substitute
                    }),
                )?);
            }
        }

        let market_analysis_sources: RelationFile<SubstituteRelationEntry> =
            read_relation_file(&root.join("market_analysis_sources.v1.json"))?;
        validate_source_ids(&market_analysis_sources.source_ids, &source_ids)?;
        for entry in &market_analysis_sources.entries {
            let primary_item_id = parse_item_id(entry.primary_item_id)?;
            validate_ref_confidence(&entry.source_ref, entry.confidence, &source_ids)?;
            if !entry.requires_review {
                return Err(IngestError::InvalidRelationCorpus(
                    "market_analysis_sources entries must require review".to_string(),
                ));
            }
            node_ids.insert(primary_item_id);
            for candidate in &entry.substitute_item_ids {
                let candidate_id = parse_item_id(*candidate)?;
                node_ids.insert(candidate_id);
                edges.push(build_edge(
                    primary_item_id,
                    candidate_id,
                    GraphEdgeType::CandidateNamePattern,
                    GraphEdgeSourceType::PatternCandidate,
                    GraphEdgeDirection::Bidirectional,
                    entry.confidence,
                    &entry.source_ref,
                    true,
                    serde_json::json!({
                        "formulaType": "candidate_market_analysis_link",
                        "primaryItemId": entry.primary_item_id,
                        "candidateItemId": candidate
                    }),
                )?);
            }
        }

        let graph_version = GraphVersion {
            graph_version: "mechanical_relations_v1".to_string(),
            source_hash: build_source_hash(&source_entries),
            created_at: source_registry.generated_at,
            description: "Curated mechanical relation corpus import".to_string(),
        };
        graph_version.validate()?;

        let nodes = node_ids
            .into_iter()
            .map(|item_id| ItemGraphNode {
                item_id,
                graph_version: graph_version.graph_version.clone(),
                category: node_categories.get(&item_id).cloned(),
                metadata: serde_json::json!({
                    "source": "relation_corpus"
                }),
                updated_at: graph_version.created_at,
            })
            .collect();

        Ok(RelationCorpusManifest {
            graph_version,
            sources: source_entries,
            nodes,
            edges,
        })
    }

    pub async fn import_relation_files(
        &self,
        root: &Path,
        dry_run: bool,
    ) -> Result<RelationImportReport, IngestError> {
        let manifest = Self::validate_relation_files(root)?;
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
        self.storage.upsert_graph_edges(&manifest.edges).await?;

        Ok(report)
    }
}

#[async_trait]
impl RelationImportStore for Storage {
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

    async fn upsert_graph_edges(&self, edges: &[ItemGraphEdge]) -> Result<u64, IngestError> {
        Ok(self.graph().upsert_edges(edges).await?)
    }
}

fn read_relation_file<T>(path: &Path) -> Result<RelationFile<T>, IngestError>
where
    T: for<'de> Deserialize<'de>,
{
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn canonical_relations_root(root: &Path) -> Result<PathBuf, IngestError> {
    let candidate = if root.ends_with("relations") {
        root.to_path_buf()
    } else {
        root.join("data").join("relations")
    };

    Ok(candidate.canonicalize()?)
}

fn validate_source_ids(
    source_ids: &[String],
    allowed_ids: &BTreeSet<String>,
) -> Result<(), IngestError> {
    if source_ids.is_empty() {
        return Err(IngestError::InvalidRelationCorpus(
            "relation file must list at least one source id".to_string(),
        ));
    }

    for source_id in source_ids {
        if !allowed_ids.contains(source_id) {
            return Err(IngestError::InvalidRelationCorpus(format!(
                "unknown source id `{source_id}`"
            )));
        }
    }

    Ok(())
}

fn validate_ref_confidence(
    source_ref: &str,
    confidence: f64,
    allowed_ids: &BTreeSet<String>,
) -> Result<(), IngestError> {
    if source_ref.trim().is_empty() {
        return Err(IngestError::InvalidRelationCorpus(
            "source_ref must not be empty".to_string(),
        ));
    }
    if !allowed_ids.contains(source_ref) {
        return Err(IngestError::InvalidRelationCorpus(format!(
            "unknown source_ref `{source_ref}`"
        )));
    }
    validate_edge_confidence(confidence)?;
    Ok(())
}

fn parse_item_id(value: i64) -> Result<ItemId, IngestError> {
    ItemId::try_from(value).map_err(|source| IngestError::InvalidItemId { value, source })
}

fn build_edge(
    from_item_id: ItemId,
    to_item_id: ItemId,
    edge_type: GraphEdgeType,
    source_type: GraphEdgeSourceType,
    direction: GraphEdgeDirection,
    confidence: f64,
    source_ref: &str,
    requires_review: bool,
    formula: serde_json::Value,
) -> Result<ItemGraphEdge, IngestError> {
    let edge = ItemGraphEdge {
        edge_id: Uuid::new_v4(),
        graph_version: "mechanical_relations_v1".to_string(),
        from_item_id,
        to_item_id,
        edge_type,
        direction,
        sign: 1.0,
        weight: confidence,
        lag_seconds: None,
        confidence,
        source_type,
        source_ref: Some(source_ref.to_string()),
        observations: Vec::new(),
        formula,
        requires_review,
        active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
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

fn report_from_manifest(
    manifest: &RelationCorpusManifest,
    dry_run: bool,
) -> Result<RelationImportReport, IngestError> {
    let mut edge_type_counts = BTreeMap::new();
    let mut source_type_counts = BTreeMap::new();

    for edge in &manifest.edges {
        *edge_type_counts
            .entry(enum_to_string(&edge.edge_type)?)
            .or_insert(0) += 1;
        *source_type_counts
            .entry(enum_to_string(&edge.source_type)?)
            .or_insert(0) += 1;
    }

    Ok(RelationImportReport {
        graph_version: manifest.graph_version.graph_version.clone(),
        dry_run,
        source_count: manifest.sources.len(),
        node_count: manifest.nodes.len(),
        edge_count: manifest.edges.len(),
        edge_type_counts,
        source_type_counts,
    })
}

fn enum_to_string<T: Serialize>(value: &T) -> Result<String, IngestError> {
    let value = serde_json::to_value(value)?;
    Ok(value
        .as_str()
        .expect("serde rename_all enums serialize to string")
        .to_string())
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };

    use async_trait::async_trait;

    use super::*;

    #[derive(Clone, Default)]
    struct MockStore {
        writes: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl RelationImportStore for MockStore {
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

    #[test]
    fn item_set_relation_imports_component_edges() {
        let manifest =
            RelationCorpusImporter::<MockStore>::validate_relation_files(&repo_root()).unwrap();

        assert!(
            manifest
                .edges
                .iter()
                .any(|edge| edge.edge_type == GraphEdgeType::ComponentOfSet)
        );
    }

    #[test]
    fn alchemy_relation_imports_floor_edge() {
        let manifest =
            RelationCorpusImporter::<MockStore>::validate_relation_files(&repo_root()).unwrap();

        assert!(
            manifest
                .edges
                .iter()
                .any(|edge| edge.edge_type == GraphEdgeType::AlchFloor)
        );
    }

    #[test]
    fn repair_relation_imports_conversion_edge() {
        let manifest =
            RelationCorpusImporter::<MockStore>::validate_relation_files(&repo_root()).unwrap();

        assert!(
            manifest
                .edges
                .iter()
                .any(|edge| edge.edge_type == GraphEdgeType::RepairConversion)
        );
    }

    #[test]
    fn name_pattern_candidate_requires_review() {
        let manifest =
            RelationCorpusImporter::<MockStore>::validate_relation_files(&repo_root()).unwrap();

        let candidate = manifest
            .edges
            .iter()
            .find(|edge| edge.edge_type == GraphEdgeType::CandidateNamePattern)
            .unwrap();
        assert_eq!(candidate.source_type, GraphEdgeSourceType::PatternCandidate);
        assert!(candidate.requires_review);
    }

    #[tokio::test]
    async fn relation_import_dry_run_writes_nothing() {
        let store = MockStore::default();
        let importer = RelationCorpusImporter::new(store.clone());

        let report = importer
            .import_relation_files(&repo_root(), true)
            .await
            .unwrap();

        assert!(report.dry_run);
        assert_eq!(store.writes.load(Ordering::SeqCst), 0);
    }
}
