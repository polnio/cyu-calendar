pub mod app;
pub mod routes;
pub mod utils;

use anyhow::{Context as _, Result};
use std::net::SocketAddr;
use tokio::net::TcpListener;

use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let app = app::App::new()
        .await
        .context("Failed to initialize app state")?;

    let addr = SocketAddr::from(([0, 0, 0, 0], app.env.port));
    println!("Listening on port {}", addr.port());

    let listener = TcpListener::bind(addr)
        .await
        .context("Failed to bind listener")?;

    axum::serve(listener, routes::get().with_state(app).into_make_service())
        .await
        .context("Failed to run server")?;
    Ok(())
}
