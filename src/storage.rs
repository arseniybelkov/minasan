// ChatStorage needs to be able to be dumped to disk and loaded from it.

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::str::FromStr;

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

    pub async fn add_user(&self, chat_id: ChatId, new_user: String) -> Option<()> {
        let mut users = self.users.lock().await;
        users.get_mut(&chat_id)?.insert(new_user);
        Some(())
    }

    pub async fn remove_user(&self, chat_id: ChatId, user: String) -> Option<()> {
        self.users.lock().await.get_mut(&chat_id)?.remove(&user);
        Some(())
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

    pub async fn clean_users(&self, chat_id: ChatId) -> Option<()> {
        self.users.lock().await.get_mut(&chat_id)?.clear();
        Some(())
    }

    pub async fn update_poll(&self, chat_id: ChatId, poll_id: String) {
        self.polls.lock().await.insert(poll_id, chat_id);
    }

    pub async fn poll2chat(&self, poll_id: &String) -> Option<ChatId> {
        self.polls.lock().await.get(poll_id).cloned()
    }

    pub async fn remove_chat(&self, chat_id: ChatId) -> Option<()> {
        self.users.lock().await.remove(&chat_id)?;
        self.messages.lock().await.remove(&chat_id)?;
        Some(())
    }
}

impl ChatStorage {
    pub async fn dump(&self, path: &str) -> std::io::Result<()> {
        let user_storage = self.users.lock().await;
        // might have potential race condition here
        let message_storage = self.messages.lock().await;
        let poll2chat_ids = self.polls.lock().await;

        for (chat_id, users) in user_storage.iter() {
            let message_id = message_storage.get(chat_id).unwrap();
            let poll_id = poll2chat_ids
                .iter()
                .find(|(_, v)| *v == chat_id)
                .map_or("null", |(p, _)| p)
                .to_string();

            let json = serde_json::json!({
                "message_id": message_id.to_string(),
                "poll_id": poll_id,
                "users": users,
            });

            let chat_id_str = chat_id.to_string();
            let file_name = format!("{chat_id_str}.json");

            let file = File::create(Path::new(path).join(file_name))?;
            let mut writer = BufWriter::new(file);
            serde_json::to_writer(&mut writer, &json)?;
            writer.flush()?;
        }

        Ok(())
    }

    pub fn load(path: &Path) -> Self {
        let mut user_storage = UserStorage::new();
        let mut message_storage = MessageStorage::new();
        let mut poll2chat_id = PollStorage::new();

        for p in path.read_dir().unwrap().flatten() {
            if p.path().is_file() {
                let content = std::fs::read_to_string(p.path()).unwrap();
                let json =
                    serde_json::from_str::<HashMap<String, Value>>(content.as_str()).unwrap();

                let users = json
                    .get("users")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<HashSet<String>>();
                let message_id =
                    MessageId(json.get("message_id").unwrap().as_i64().unwrap() as i32);
                let poll_id = json.get("poll_id").unwrap().to_string();
                let chat_id =
                    ChatId(i64::from_str(p.path().to_str().unwrap().to_string().as_str()).unwrap());

                user_storage.insert(chat_id, users);
                message_storage.insert(chat_id, message_id);
                poll2chat_id.insert(poll_id, chat_id);
            }
        }

        Self {
            users: Mutex::new(user_storage),
            messages: Mutex::new(message_storage),
            polls: Mutex::new(poll2chat_id),
        }
    }
}
