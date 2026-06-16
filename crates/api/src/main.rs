#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config =
        grand_edge_configuration::load_config(grand_edge_configuration::ConfigProfile::Local)?;
    let _ = grand_edge_configuration::init_tracing(&config.logging);

    let address = config.api.bind_addr;
    tracing::info!("grand-edge-api placeholder listening on {address}");

    let listener = tokio::net::TcpListener::bind(address).await?;

    axum::serve(listener, axum::Router::new())
        .await
        .expect("placeholder server should run");
    Ok(())
}
