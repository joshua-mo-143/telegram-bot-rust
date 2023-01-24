use sqlx::PgPool;
use sqlx::Row;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{dialogue, Dispatcher, UpdateHandler};
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

use crate::database::{create_watch, delete_watch, get_all_watch, sort_data};

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
    Start,
    ReceiveFullName
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
        );
    
    let message_handler = Update::filter_message()
            .branch(cmd_handler)
            .branch(case![State::ReceiveFullName].endpoint(receive_full_name));
    

    dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(message_handler)
    // match cmd {
    //     Command::Help => {
    //         bot.send_message(msg.chat.id, Command::descriptions().to_string())
    //             .await?;
    //     }
    //     Command::Watch { status, url } => match status.trim() {
    //         "up" => {
    //             create_watch("up".to_string(), url, msg.chat.id, db_connection)
    //                 .await
    //                 .expect("Had an issue adding your submission :(");

    //             bot.send_message(msg.chat.id, "Successfully added your link.".to_string())
    //                 .await?;
    //         }
    //         "down" => {
    //             create_watch("down".to_string(), url, msg.chat.id, db_connection)
    //                 .await
    //                 .expect("Had an issue adding your submission :(");

    //             bot.send_message(msg.chat.id, "Successfully added your link.".to_string())
    //                 .await?;
    //         }
    //         _ => {
    //             bot.send_message(
    //                 msg.chat.id,
    //                 "You need to tell me if you want to watch for up or down or not!".to_string(),
    //             )
    //             .await?;
    //         }
    //     },
    //     Command::Unwatch(url) => {
    //         delete_watch(url, msg.chat.id, db_connection)
    //             .await
    //             .expect("Had an issue unwatching {url}");

    //         bot.send_message(msg.chat.id, "Successfully unwatched.".to_string())
    //             .await?;
    //     }
    //     Command::List => {
    //         let records = get_all_watch(msg.chat.id, db_connection)
    //             .await
    //             .expect("Had an issue getting any URLs");

    //         let sorted_data = sort_data(records);

    //         let meme = sorted_data
    //             .iter()
    //             .map(|record| {
    //                 format!(
    //                     "ID {}: {} - checking for {}",
    //                     record.id, record.url, record.status
    //                 )
    //             })
    //             .collect::<Vec<String>>();

    //         bot.send_message(
    //             msg.chat.id,
    //             format!(
    //                 "Here's the URLs you're currently watching: {}",
    //                 meme.join("\n")
    //             ),
    //         )
    //         .await?;
    //     }
    //     Command::Clear => {
    //         bot.send_message(msg.chat.id, "Hello world!".to_string())
    //             .await?;
    //     }
    // }
}

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, format!("hello"))
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

    bot.send_message(
        msg.chat.id,
        format!(
            "I'm currently watching: {:?}",
            records_vec
                .iter()
                .map(|e| format!("ID {}: {} - watching for {}", e.id, e.url, e.status))
        ),
    )
    .await
    .expect("Had an issue printing out message");
    Ok(())
}

#[derive(Debug)]
pub struct Record {
    id: i32,
    url: String,
    status: String,
}