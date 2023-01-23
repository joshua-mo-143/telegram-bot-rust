use reqwest::Client;
use sqlx::PgPool;
use sqlx::Row;
use std::{error::Error, time::Duration};
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use tokio::time::sleep;

use crate::database::{create_watch, delete_watch, get_all_watch, sort_data};

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
        let (first, second) = tokio::join!(self.start(), self.monitor());
        match first {
            Ok(result) => println!("{:?}", result),
            Err(err) => println!("{:?}", err),
        };
        match second {
            Ok(result) => println!("{:?}", result),
            Err(err) => println!("{:?}", err),
        };

        Ok(())
    }
}

impl BotService {
    async fn start(&self) -> Result<(), shuttle_service::error::CustomError> {
        let bot = self.bot.clone();
        let db_connection = self.postgres.clone();

        Command::repl(bot, move |bot, msg, cmd| {
            answer(bot, msg, cmd, db_connection.clone())
        })
        .await;

        Ok(())
    }

    async fn monitor(&self) -> Result<(), shuttle_service::error::CustomError> {
        start_monitoring(self.bot.clone(), self.postgres.clone())
            .await
            .expect("Had an issue monitoring the database");
        Ok(())
    }
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

async fn answer(bot: Bot, msg: Message, cmd: Command, db_connection: PgPool) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Watch { status, url } => match status.trim() {
            "up" => {
                create_watch("up".to_string(), url, msg.chat.id, db_connection)
                    .await
                    .expect("Had an issue adding your submission :(");

                bot.send_message(msg.chat.id, "Successfully added your link.".to_string())
                    .await?;
            }
            "down" => {
                create_watch("down".to_string(), url, msg.chat.id, db_connection)
                    .await
                    .expect("Had an issue adding your submission :(");

                bot.send_message(msg.chat.id, "Successfully added your link.".to_string())
                    .await?;
            }
            _ => {
                bot.send_message(
                    msg.chat.id,
                    "You need to tell me if you want to watch for up or down or not!".to_string(),
                )
                .await?;
            }
        },
        Command::Unwatch(url) => {
            delete_watch(url, msg.chat.id, db_connection)
                .await
                .expect("Had an issue unwatching {url}");

            bot.send_message(msg.chat.id, "Successfully unwatched.".to_string())
                .await?;
        }
        Command::List => {
            let records = get_all_watch(msg.chat.id, db_connection)
                .await
                .expect("Had an issue getting any URLs");

            let sorted_data = sort_data(records);

            let meme = sorted_data
                .iter()
                .map(|record| {
                    format!(
                        "ID {}: {} - checking for {}",
                        record.id, record.url, record.status
                    )
                })
                .collect::<Vec<String>>();

            bot.send_message(
                msg.chat.id,
                format!(
                    "Here's the URLs you're currently watching: {}",
                    meme.join("\n")
                ),
            )
            .await?;
        }
        Command::Clear => {
            bot.send_message(msg.chat.id, "Hello world!".to_string())
                .await?;
        }
    }
    Ok(())
}

pub async fn start_monitoring(bot: Bot, db_connection: PgPool) -> Result<(), Box<dyn Error>> {
    loop {
        let records = sqlx::query("SELECT * FROM links")
            .fetch_all(&db_connection)
            .await?;

        for row in records.iter() {
            let url: String = row.get("url");
            let status: String = row.get("status");
            let user_id: String = row.get("user_id");

            let reqwest_client = Client::new();
            let resp = reqwest_client.get(&url).send().await;

            match status.trim() {
                "up" => {
                    if resp.unwrap().status().is_success() {
                        bot.send_message(user_id, format!("{url} is up!"))
                            .await
                            .expect("Had an error trying to send a message");
                    }
                }
                "down" => {
                    if !resp.unwrap().status().is_success() {
                        bot.send_message(user_id, format!("{url} is down!"))
                            .await
                            .expect("Had an error trying to send a message");
                    }
                }
                _ => {}
            }
        }

        sleep(Duration::from_secs(120)).await;
    }
}
