use sqlx::PgPool;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

use crate::database::{create_watch, delete_watch, get_all_watch};

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
        self.start().await?;

        Ok(())
    }
}

impl BotService {
    async fn start(&self) -> Result<(), shuttle_service::error::CustomError> {
        let bot = self.bot.clone();
        let db_connection = self.postgres.clone();
        Command::repl(bot, move |bot, msg, cmd| answer(bot, msg, cmd, db_connection.clone())).await;

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
    #[command(description = "handle a username.")]
    Username(String),
    #[command(description = "handle a username and an age.", parse_with = "split")]
    UsernameAndAge { username: String, age: u8 },
    #[command(description = "Allow me to alert you when a website is down (or up!).", parse_with = "split")]
    Watch {status: String, url: String},
    #[command(description = "Stop watching a webpage.")]
    Unwatch(String),
    #[command(description = "List all webpages that I'm watching for you.")]
    List,
    #[command(description = "Stop watching any webpages that you've asked me to watch for you.")]
    Clear

}

async fn answer(bot: Bot, msg: Message, cmd: Command, db_connection: PgPool) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Username(username) => {
            bot.send_message(msg.chat.id, format!("Your username is @{username}."))
                .await?;
        }
        Command::UsernameAndAge { username, age } => {
            bot.send_message(
                msg.chat.id,
                format!("Your username is @{username} and age is {age}."),
            )
            .await?;
        }
        Command::Watch {status, url} => {
            match status.trim() {
                "up" => {bot.send_message(msg.chat.id, format!("Up")).await?;},
                "down" =>  {bot.send_message(msg.chat.id, format!("Down")).await?;},
                _ => {bot.send_message(msg.chat.id, format!("You need to tell me if you want to watch for up or down or not!")).await?;}
            }

            create_watch(status, url, msg.chat.id, db_connection).await.expect("Had an issue adding your submission :(");

                bot.send_message(msg.chat.id,
                format!("Successfully added your link.")).await?;
            }
        Command::Unwatch(url) => {bot.send_message(msg.chat.id, format!("Hello world")).await?;}
        Command::List => {
            
        }
        Command::Clear => {bot.send_message(msg.chat.id, format!("Hello world")).await?;}
    }
    Ok(())
}