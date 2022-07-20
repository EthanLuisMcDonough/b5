mod entities;
mod posts;

use self::posts::*;
use actix_files::Files;
use actix_web::{
    web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result as ActixResult,
};
use sea_orm::{Database, DatabaseConnection};
use std::env;

#[derive(Debug, Clone)]
pub struct AppState {
    db: DatabaseConnection,
}

pub fn tagline() -> &'static str {
    use rand::seq::IteratorRandom;
    let mut rng = rand::thread_rng();
    include_str!("../config/taglines.txt")
        .split('\n')
        .filter(|e| e.len() > 0)
        .choose(&mut rng)
        .unwrap_or("No taglines found")
}

pub fn page_title() -> &'static str {
    include_str!("../config/site_title.txt")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().unwrap();
    let connection_str = env::var("DATABASE_URL").expect("no connection string fond in env");

    let db = Database::connect(&connection_str)
        .await
        .expect("Could not create db");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { db: db.clone() }))
            .service(home)
            .route("/posts", web::get().to(posts_page))
            .service(post_page)
            .service(Files::new("/static", "./static"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
