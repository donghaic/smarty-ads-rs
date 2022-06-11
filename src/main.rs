use axum::{
    routing::{get, post},
    Extension, Router,
};
use log;

use crate::{dao::*, service::*};

mod api;
mod dao;
mod model;
mod service;

#[tokio::main]
async fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
    );

    log::info!("----start smarty-adserver---------");
    let redis = redis::Client::open("redis://127.0.0.1").unwrap();

    let ads_db = AdsDB::new(redis.clone());
    let prediction_service = ProdictionService::new(ads_db.clone());

    // build our application with a single route
    let app = Router::new()
        .route("/", get(ping).head(ping))
        .route("/api/predict", post(api::predict))
        .route("/api/test", get(api::test))
        .layer(Extension(ads_db))
        .layer(Extension(prediction_service));

    log::info!("start server on port 3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ping() -> &'static str {
    "Hello, World!"
}
