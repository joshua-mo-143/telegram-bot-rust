use reqwest::Client;
use sqlx::PgPool;
use sqlx::Row;
use std::sync::Arc;
use std::{error::Error, time::Duration};
use teloxide::payloads::SendMessage;
use teloxide::prelude::*;
use teloxide::requests::JsonRequest;
use teloxide::utils::command::BotCommands;
use tokio::time::sleep;

use crate::database::{create_record, delete_record, get_all_records, sort_data};

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
        let share_self = Arc::new(self);

        let background_task = share_self.clone();
        tokio::spawn(async move {
            Arc::clone(&share_self)
                .start()
                .await
                .expect("An error ocurred while using the bot!");
        });

        background_task.monitor().await?;

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
            "up" | "down" => {
                create_record(status, url, msg.chat.id, db_connection)
                    .await
                    .expect("Had an issue adding your submission :(");

                bot.send_message(msg.chat.id, "Successfully added your link.".to_string())
                    .await?;
            },
            _ => {
                bot.send_message(
                    msg.chat.id,
                    "You need to tell me if you want to watch for up or down or not!".to_string(),
                )
                .await?;
            }
        },
        Command::Unwatch(url) => {
            delete_record(url, msg.chat.id, db_connection)
                .await
                .expect("Had an issue unwatching {url}");

            bot.send_message(msg.chat.id, "Successfully unwatched.".to_string())
                .await?;
        }
        Command::List => {
            let records = get_all_records(msg.chat.id, db_connection)
                .await
                .expect("Had an issue getting any URLs");

            let sorted_data = sort_data(records);

            let data_to_strings = sorted_data
                .iter()
                .map(|record| {
                    format!(
                        "ID {}: {} - checking for {}",
                        record.id, record.url, record.status
                    )
                })
                .collect::<Vec<String>>();

            let data_to_strings = format!(
                "Here's the URLs you're currently watching: \n{}",
                data_to_strings.join("\n")
            );

            send_message_without_link_preview(bot, msg.chat.id, data_to_strings)
                .await
                .expect("Oh no! There was an error sending a list message");
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

async fn send_message_without_link_preview(
    bot: Bot,
    user_id: ChatId,
    msg: String,
) -> Result<(), Box<dyn Error>> {
    let message_to_send = SendMessage {
        chat_id: user_id.into(),
        text: msg,
        disable_web_page_preview: Some(true),
        message_thread_id: None,
        entities: None,
        parse_mode: None,
        disable_notification: Some(false),
        protect_content: Some(true),
        reply_to_message_id: None,
        reply_markup: None,
        allow_sending_without_reply: Some(false),
    };

    let request = JsonRequest::new(bot, message_to_send);

    request.send().await.unwrap();

    Ok(())
}
