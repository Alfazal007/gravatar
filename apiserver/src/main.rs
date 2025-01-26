use actix_web::{web, App, HttpServer};
use log::info;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;

pub mod responses;
pub mod routes;
pub mod validation_types;

pub struct AppState {
    pub database_connection_pool: Pool<Postgres>,
    pub access_token_secret: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().expect("Issue getting env files");
    env_logger::Builder::new().parse_filters("info").init();

    let database_url = env::var("DATABASE_URL").expect("Database url not found in the env file");
    let access_token_secret =
        env::var("ACCESS_SECRET").expect("Database url not found in the env file");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Issue connecting to the database");

    info!("Starting Actix Web server...");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                database_connection_pool: pool.clone(),
                access_token_secret: access_token_secret.clone(),
            }))
            .service(
                web::scope("/api/v1/user").route(
                    "/signup",
                    web::post().to(routes::user::create_user::create_user),
                ), /*                    .service(
                       web::scope("/protected")
                           .wrap(from_fn(middlewares::auth_middleware::auth_middleware))
                           .route(
                               "/currentUser",
                               web::get().to(routes::user::current_user::get_current_user),
                           ),
                   ),*/
            )
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
