use teloxide::utils::command::BotCommands;
use shuttle_secrets::SecretStore;
use shuttle_service;
use teloxide::prelude::*;

pub struct BotService {
    pub bot: Bot
}

#[shuttle_service::main]
async fn init(
    #[shuttle_secrets::Secrets] secrets: SecretStore,
) -> Result<BotService, shuttle_service::Error> {

    let teloxide_key = secrets.get("TELOXIDE_TOKEN").expect("You need a teloxide key set for this to work!");
    
    Ok(BotService { bot: Bot::new(teloxide_key) })
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
        Command::repl(self.bot.clone(), answer).await;
        Ok(())  
    }
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "handle a username.")]
    Username(String),
    #[command(description = "handle a username and an age.", parse_with = "split")]
    UsernameAndAge { username: String, age: u8 },
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
        Command::Username(username) => {
            bot.send_message(msg.chat.id, format!("Your username is @{username}.")).await?
        }
        Command::UsernameAndAge { username, age } => {
            bot.send_message(msg.chat.id, format!("Your username is @{username} and age is {age}."))
                .await?
        }
    };

    Ok(())
}