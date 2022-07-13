use actix_web::{get, web, App, HttpServer, Responder};

use actix_files::Files;

#[get("/")]
async fn home() -> impl Responder {
    "IT REALLY IS"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(home)
            .service(Files::new("/static", "./static"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
