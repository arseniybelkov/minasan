#![forbid(unsafe_code)]

use clap::Parser;
use simplelog::*;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use teloxide::prelude::*;

use crate::commands::{endpoints, Command};
use crate::storage::ChatStorage;

mod cli;
mod commands;
mod storage;

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();

    TermLogger::init(
        LevelFilter::Info,
        ConfigBuilder::default()
            .add_filter_allow("minasan".to_string())
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .expect("TermLogger has already been created");

    run(args.path, args.interval).await;
}

pub async fn run(path: Option<String>, interval: u16) {
    let bot = Bot::from_env();
    let chat_storage = Arc::new(match path {
        Some(ref p) => {
            log::info!("ChatStorage is loaded from {p}");
            log::info!("Disk dump will happen after {interval} seconds");
            ChatStorage::load(Path::new(p))
        }
        None => {
            log::info!("ChatStorage created anew.");
            ChatStorage::new()
        }
    });

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

    let database_dumper = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(interval as u64)).await;
            if let Some(ref p) = path {
                match storage.dump(Path::new(p)).await {
                    Ok(count) => log::info!("Dumped database ({count} entries) to {p}."),
                    Err(err) => log::warn!("Database dump failed: {err}."),
                }
            } else {
                log::warn!("No db dump happened since no path was specified.");
            }
        }
    });

    let result = tokio::join!(dispatcher.dispatch(), database_dumper);
    if let Err(e) = result.1 {
        panic!("{}", e);
    }
}
