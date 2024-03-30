use std::sync::Arc;
use teloxide::RequestError;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use crate::storage::ChatStorage;

#[derive(BotCommands, Debug, PartialEq)]
#[command(rename_rule = "snake_case")]
pub enum Command {
    Minasan,
    MinasanPoll,
    MinasanStart,
    MinasanRestart,
}

pub async fn start(bot: Arc<Bot>, message: Message, chat_storage: ChatStorage) -> Result<(), RequestError> {
    if chat_storage.get_message_id(message.chat.id).await.is_some() {
        bot.send_message(message.chat.id, "\
            You have already started the poll, if you want to restart, \
            use the `/restart` command.\
        ").await?;
    } else {
        chat_storage.add_chat(message.chat.id, message.id).await;
    }
    Ok(())
}

pub async fn restart(bot: Arc<Bot>, chat_id: ChatId, chat_storage: ChatStorage) -> Result<(), RequestError> {
    let message_id = chat_storage.get_message_id(chat_id).await;

    if message_id.is_none() {
        bot.send_message(chat_id, "You haven't started working with me!\
            Use the `/start` command.").await?;
        return Ok(())
    }

    let message_id = message_id.unwrap();
    bot.delete_message(chat_id, message_id).await?;
    chat_storage.clean_users(chat_id).await;
    create_poll(bot, chat_id, chat_storage).await?;
    Ok(())
}

pub async fn kill(bot: Arc<Bot>, chat_id: ChatId, chat_storage: ChatStorage) -> Result<(), RequestError> {
    chat_storage.remove_chat(chat_id).await;
    bot.send_message(chat_id, "I will work here no more!").await?;
    Ok(())
}

pub async fn tag_everyone(bot: Arc<Bot>, chat_id: ChatId, chat_storage: ChatStorage) -> Result<(), RequestError> {
    let users = chat_storage.get_users(chat_id).await.unwrap();
    
    let message = if users.is_empty() {
        String::from("No user provided any @username!!!")
    } else {
        users.into_iter().map(|s| format!("@{s}")).collect::<Vec<String>>().join(" ")
    };
    
    bot.send_message(chat_id, message).await?;
    Ok(())
}

pub async fn update_users(bot: Arc<Bot>, chat_id: ChatId, chat_storage: ChatStorage, poll_anwser: PollAnswer) -> Result<(), RequestError> {
    if chat_storage.get_message_id(chat_id).await.is_none() {
        return Ok(())
    }
    chat_storage.update_users(chat_id, vec![poll_anwser.user.username.unwrap()]).await;
    Ok(())
}

pub async fn get_poll(bot: Arc<Bot>, chat_id: ChatId, chat_storage: ChatStorage) -> Result<(), RequestError> {
    let message_id = chat_storage.get_message_id(chat_id).await;
    if message_id.is_none() {
        bot.send_message(chat_id, "You haven't started any poll.\
            Please, use the `/start` command.").await?;
        return Ok(())
    }

    let message_id = message_id.unwrap();
    bot.send_message(chat_id, "Here's your poll.").await?;
    bot.forward_message(chat_id, chat_id, message_id).await?;
    Ok(())
}

async fn create_poll(bot: Arc<Bot>, chat_id: ChatId, chat_storage: ChatStorage) -> Result<(), RequestError> {
    let question_str = "\
        Do you consent to be tagged by `minasan` bot,\
        via submission of your @username?\
        ";

    let poll_options = [String::from("I do."), String::from("I don't.")];
    
    let request = bot.send_poll(chat_id, question_str, poll_options).await?;
    chat_storage.update_message(chat_id, request.id).await;
    Ok(())
}
