use std::collections::HashMap;
use teloxide::prelude::*;
use teloxide::RequestError;
use tokio::sync::Mutex;
use teloxide::types::MessageId;

type MessageStorage = HashMap<ChatId, MessageId>;

#[derive(Debug)]
pub struct MinasanBot {
    bot: Bot,
    storage: Mutex<MessageStorage>,
}

impl MinasanBot {
    pub fn new(bot: Bot) -> Self {
        Self {bot, storage: Mutex::new(HashMap::new())}
    }
    
    pub fn from_token(token: &str) -> Self {
        Self::new(Bot::new(token))
    }
    
    pub async fn start(&self, chat_id: ChatId) -> Result<(), RequestError> {
        if self.storage.lock().await.get(&chat_id).is_some() {
            self.bot.send_message(chat_id, "\
            You have already started the poll, if you want to restart, \
            use the `/restart` command.\
            ").await?;
            self.get_poll(chat_id).await?;
            return Ok(())
        }
        
        self.create_poll(chat_id).await?;
        Ok(())
    }
    
    pub async fn restart(&self, chat_id: ChatId) -> Result<(), RequestError> {
        let message_id = self.storage.lock().await.get(&chat_id).copied();
        
        if message_id.is_none() {
            self.bot.send_message(chat_id, "You haven't started working with me!\
            Use the `/start` command.").await?;
            return Ok(())
        }
        
        let message_id = message_id.unwrap();
        self.bot.delete_message(chat_id, message_id).await?;
        
        self.create_poll(chat_id).await?;
        Ok(())
    }
    
    pub async fn tag_everyone(&self, chat_id: ChatId) -> Result<(), RequestError> {
        let message_id = self.storage.lock().await.get(&chat_id).copied();

        if message_id.is_none() {
            self.bot.send_message(chat_id, "You haven't started working with me!\
            Use the `/start` command.").await?;
            return Ok(())
        }
        
        let message_id = message_id.unwrap();
        todo!()
    }
    
    async fn create_poll(&self, chat_id: ChatId) -> Result<(), RequestError> {
        let question_str = "\
        Do you consent to be tagged by `minasan` bot,\
        via submission of your @username?\
        ";

        let poll_options = [String::from("I do."), String::from("I don't.")];
        let request = self.bot.send_poll(chat_id, question_str, poll_options).await?;

        let mut storage = self.storage.lock().await;
        storage.insert(chat_id, request.id);
        Ok(())
    }
    
    pub async fn get_poll(&self, chat_id: ChatId) -> Result<(), RequestError> {
        let message_id = self.storage.lock().await.get(&chat_id).copied();
        
        if message_id.is_none() {
            self.bot.send_message(chat_id, "You haven't started any poll.\
            Please, use the `/start` command.").await?;
            return Ok(())
        }
        
        let message_id = message_id.unwrap();
        self.bot.send_message(chat_id, "Here's your poll.").await?;
        self.bot.forward_message(chat_id, chat_id, message_id).await?;
        Ok(())
    }
    
    pub async fn kill(self) -> Result<(), RequestError> { todo!() }
}
