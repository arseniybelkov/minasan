use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::RequestError;
use teloxide::utils::command::BotCommands;

use crate::storage::ChatStorage;

#[derive(BotCommands, Debug, PartialEq, Clone)]
#[command(rename_rule = "snake_case")]
pub enum Command {
    /// Tag everyone.
    Minasan,
    /// Stops the bot and removes it from the chat. 
    MinasanKill,
    /// Resend currently active poll to this chat.
    MinasanPoll,
    /// Bot creates poll and starts tracking users.
    MinasanStart,
    /// Restarts the bot, recreating the poll.
    MinasanRestart,
}

pub mod endpoints {
    use teloxide::payloads::SendPoll;
    use teloxide::requests::JsonRequest;
    use teloxide::types::{MediaKind, Message, MessageId, MessageKind};

    use super::*;

    pub async fn start(
        bot: Bot,
        message: Message,
        chat_storage: Arc<ChatStorage>,
    ) -> Result<(), RequestError> {
        let chat_id = message.chat.id;

        if chat_storage.get_message_id(chat_id).await.is_some() {
            bot.send_message(
                chat_id,
                "\
                You have already started the poll, if you want to restart, \
                use the `/minasan_restart` command.\
            ",
            )
            .await?;
        } else {
            let message_id = create_poll(bot, chat_id, chat_storage.clone()).await?;
            chat_storage.add_chat(chat_id, message_id).await;
        }
        Ok(())
    }

    pub async fn restart(
        bot: Bot,
        message: Message,
        chat_storage: Arc<ChatStorage>,
    ) -> Result<(), RequestError> {
        let chat_id = message.chat.id;
        let message_id = chat_storage.get_message_id(chat_id).await;

        if message_id.is_none() {
            bot.send_message(
                chat_id,
                "You haven't started working with me! \
                Use the `/minasan_start` command.",
            )
            .await?;
            return Ok(());
        }

        let message_id = message_id.unwrap();
        bot.delete_message(chat_id, message_id).await?;
        chat_storage.clean_users(chat_id).await;
        create_poll(bot, chat_id, chat_storage).await?;
        Ok(())
    }

    pub async fn kill(
        bot: Bot,
        message: Message,
        chat_storage: Arc<ChatStorage>,
    ) -> Result<(), RequestError> {
        let poll_message_id = chat_storage.get_message_id(message.chat.id).await;

        if let Some(poll_message_id) = poll_message_id {
            bot.delete_message(message.chat.id, poll_message_id).await?;
            chat_storage.remove_chat(message.chat.id).await;
        }
        bot.send_message(message.chat.id, "I will work here no more!")
            .await?;
        bot.leave_chat(message.chat.id).await?;
        Ok(())
    }

    pub async fn tag_everyone(
        bot: Bot,
        message: Message,
        chat_storage: Arc<ChatStorage>,
    ) -> Result<(), RequestError> {
        let chat_id = message.chat.id;
        let users = chat_storage.get_users(chat_id).await.unwrap();

        let message = if users.is_empty() {
            String::from("No user provided any @username!!!")
        } else {
            users
                .into_iter()
                .map(|s| format!("@{s}"))
                .collect::<Vec<String>>()
                .join(" ")
        };

        bot.send_message(chat_id, message).await?;
        Ok(())
    }

    pub async fn update_users(
        _bot: Bot,
        chat_storage: Arc<ChatStorage>,
        poll_answer: PollAnswer,
    ) -> Result<(), RequestError> {
        let chat_id = chat_storage.poll2chat(&poll_answer.poll_id).await;

        if chat_storage.get_message_id(chat_id).await.is_some() {
            chat_storage
                .update_users(chat_id, vec![poll_answer.user.username.unwrap()])
                .await;
        }
        Ok(())
    }

    pub async fn get_poll(
        bot: Bot,
        message: Message,
        chat_storage: Arc<ChatStorage>,
    ) -> Result<(), RequestError> {
        let chat_id = message.chat.id;

        let message_id = chat_storage.get_message_id(chat_id).await;
        if message_id.is_none() {
            bot.send_message(
                chat_id,
                "You haven't started any poll.\
                Please, use the `/minasan_start` command.",
            )
            .await?;
            return Ok(());
        }

        let message_id = message_id.unwrap();
        bot.send_message(chat_id, "Here's your poll.").await?;
        bot.forward_message(chat_id, chat_id, message_id).await?;
        Ok(())
    }

    async fn create_poll(
        bot: Bot,
        chat_id: ChatId,
        chat_storage: Arc<ChatStorage>,
    ) -> Result<MessageId, RequestError> {
        let question_str = "\
            Do you consent to be tagged by `minasan` bot, \
            via submission of your @username?\
            ";

        let poll_options = [String::from("I do."), String::from("I don't.")];

        let mut poll_payload = SendPoll::new(chat_id, question_str, poll_options);
        poll_payload.is_anonymous = Some(false);

        let message: Message = JsonRequest::new(bot, poll_payload).send().await?;

        let poll_id = match message.kind {
            MessageKind::Common(msg) => match msg.media_kind {
                MediaKind::Poll(mpoll) => mpoll.poll.id,
                _ => panic!("Not Poll"),
            },
            _ => panic!("Not common"),
        };

        chat_storage.update_message(chat_id, message.id).await;
        chat_storage.update_poll(chat_id, poll_id).await;
        Ok(message.id)
    }
}
