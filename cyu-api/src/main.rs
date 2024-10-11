pub mod app;
pub mod utils;
pub mod routes;

use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{addr}");

    let app = app::App::new();

    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("Failed to bind listener: {}", err);
            std::process::exit(1);
        }
    };

    let result = axum::serve(listener, routes::get().with_state(app).into_make_service()).await;
    if let Err(err) = result {
        eprintln!("Failed to run server: {}", err);
        std::process::exit(1);
    }
}
