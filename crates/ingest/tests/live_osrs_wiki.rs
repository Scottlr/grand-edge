use grand_edge_ingest::{OsrsWikiClient, OsrsWikiConfig};

#[tokio::test]
#[ignore = "live_osrs_wiki"]
async fn live_osrs_wiki() {
    let client = OsrsWikiClient::new(OsrsWikiConfig::grandedge_default().unwrap()).unwrap();
    let latest = client.latest_for_item_debug(4151).await.unwrap();
    assert!(latest.data.contains_key("4151"));
}
