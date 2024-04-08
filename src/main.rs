#![forbid(unsafe_code)]

use simplelog::*;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

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
    )
    .expect("TermLogger has already been created");

    run().await;
}

pub async fn run() {
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
                    .branch(dptree::case![Command::MinasanHelp].endpoint(endpoints::help)),
            ),
        )
        .branch(Update::filter_poll_answer().endpoint(endpoints::update_users));

    let storage = Arc::clone(&chat_storage);

    let mut dispatcher = Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![chat_storage])
        .build();

    let dumper = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(3600)).await;
            storage.dump(Path::new("")).await.unwrap();
            log::info!("Dumped database to disk.");
        }
    });

    tokio::select! {
        _ = dispatcher.dispatch() => {},
        _ = dumper => {},
    }
}
