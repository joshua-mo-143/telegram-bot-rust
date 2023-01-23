use shuttle_secrets::SecretStore;
use sqlx::PgPool;
use teloxide::prelude::*;

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
