use std::collections::{HashMap, HashSet};

use teloxide::prelude::*;
use teloxide::types::MessageId;
use tokio::sync::Mutex;

type MessageStorage = HashMap<ChatId, MessageId>;
type UserStorage = HashMap<ChatId, HashSet<String>>;

pub struct ChatStorage {
    users: Mutex<UserStorage>,
    messages: Mutex<MessageStorage>,
}

impl ChatStorage {
    pub fn new() -> Self {
        Self {
            users: Mutex::new(UserStorage::new()),
            messages: Mutex::new(MessageStorage::new()),
        }
    }
    
    pub async fn add_chat(&self, chat_id: ChatId, message_id: MessageId) {
        self.update_users(chat_id, Vec::new()).await;
        self.update_message(chat_id, message_id).await;
    }
    
    pub async fn update_users(&self, chat_id: ChatId, new_users: Vec<String>) {
        let mut users = self.users.lock().await;
        users.get_mut(&chat_id).unwrap().extend(new_users);
    }
    
    pub async fn update_message(&self, chat_id: ChatId, message_id: MessageId) {
        self.messages.lock().await.insert(chat_id, message_id);
    }
    
    pub async fn get_users(&self, chat_id: ChatId) -> Option<HashSet<String>> {
        self.users.lock().await.get(&chat_id).cloned()
    }
    
    pub async fn get_message_id(&self, chat_id: ChatId) -> Option<MessageId> {
        self.messages.lock().await.get(&chat_id).cloned()
    }
    
    pub async fn clean_users(&self, chat_id: ChatId) {
        self.users.lock().await.get_mut(&chat_id).unwrap().clear()
    }
    
    pub async fn remove_chat(&self, chat_id: ChatId) {
        self.users.lock().await.remove(&chat_id).unwrap();
        self.messages.lock().await.remove(&chat_id).unwrap();
    }
}


