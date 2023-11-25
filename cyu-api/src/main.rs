use std::net::SocketAddr;

pub mod app;
pub mod error;
pub mod web;

pub use error::{Error, Result};

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{addr}",);

    let app = app::App::new();

    axum::Server::bind(&addr)
        .serve(web::routes::get().with_state(app).into_make_service())
        .await
        .unwrap();
}
