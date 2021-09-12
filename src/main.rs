#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

mod config;
mod models;
mod schema;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

embed_migrations!();

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let conf = config::Config::new();

    let manager = ConnectionManager::<PgConnection>::new(conf.db_url);
    let pool = r2d2::Pool::builder()
        .min_idle(conf.db_pool_min)
        .max_size(conf.db_pool_max)
        .build(manager)
        .expect("Failed to create connection pool");

    let connection = pool.get().expect("Failed to run migrations");
    web::block(move || embedded_migrations::run(&connection))
        .await
        .expect("Failed to run migrations");

    let server_address = format!("{}:{}", conf.server_bind, conf.server_port);
    println!("Listening requests on {}", server_address);

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(
                web::scope("/api")
                    .configure(models::router)
                    .wrap(Cors::permissive().max_age(3600)),
            )
            .route("/health", web::get().to(health_handler))
            .default_service(web::route().to(HttpResponse::NotFound))
    })
    .bind(server_address)?
    .run()
    .await
}

async fn health_handler() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
