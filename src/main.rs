#![forbid(unsafe_code)]

use std::sync::Arc;
use simplelog::*;

use teloxide::prelude::*;

use crate::commands::{endpoints, Command};
use crate::storage::ChatStorage;

mod commands;
mod storage;

#[tokio::main]
async fn main() {

    TermLogger::init(
        LevelFilter::Info,
        ConfigBuilder::default()
            .add_filter_allow("minasan".to_string())
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    ).expect("TermLogger has already been created");

    let bot = Bot::from_env();
    let chat_storage = Arc::new(ChatStorage::new());

    let handler = dptree::entry()
        .branch(
            Update::filter_message().chain(
                teloxide::filter_command::<Command, _>()
                    .branch(dptree::case![Command::MinasanStart].endpoint(endpoints::start))
                    .branch(dptree::case![Command::MinasanRestart].endpoint(endpoints::restart))
                    .branch(dptree::case![Command::MinasanPoll].endpoint(endpoints::get_poll))
                    .branch(dptree::case![Command::MinasanKill].endpoint(endpoints::kill))
                    .branch(dptree::case![Command::Minasan].endpoint(endpoints::tag_everyone))
                    .branch(dptree::case![Command::MinasanHelp].endpoint(endpoints::help))
            ),
        )
        .branch(Update::filter_poll_answer().endpoint(endpoints::update_users));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![chat_storage])
        .build()
        .dispatch()
        .await;
}
