use actix_web::{middleware::from_fn, web, App, HttpServer};
use helpers::generate_id::Snowflake;
use log::info;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{
    env,
    sync::{Arc, Mutex},
};

pub mod dbcalls;
pub mod helpers;
pub mod middlewares;
pub mod models;
pub mod responses;
pub mod routes;
pub mod validation_types;

pub struct AppState {
    pub database_connection_pool: Pool<Postgres>,
    pub access_token_secret: String,
    pub snow_flake: Arc<Mutex<Snowflake>>,
    pub redis_conn: r2d2::Pool<redis::Client>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().expect("Issue getting env files");
    env_logger::Builder::new().parse_filters("info").init();

    let database_url = env::var("DATABASE_URL").expect("Database url not found in the env file");
    let redis_url = env::var("REDIS_URL").expect("Redis url not found in the env file");
    let access_token_secret =
        env::var("ACCESS_SECRET").expect("Database url not found in the env file");
    let machine_id: u64 = env::var("MACHINE_ID")
        .expect("Machine id not found in the env file")
        .parse()
        .expect("Invalid machine id");

    if machine_id > 1023 {
        panic!("Machine id should be between 0 and 1024");
    }

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Issue connecting to the database");

    let redis_client = redis::Client::open(redis_url).expect("Issue creating redis client");
    let redis_conn = r2d2::Pool::builder()
        .max_size(5)
        .build(redis_client)
        .expect("Issue connecting to redis");

    info!("Starting Actix Web server...");
    let snowflake = Arc::new(Mutex::new(Snowflake {
        machine_id,
        counter: 0,
    }));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                database_connection_pool: pool.clone(),
                access_token_secret: access_token_secret.clone(),
                snow_flake: snowflake.clone(),
                redis_conn: redis_conn.clone(),
            }))
            .service(
                web::scope("/api/v1/user")
                    .route(
                        "/signup",
                        web::post().to(routes::user::create_user::create_user),
                    )
                    .route(
                        "/signin",
                        web::post().to(routes::user::login_user::login_user),
                    )
                    .service(
                        web::scope("/protected")
                            .wrap(from_fn(middlewares::auth_middleware::auth_middleware))
                            .route(
                                "/currentUser",
                                web::get().to(routes::user::current_user::get_current_user),
                            ),
                    ),
            )
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
