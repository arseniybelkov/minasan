// Poll2ChatId is really stupid, need to change it in the nearest future

use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use teloxide::RequestError;

use crate::storage::ChatStorage;

#[derive(BotCommands, Debug, PartialEq, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    /// Displays commands description.
    #[command(description = "Displays commands description.")]
    MinasanHelp,
    /// Tag everyone.
    #[command(description = "Tags every chat member consented to be tagged.")]
    Minasan,
    /// Stops the bot and removes it from the chat.
    #[command(description = "Deletes the last active poll and removes the bot from the group.")]
    MinasanKill,
    /// Resend currently active poll to this chat.
    #[command(description = "Shows the last active poll.")]
    MinasanPoll,
    /// Bot creates poll and starts tracking users.
    #[command(description = "Activates the bot and starts poll.")]
    MinasanStart,
    /// Restarts the bot, recreating the poll.
    #[command(description = "Recreates the poll.")]
    MinasanRestart,
}

pub mod endpoints {
    use teloxide::payloads::SendPoll;
    use teloxide::requests::JsonRequest;
    use teloxide::types::{MediaKind, Message, MessageId, MessageKind};

    use super::*;

    const POLL_OPTIONS: [&str; 2] = ["I do.", "I don't."];

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
                use the `/minasanrestart` command.\
            ",
            )
            .await?;
        } else {
            let message_id = create_poll(bot, chat_id, Arc::clone(&chat_storage)).await?;
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

        if let Some(message_id) = chat_storage.get_message_id(chat_id).await {
            bot.delete_message(chat_id, message_id).await?;
            chat_storage.clean_users(chat_id).await;
            create_poll(bot, chat_id, chat_storage).await?;
        } else {
            bot.send_message(
                chat_id,
                "You haven't started working with me. \
            Please use `/minasanstart` command.",
            )
            .await?;
        }
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
        if let Some(users) = chat_storage.get_users(chat_id).await {
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
        } else {
            bot.send_message(
                chat_id,
                "You haven't started the poll, \
            please use `/minasanstart` command",
            )
            .await?;
        }
        Ok(())
    }

    pub async fn update_users(
        _bot: Bot,
        chat_storage: Arc<ChatStorage>,
        poll_answer: PollAnswer,
    ) -> Result<(), RequestError> {
        let Some(chat_id) = chat_storage.poll2chat(&poll_answer.poll_id).await else {
            log::warn!(
                "Trying to update users for unstored chat, \
            bot was likely restarted there."
            );
            return Ok(());
        };

        if let Some(v) = poll_answer.option_ids.first() {
            match v {
                0 => {
                    if chat_storage.get_message_id(chat_id).await.is_some() {
                        chat_storage
                            .add_user(chat_id, poll_answer.user.username.unwrap())
                            .await;
                    }
                },
                1 => {},
                x => log::error!("Invalid poll option {x} in chat # {chat_id}, check what the fuck has happened!"),
            }
        } else {
            chat_storage
                .remove_user(chat_id, poll_answer.user.username.unwrap())
                .await;
        };
        Ok(())
    }

    pub async fn help(
        bot: Bot,
        message: Message,
        _chat_storage: Arc<ChatStorage>,
    ) -> Result<(), RequestError> {
        bot.send_message(message.chat.id, Command::descriptions().to_string())
            .await?;
        Ok(())
    }

    pub async fn get_poll(
        bot: Bot,
        message: Message,
        chat_storage: Arc<ChatStorage>,
    ) -> Result<(), RequestError> {
        let chat_id = message.chat.id;

        if let Some(message_id) = chat_storage.get_message_id(chat_id).await {
            bot.send_message(chat_id, "Here's your poll.").await?;
            bot.forward_message(chat_id, chat_id, message_id).await?;
        } else {
            bot.send_message(
                chat_id,
                "You haven't started any poll.\
                Please, use the `/minasanstart` command.",
            )
            .await?;
        }
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

        let poll_options = POLL_OPTIONS.into_iter().map(String::from);

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
