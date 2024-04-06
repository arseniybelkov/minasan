use std::collections::{HashMap, HashSet};

use teloxide::prelude::*;
use teloxide::types::MessageId;
use tokio::sync::Mutex;

type MessageStorage = HashMap<ChatId, MessageId>;
type UserStorage = HashMap<ChatId, HashSet<String>>;
type PollStorage = HashMap<String, ChatId>;

pub struct ChatStorage {
    users: Mutex<UserStorage>,
    polls: Mutex<PollStorage>,
    messages: Mutex<MessageStorage>,
}

impl ChatStorage {
    pub fn new() -> Self {
        Self {
            users: Mutex::new(UserStorage::new()),
            polls: Mutex::new(PollStorage::new()),
            messages: Mutex::new(MessageStorage::new()),
        }
    }

    pub async fn add_chat(&self, chat_id: ChatId, message_id: MessageId) {
        self.users.lock().await.insert(chat_id, HashSet::new());
        self.messages.lock().await.insert(chat_id, message_id);
    }

    pub async fn add_user(&self, chat_id: ChatId, new_user: String) {
        let mut users = self.users.lock().await;
        users.get_mut(&chat_id).unwrap().insert(new_user);
    }

    pub async fn remove_user(&self, chat_id: ChatId, user: String) {
        self.users.lock().await.get_mut(&chat_id).unwrap().remove(&user);
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
    
    pub async fn update_poll(&self, chat_id: ChatId, poll_id: String) {
        self.polls.lock().await.insert(poll_id, chat_id);
    }

    pub async fn poll2chat(&self, poll_id: &String) -> Option<ChatId> {
        self.polls.lock().await.get(poll_id).cloned()
    }

    pub async fn remove_chat(&self, chat_id: ChatId) {
        self.users.lock().await.remove(&chat_id).unwrap();
        self.messages.lock().await.remove(&chat_id).unwrap();
    }
}
