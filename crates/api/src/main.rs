use grand_edge_api::{app::build_router, config::ApiConfig, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runtime =
        grand_edge_configuration::load_config(grand_edge_configuration::ConfigProfile::Local)?;
    let _ = grand_edge_configuration::init_tracing(&runtime.logging);

    let config = ApiConfig::from_runtime(&runtime);
    let address = config.bind_addr;
    let app_state = AppState::from_config(config.clone()).await?;
    let app = build_router(
        app_state,
        config.cors_origin.clone(),
        config.swagger_ui_enabled,
    );

    tracing::info!("grand-edge-api listening on {address}");
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
