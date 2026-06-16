use chrono::{DateTime, Utc};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode};
use serde::{Deserialize, Serialize};

const WIKI_IMAGE_PATH_SET: &AsciiSet = &NON_ALPHANUMERIC.remove(b'-').remove(b'.').remove(b'_');

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub item_id: i64,
    pub name: String,
    pub examine: Option<String>,
    pub members: bool,
    pub buy_limit: Option<i32>,
    pub low_alch: Option<i64>,
    pub high_alch: Option<i64>,
    pub value: Option<i64>,
    pub icon: Option<ItemIcon>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemIcon {
    pub source_file_name: String,
    pub canonical_file_name: String,
    pub cdn_url: String,
    pub source: WikiImageSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WikiImageSource {
    MappingIcon,
    HtmlSourceMatch,
    Missing,
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ItemImageError {
    #[error("empty wiki image file name")]
    EmptyFileName,
    #[error("unsupported wiki image path: {0}")]
    UnsupportedPath(String),
}

pub fn canonical_wiki_file_name(raw_file_name_or_src: &str) -> Result<String, ItemImageError> {
    let trimmed = raw_file_name_or_src.trim();
    if trimmed.is_empty() {
        return Err(ItemImageError::EmptyFileName);
    }

    let without_query = trimmed.split('?').next().unwrap_or(trimmed);
    let path_part = if let Some(stripped) = without_query.strip_prefix("/images/thumb/") {
        let mut segments = stripped.split('/');
        let _width = segments.next();
        segments
            .next()
            .ok_or_else(|| ItemImageError::UnsupportedPath(trimmed.to_string()))?
    } else if let Some(stripped) = without_query.strip_prefix("/images/") {
        stripped
            .rsplit('/')
            .next()
            .ok_or_else(|| ItemImageError::UnsupportedPath(trimmed.to_string()))?
    } else if without_query.starts_with('/') {
        return Err(ItemImageError::UnsupportedPath(trimmed.to_string()));
    } else {
        without_query
    };

    let html_decoded = decode_html_entities(path_part);
    let percent_decoded = percent_decode_str(&html_decoded).decode_utf8_lossy();
    let canonical = percent_decoded.trim().replace(' ', "_");

    if canonical.is_empty() {
        return Err(ItemImageError::EmptyFileName);
    }

    Ok(canonical)
}

pub fn wiki_image_cdn_url(canonical_file_name: &str) -> Result<String, ItemImageError> {
    if canonical_file_name.trim().is_empty() {
        return Err(ItemImageError::EmptyFileName);
    }

    let encoded = utf8_percent_encode(canonical_file_name.trim(), WIKI_IMAGE_PATH_SET).to_string();
    Ok(format!("https://oldschool.runescape.wiki/images/{encoded}"))
}

fn decode_html_entities(input: &str) -> String {
    input
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
}

#[cfg(test)]
mod tests {
    use super::{canonical_wiki_file_name, wiki_image_cdn_url};

    #[test]
    fn canonical_wiki_file_name_decodes_html_apostrophe() {
        assert_eq!(
            canonical_wiki_file_name("Chef&#39;s hat.png").unwrap(),
            "Chef's_hat.png"
        );
    }

    #[test]
    fn wiki_image_cdn_url_encodes_apostrophe_as_percent_27() {
        assert_eq!(
            wiki_image_cdn_url("Chef's_hat.png").unwrap(),
            "https://oldschool.runescape.wiki/images/Chef%27s_hat.png"
        );
    }

    #[test]
    fn canonical_wiki_file_name_strips_cache_hash_query() {
        assert_eq!(
            canonical_wiki_file_name("/images/Bronze_pickaxe.png?fe489").unwrap(),
            "Bronze_pickaxe.png"
        );
    }

    #[test]
    fn wiki_image_cdn_url_encodes_parentheses() {
        assert_eq!(
            wiki_image_cdn_url("Mining_cape(t).png").unwrap(),
            "https://oldschool.runescape.wiki/images/Mining_cape%28t%29.png"
        );
    }

    #[test]
    fn canonical_wiki_file_name_decodes_percent_parentheses() {
        assert_eq!(
            canonical_wiki_file_name("Mining_cape%28t%29.png?47004").unwrap(),
            "Mining_cape(t).png"
        );
    }

    #[test]
    fn canonical_wiki_file_name_replaces_spaces() {
        assert_eq!(
            canonical_wiki_file_name("Bronze pickaxe.png").unwrap(),
            "Bronze_pickaxe.png"
        );
    }
}
