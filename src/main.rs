use actix_web::{get, web, App, HttpServer, Responder};

#[get("/")]
async fn home() -> impl Responder {
    "IT REALLY IS"
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(home))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
