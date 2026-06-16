use chrono::{DateTime, Utc};
use grand_edge_domain::{Gp, Item, ItemIcon, ItemId};
use sqlx::{PgPool, Row};

use crate::StorageError;

#[derive(Clone)]
pub struct ItemRepository {
    pool: PgPool,
}

impl ItemRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_items(&self, items: &[Item]) -> Result<u64, StorageError> {
        let mut affected = 0;

        for item in items {
            let (icon_source_file_name, icon_canonical_file_name, icon_cdn_url, icon_source) =
                split_icon(item.icon.as_ref())?;

            let result = sqlx::query(
                r#"
                INSERT INTO items (
                    item_id, name, examine, members, buy_limit, low_alch, high_alch, value,
                    icon_source_file_name, icon_canonical_file_name, icon_cdn_url, icon_source, updated_at
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8,
                    $9, $10, $11, $12, $13
                )
                ON CONFLICT (item_id) DO UPDATE SET
                    name = EXCLUDED.name,
                    examine = EXCLUDED.examine,
                    members = EXCLUDED.members,
                    buy_limit = EXCLUDED.buy_limit,
                    low_alch = EXCLUDED.low_alch,
                    high_alch = EXCLUDED.high_alch,
                    value = EXCLUDED.value,
                    icon_source_file_name = EXCLUDED.icon_source_file_name,
                    icon_canonical_file_name = EXCLUDED.icon_canonical_file_name,
                    icon_cdn_url = EXCLUDED.icon_cdn_url,
                    icon_source = EXCLUDED.icon_source,
                    updated_at = EXCLUDED.updated_at
                "#,
            )
            .bind(item.item_id.0)
            .bind(&item.name)
            .bind(&item.examine)
            .bind(item.members)
            .bind(item.buy_limit)
            .bind(item.low_alch.map(|value| value.0))
            .bind(item.high_alch.map(|value| value.0))
            .bind(item.value.map(|value| value.0))
            .bind(icon_source_file_name)
            .bind(icon_canonical_file_name)
            .bind(icon_cdn_url)
            .bind(icon_source)
            .bind(item.updated_at)
            .execute(&self.pool)
            .await?;

            affected += result.rows_affected();
        }

        Ok(affected)
    }

    pub async fn get_item(&self, item_id: ItemId) -> Result<Option<Item>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT item_id, name, examine, members, buy_limit, low_alch, high_alch, value,
                   icon_source_file_name, icon_canonical_file_name, icon_cdn_url, icon_source, updated_at
            FROM items
            WHERE item_id = $1
            "#,
        )
        .bind(item_id.0)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_item).transpose()
    }

    pub async fn list_items(&self, limit: i64, offset: i64) -> Result<Vec<Item>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT item_id, name, examine, members, buy_limit, low_alch, high_alch, value,
                   icon_source_file_name, icon_canonical_file_name, icon_cdn_url, icon_source, updated_at
            FROM items
            ORDER BY item_id
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_item).collect()
    }
}

fn split_icon(
    icon: Option<&ItemIcon>,
) -> Result<
    (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ),
    StorageError,
> {
    match icon {
        Some(icon) => Ok((
            Some(icon.source_file_name.clone()),
            Some(icon.canonical_file_name.clone()),
            Some(icon.cdn_url.clone()),
            Some(enum_to_string(&icon.source)?),
        )),
        None => Ok((None, None, None, None)),
    }
}

fn row_to_item(row: sqlx::postgres::PgRow) -> Result<Item, StorageError> {
    Ok(Item {
        item_id: ItemId(row.try_get::<i64, _>("item_id")?),
        name: row.try_get("name")?,
        examine: row.try_get("examine")?,
        members: row.try_get("members")?,
        buy_limit: row.try_get("buy_limit")?,
        low_alch: row.try_get::<Option<i64>, _>("low_alch")?.map(Gp),
        high_alch: row.try_get::<Option<i64>, _>("high_alch")?.map(Gp),
        value: row.try_get::<Option<i64>, _>("value")?.map(Gp),
        icon: join_icon(
            row.try_get("icon_source_file_name")?,
            row.try_get("icon_canonical_file_name")?,
            row.try_get("icon_cdn_url")?,
            row.try_get("icon_source")?,
        )?,
        updated_at: row.try_get::<DateTime<Utc>, _>("updated_at")?,
    })
}

fn join_icon(
    source_file_name: Option<String>,
    canonical_file_name: Option<String>,
    cdn_url: Option<String>,
    source: Option<String>,
) -> Result<Option<ItemIcon>, StorageError> {
    match (source_file_name, canonical_file_name, cdn_url, source) {
        (Some(source_file_name), Some(canonical_file_name), Some(cdn_url), Some(source)) => {
            Ok(Some(ItemIcon {
                source_file_name,
                canonical_file_name,
                cdn_url,
                source: serde_json::from_value(serde_json::Value::String(source))?,
            }))
        }
        _ => Ok(None),
    }
}

fn enum_to_string<T: serde::Serialize>(value: &T) -> Result<String, StorageError> {
    let value = serde_json::to_value(value)?;
    Ok(value
        .as_str()
        .expect("serde rename_all enums serialize to string")
        .to_string())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use grand_edge_domain::{ItemIcon, WikiImageSource};

    use super::{Item, join_icon, split_icon};

    #[test]
    fn item_icon_metadata_round_trips() {
        let icon = ItemIcon {
            source_file_name: "Chef's hat.png".to_string(),
            canonical_file_name: "Chef's_hat.png".to_string(),
            cdn_url: "https://oldschool.runescape.wiki/images/Chef%27s_hat.png".to_string(),
            source: WikiImageSource::MappingIcon,
        };

        let item = Item {
            item_id: grand_edge_domain::ItemId(1949),
            name: "Chef's hat".to_string(),
            examine: None,
            members: false,
            buy_limit: None,
            low_alch: None,
            high_alch: None,
            value: None,
            icon: Some(icon.clone()),
            updated_at: Utc::now(),
        };

        let split = split_icon(item.icon.as_ref()).unwrap();
        let rebuilt = join_icon(split.0, split.1, split.2, split.3).unwrap();
        assert_eq!(rebuilt, Some(icon));
    }
}
