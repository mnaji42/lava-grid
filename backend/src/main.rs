mod server;
mod game;

use actix_web::{web, App, HttpServer, Responder};

async fn hello_world() -> impl Responder {
    "Hello, world!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(hello_world))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
