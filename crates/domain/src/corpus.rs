use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::GraphDomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusSourceType {
    Wiki,
    OfficialNews,
    ManualCuration,
    MarketAnalysis,
    CompetitorResearch,
    CommunityNote,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorpusSourceEntry {
    pub source_id: String,
    pub title: String,
    pub url: Option<String>,
    pub retrieved_at: Option<DateTime<Utc>>,
    pub license_note: String,
    pub content_hash: String,
    pub source_type: CorpusSourceType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationFile<T> {
    pub schema_version: String,
    pub relation_version: String,
    pub source_ids: Vec<String>,
    pub generated_at: DateTime<Utc>,
    pub entries: Vec<T>,
}

impl CorpusSourceEntry {
    pub fn validate(&self) -> Result<(), GraphDomainError> {
        if self.source_id.trim().is_empty() {
            return Err(GraphDomainError::EmptyField { field: "source_id" });
        }
        if self.title.trim().is_empty() {
            return Err(GraphDomainError::EmptyField { field: "title" });
        }
        if self.license_note.trim().is_empty() {
            return Err(GraphDomainError::EmptyField {
                field: "license_note",
            });
        }
        if self.content_hash.trim().is_empty() {
            return Err(GraphDomainError::EmptyField {
                field: "content_hash",
            });
        }
        if self
            .url
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(GraphDomainError::EmptyField { field: "url" });
        }

        Ok(())
    }
}

impl<T> RelationFile<T> {
    pub fn validate_metadata(&self) -> Result<(), GraphDomainError> {
        if self.schema_version.trim().is_empty() {
            return Err(GraphDomainError::EmptyField {
                field: "schema_version",
            });
        }
        if self.relation_version.trim().is_empty() {
            return Err(GraphDomainError::EmptyField {
                field: "relation_version",
            });
        }
        if self.source_ids.is_empty() {
            return Err(GraphDomainError::EmptyField {
                field: "source_ids",
            });
        }
        if self.source_ids.iter().any(|value| value.trim().is_empty()) {
            return Err(GraphDomainError::EmptyField { field: "source_id" });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::{CorpusSourceEntry, CorpusSourceType, RelationFile};
    use crate::GraphDomainError;

    #[test]
    fn relation_file_requires_source_ids() {
        let file = RelationFile::<serde_json::Value> {
            schema_version: "v1".to_string(),
            relation_version: "relations_v1".to_string(),
            source_ids: Vec::new(),
            generated_at: chrono::Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            entries: Vec::new(),
        };

        assert_eq!(
            file.validate_metadata(),
            Err(GraphDomainError::EmptyField {
                field: "source_ids"
            })
        );
    }

    #[test]
    fn corpus_source_entry_requires_source_id() {
        let entry = CorpusSourceEntry {
            source_id: "   ".to_string(),
            title: "OSRS Wiki".to_string(),
            url: Some("https://oldschool.runescape.wiki".to_string()),
            retrieved_at: None,
            license_note: "CC BY-NC-SA".to_string(),
            content_hash: "abc123".to_string(),
            source_type: CorpusSourceType::Wiki,
        };

        assert_eq!(
            entry.validate(),
            Err(GraphDomainError::EmptyField { field: "source_id" })
        );
    }
}
