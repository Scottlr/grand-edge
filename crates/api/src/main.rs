use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let address = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("grand-edge-api placeholder listening on {address}");

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("placeholder listener should bind");

    axum::serve(listener, axum::Router::new())
        .await
        .expect("placeholder server should run");
}
