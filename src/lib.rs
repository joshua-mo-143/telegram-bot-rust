use shuttle_secrets::SecretStore;
use sqlx::PgPool;
use sync_wrapper::SyncWrapper;
use teloxide::prelude::*;
use axum::{Router, routing::get};
use std::error::Error;

mod bot;
use bot::BotService;

mod database;

#[shuttle_service::main]
async fn init(
    #[shuttle_secrets::Secrets] secrets: SecretStore,
    #[shuttle_shared_db::Postgres] postgres: PgPool,
) -> Result<BotService, shuttle_service::Error> {

    create_router().await.expect("An error occurred while setting up the router :(");

    let teloxide_key = secrets
        .get("TELOXIDE_TOKEN")
        .expect("You need a teloxide key set for this to work!");

    Ok(BotService {
        bot: Bot::new(teloxide_key),
        postgres,
    })
}

async fn create_router() -> Result<SyncWrapper<Router>, Box<dyn Error>> {
    let router = Router::new()
            .route("/", get(hello));
    
    let syncwrapper = SyncWrapper::new(router);

    Ok(syncwrapper)
}

async fn hello() -> &'static str {
    "hello"
}