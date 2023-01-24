use sqlx::PgPool;
use sqlx::Row;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{dialogue, Dispatcher, UpdateHandler};
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

use crate::database::{get_all_watch};

#[derive(Clone)]
pub struct BotService {
    pub bot: Bot,
    pub postgres: PgPool,
}

#[shuttle_service::async_trait]
impl shuttle_service::Service for BotService {
    async fn bind(
        mut self: Box<Self>,
        _addr: std::net::SocketAddr,
    ) -> Result<(), shuttle_service::error::Error> {
        self.clone().start().await?;
        Ok(())
    }
}

impl BotService {
    async fn start(&self) -> Result<(), shuttle_service::error::CustomError> {
        let bot = self.bot.clone();

        Dispatcher::builder(bot, answer())
            .dependencies(dptree::deps![
                self.postgres.clone(),
                InMemStorage::<State>::new()
            ])
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(
        description = "Allow me to alert you when a website is down (or up!).",
        parse_with = "split"
    )]
    Watch { status: String, url: String },
    #[command(description = "Stop watching a webpage.")]
    Unwatch(String),
    #[command(description = "List all webpages that I'm watching for you.")]
    List,
    #[command(description = "Stop watching any webpages that you've asked me to watch for you.")]
    Clear,
}

fn answer() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let cmd_handler = teloxide::filter_command::<Command, _>().branch(
        case![State::Start]
                .branch(case![Command::Help].endpoint(help))
                .branch(case![Command::List].endpoint(list))
                .branch(case![Command::Watch {status, url}].endpoint(watch))
                .branch(case![Command::Unwatch (status)].endpoint(unwatch))
                .branch(case![Command::Clear].endpoint(clear))
        );

    dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(cmd_handler)
}

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await
        .expect("Had an error sending message");

    Ok(())
}

async fn list(bot: Bot, msg: Message, dbconn: PgPool) -> HandlerResult {
    let records = get_all_watch(msg.chat.id, dbconn).await?;

    let mut records_vec: Vec<Record> = Vec::new();

    for row in records.iter() {
        let record = Record {
            id: row.get("id"),
            url: row.get("url"),
            status: row.get("status"),
        };

        records_vec.push(record);
    }

    let iterated_records = records_vec.iter().map(|e| format!("ID {}: {} - watching for {}", e.id, e.url, e.status)).collect::<Vec<String>>();

    bot.send_message(
        msg.chat.id,
        format!(
            "I'm currently watching: \n{}",
            iterated_records.join("\n ")
        ),
    )
    .await
    .expect("Had an issue printing out message");
    Ok(())
}

async fn watch(bot: Bot, dbconn: PgPool, msg: Message, status: String, url: String) -> HandlerResult {
        sqlx::query("INSERT INTO links (url, status, user_id) VALUES ($1, $2, $3)")
            .bind(url)
            .bind(status)
            .bind(msg.chat.id.to_string())
            .execute(&dbconn)
            .await?;
        
        bot.send_message(msg.chat.id, "Successfully saved your link.").await?;
    Ok(())
}

async fn unwatch() -> HandlerResult {
    Ok(())
}
async fn clear() -> HandlerResult {
    Ok(())
}

#[derive(Debug)]
pub struct Record {
    id: i32,
    url: String,
    status: String,
}