use shuttle_secrets::SecretStore;
use sqlx::PgPool;
use teloxide::prelude::*;
use axum::{Router, routing::{get}};
use std::{net::SocketAddr};

mod bot;
use bot::BotService;

mod database;

#[shuttle_service::main]
async fn init(
    #[shuttle_secrets::Secrets] secrets: SecretStore,
    #[shuttle_shared_db::Postgres] postgres: PgPool,
) -> Result<BotService, shuttle_service::Error> {

    let teloxide_key = secrets
        .get("TELOXIDE_TOKEN")
        .expect("You need a teloxide key set for this to work!");

    Ok(BotService {
        bot: Bot::new(teloxide_key),
        postgres,
    })
}

async fn create_router(addr: SocketAddr) -> Result<(), hyper::Error> {
    let router = Router::new()
            .route("/", get(hello));
    
    let serverbind = axum::Server::bind(&addr).serve(router.into_make_service());
    
    serverbind.await
}

async fn hello() -> &'static str {
    "hello"
}