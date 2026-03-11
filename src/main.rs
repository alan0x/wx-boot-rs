#[macro_use]
extern crate diesel;
extern crate dotenv;

#[macro_use]
extern crate serde;
#[macro_use]
extern crate bitflags;

#[macro_use]
mod macros;

#[macro_use]
extern crate diesel_migrations;

pub(crate) mod context;
pub(crate) mod data;
#[macro_use]
pub(crate) mod db;
pub(crate) mod email;
pub(crate) mod error;
pub(crate) mod helpers;
pub(crate) mod models;
pub(crate) mod redis;
pub(crate) mod routers;
pub(crate) mod schema;
pub(crate) mod things;
pub(crate) mod utils;

mod shared;
use std::env;

use dotenv::dotenv;
use flexi_logger::{FileSpec, Logger};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use tracing_futures::Instrument;

pub use error::Error;
pub use shared::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    user: i64,
    exp: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if dotenv::from_filename(".env.local").is_err() {
        println!("No .env.local file found, using .env file");
    }

    if let Err(e) = dotenv() {
        println!("Error loading .env file: {}", e);
    }

    println!("DATABASE_URL: {}", crate::database_url());
    println!("REDIS_URL: {}", crate::redis_url());
    println!("========================= APP STARTING =======================================");

    let mut build_result = db::build_pool(&crate::database_url());
    while let Err(e) = build_result {
        println!("db connect failed, will try after 10 seconds...");
        println!("error: {:?}", e);
        std::thread::sleep(std::time::Duration::from_secs(10));
        build_result = db::build_pool(&crate::database_url());
    }
    if crate::db::DB_POOL.set(build_result.unwrap()).is_err() {
        println!("set db pool failed");
    } else {
        println!("db connected");
    }

    while let Err(e) = crate::redis::try_init() {
        println!("redis client init failed, will try after 10 seconds...");
        println!("error: {:?}", e);
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
    println!("redis client inited");

    let mut conn = db::connect().unwrap();
    db::migrate(&mut conn);
    println!("db migrated");
    drop(conn);

    // Background: user login state check
    tokio::spawn(async {
        crate::things::user_state::user_state_check().await;
    });

    Logger::try_with_str("info")?
        .log_to_file(
            FileSpec::default()
                .directory(env::var("LOG_LOCATION").unwrap_or(String::from("/data/log_files"))),
        )
        .print_message()
        .start()?;

    let port = env::var("PORT").unwrap_or_else(|_| "7117".to_string());
    let addr = format!("0.0.0.0:{}", port);
    println!("Server listening on {}", addr);

    Server::new(TcpListener::bind(&addr))
        .serve(routers::root())
        .instrument(tracing::info_span!("server.serve"))
        .await;

    Ok(())
}
