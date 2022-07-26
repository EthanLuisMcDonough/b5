mod config;
mod entities;
mod format;
mod posts;

use self::posts::*;
use actix_files::Files;
use actix_web::{web, App, HttpServer};
use config::CONFIG;
use sea_orm::{Database, DatabaseConnection};
use std::env;

#[derive(Debug, Clone)]
pub struct AppState {
    db: DatabaseConnection,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting {}", CONFIG.title);
    dotenv::dotenv().unwrap();
    let connection_str = env::var("DATABASE_URL").expect("no connection string fond in env");

    let db = Database::connect(&connection_str)
        .await
        .expect("Could not create db");
    println!("Connected to DB. Running server...");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { db: db.clone() }))
            .service(home)
            .service(rss)
            .route("/posts", web::get().to(posts_page))
            .service(post_page)
            .service(Files::new("/static", "./static"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

/*
Levels:
0 - commenter
1 - author
2 - mod
3 - admin
*/
