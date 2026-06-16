use grand_edge_domain::{ItemIcon, ItemImageError, WikiImageSource, wiki_image_cdn_url};
use percent_encoding::percent_decode_str;

use crate::IngestError;

pub const OSRS_WIKI_IMAGE_BASE_URL: &str = "https://oldschool.runescape.wiki/images/";

pub fn normalize_wiki_icon_file_name(raw_file_name_or_src: &str) -> Result<String, IngestError> {
    let trimmed = raw_file_name_or_src.trim();
    if trimmed.is_empty() {
        return Err(IngestError::WikiImage(ItemImageError::EmptyFileName));
    }

    let without_query = trimmed.split('?').next().unwrap_or(trimmed);
    let file_name = if let Some(stripped) = without_query.strip_prefix("/images/thumb/") {
        let last_segment = stripped.rsplit('/').next().ok_or_else(|| {
            IngestError::WikiImage(ItemImageError::UnsupportedPath(trimmed.to_string()))
        })?;

        let width_end = last_segment.find("px-").ok_or_else(|| {
            IngestError::WikiImage(ItemImageError::UnsupportedPath(trimmed.to_string()))
        })?;
        &last_segment[(width_end + 3)..]
    } else if let Some(stripped) = without_query.strip_prefix("/images/") {
        stripped.rsplit('/').next().ok_or_else(|| {
            IngestError::WikiImage(ItemImageError::UnsupportedPath(trimmed.to_string()))
        })?
    } else if without_query.starts_with('/') {
        return Err(IngestError::WikiImage(ItemImageError::UnsupportedPath(
            trimmed.to_string(),
        )));
    } else {
        without_query
    };

    let canonical = decode_html_entities(file_name);
    let canonical = percent_decode_str(&canonical).decode_utf8_lossy();
    let canonical = canonical.trim().replace(' ', "_");
    if canonical.is_empty() {
        return Err(IngestError::WikiImage(ItemImageError::EmptyFileName));
    }

    Ok(canonical)
}

pub fn wiki_image_cdn_url_from_file_name(
    raw_file_name_or_src: &str,
) -> Result<String, IngestError> {
    let canonical_file_name = normalize_wiki_icon_file_name(raw_file_name_or_src)?;
    debug_assert!(
        wiki_image_cdn_url(&canonical_file_name)
            .as_deref()
            .is_ok_and(|url| url.starts_with(OSRS_WIKI_IMAGE_BASE_URL))
    );
    wiki_image_cdn_url(&canonical_file_name).map_err(IngestError::from)
}

pub fn item_icon_from_mapping_icon(
    raw_icon: Option<&str>,
) -> Result<Option<ItemIcon>, IngestError> {
    let Some(raw_icon) = raw_icon else {
        return Ok(None);
    };

    let canonical_file_name = normalize_wiki_icon_file_name(raw_icon)?;
    let cdn_url = wiki_image_cdn_url_from_file_name(raw_icon)?;
    Ok(Some(ItemIcon {
        source_file_name: raw_icon.to_owned(),
        canonical_file_name,
        cdn_url,
        source: WikiImageSource::MappingIcon,
    }))
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
    use super::{
        item_icon_from_mapping_icon, normalize_wiki_icon_file_name,
        wiki_image_cdn_url_from_file_name,
    };

    #[test]
    fn normalizes_thumb_paths_to_final_file_name() {
        let value = "/images/thumb/a/a8/Maple_logs_detail.png/120px-Maple_logs_detail.png?4fdfa";
        assert_eq!(
            normalize_wiki_icon_file_name(value).unwrap(),
            "Maple_logs_detail.png"
        );
    }

    #[test]
    fn cdn_url_preserves_percent_encoding_rules() {
        assert_eq!(
            wiki_image_cdn_url_from_file_name("Chef&#39;s hat.png").unwrap(),
            "https://oldschool.runescape.wiki/images/Chef%27s_hat.png"
        );
        assert_eq!(
            wiki_image_cdn_url_from_file_name("Mining_cape%28t%29.png?47004").unwrap(),
            "https://oldschool.runescape.wiki/images/Mining_cape%28t%29.png"
        );
    }

    #[test]
    fn mapping_icon_preserves_original_source_file_name() {
        let icon = item_icon_from_mapping_icon(Some("Chef&#39;s hat.png"))
            .unwrap()
            .unwrap();
        assert_eq!(icon.source_file_name, "Chef&#39;s hat.png");
        assert_eq!(icon.canonical_file_name, "Chef's_hat.png");
    }
}
