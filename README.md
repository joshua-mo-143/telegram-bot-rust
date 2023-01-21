## Rust Telegram Bot
This repo is designed to showcase an example of a Telegram bot created using Rust (with the [Teloxide](https://crates.io/crates/teloxide) crate). 

## Usage Pre-requisites
This project depends on [shuttle](https://www.shuttle.rs) to be used. You'll need to sign in via Github to get an API key to use in their CLI auth so that you can deploy this project.

## How to Deploy
Clone this repository, then create a `Shuttle.toml` file where you'll set your project name and a `Secrets.toml` file where you'll set your Telegram API token to be able to use Teloxide.

Then simply run `cargo shuttle project new` to initialise your project and `cargo shuttle run` to run in local or `cargo shuttle deploy` to deploy! 