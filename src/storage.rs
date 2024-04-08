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
    pub async fn dump(&self, path: &Path) -> std::io::Result<usize> {
        let user_storage = self.users.lock().await;
        // might have potential race condition here
        let message_storage = self.messages.lock().await;
        let poll2chat_ids = self.polls.lock().await;

        let mut counter = 0;

        for (chat_id, users) in user_storage.iter() {
            let message_id = message_storage.get(chat_id).unwrap();
            let poll_id = poll2chat_ids
                .iter()
                .find(|(_, v)| *v == chat_id)
                .map_or("null", |(p, _)| p)
                .to_string();

            let json = serde_json::json!({
                "message_id": message_id.0,
                "poll_id": poll_id,
                "users": users,
            });

            let chat_id_str = chat_id.to_string();
            let file_name = format!("{chat_id_str}.json");

            let file = File::create(path.join(file_name))?;
            let mut writer = BufWriter::new(file);
            serde_json::to_writer(&mut writer, &json)?;
            writer.flush()?;
            counter += 1;
        }

        Ok(counter)
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
                    .map(|v| v.as_str().unwrap().to_string())
                    .collect::<HashSet<String>>();
                let message_id =
                    MessageId(json.get("message_id").unwrap().as_i64().unwrap() as i32);
                let poll_id = json.get("poll_id").unwrap().as_str().unwrap().to_string();
                let chat_id = ChatId(
                    i64::from_str(p.path().as_path().file_stem().unwrap().to_str().unwrap())
                        .unwrap(),
                );

                user_storage.insert(chat_id, users);
                message_storage.insert(chat_id, message_id);
                if poll_id != "null" {
                    poll2chat_id.insert(poll_id, chat_id);
                }
            }
        }

        Self {
            users: Mutex::new(user_storage),
            messages: Mutex::new(message_storage),
            polls: Mutex::new(poll2chat_id),
        }
    }
}

// Here on only are the tests for `ChatStorage`.

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[tokio::test]
    async fn test_dump() {
        let chat_storage = ChatStorage::new();

        let chat_id = ChatId(0);
        chat_storage.add_chat(chat_id, MessageId(1)).await;
        chat_storage
            .add_user(chat_id, "user1".to_string())
            .await
            .unwrap();
        chat_storage
            .add_user(chat_id, "user2".to_string())
            .await
            .unwrap();

        let chat_id = ChatId(123);
        chat_storage.add_chat(chat_id, MessageId(321)).await;
        chat_storage
            .add_user(chat_id, "user3".to_string())
            .await
            .unwrap();
        chat_storage
            .add_user(chat_id, "user4".to_string())
            .await
            .unwrap();
        chat_storage.update_poll(chat_id, "12345".to_string()).await;

        let tmp_dir = tempfile::tempdir().unwrap();
        let n_dumped = chat_storage.dump(tmp_dir.path()).await.unwrap();
        assert_eq!(n_dumped, 2);

        let mut file_names = tmp_dir
            .path()
            .read_dir()
            .unwrap()
            .flatten()
            .map(|p| {
                p.path()
                    .as_path()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            })
            .collect::<Vec<_>>();
        file_names.sort();
        let target = vec!["0".to_string(), "123".to_string()];

        assert_eq!(file_names, target);
    }

    #[tokio::test]
    async fn test_load() {
        let tmp_dir = tempfile::tempdir().unwrap();

        let json1 = json!({
            "users": vec!["user1", "user2"],
            "message_id": 123,
            "poll_id": "123456",
        });

        let file = File::create(tmp_dir.path().join("1.json")).unwrap();
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &json1).unwrap();
        writer.flush().unwrap();

        let json2 = json!({
            "users": vec!["user3"],
            "message_id": 890,
            "poll_id": "null",
        });

        let file = File::create(tmp_dir.path().join("2.json")).unwrap();
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &json2).unwrap();
        writer.flush().unwrap();

        let chat_storage = ChatStorage::load(tmp_dir.path());

        let (chat_id1, chat_id2) = (ChatId(1), ChatId(2));
        let (users1, users2) = (
            HashSet::from(["user1".to_string(), "user2".to_string()]),
            HashSet::from([String::from("user3")]),
        );
        let (message_id1, message_id2) = (MessageId(123), MessageId(890));
        let (poll_id1, poll_id2) = (String::from("123456"), String::from("abc"));

        assert_eq!(chat_storage.get_users(chat_id1).await.unwrap(), users1);
        assert_eq!(chat_storage.get_users(chat_id2).await.unwrap(), users2);

        assert_eq!(
            chat_storage.get_message_id(chat_id1).await.unwrap(),
            message_id1
        );
        assert_eq!(
            chat_storage.get_message_id(chat_id2).await.unwrap(),
            message_id2
        );

        assert_eq!(chat_storage.poll2chat(&poll_id1).await.unwrap(), chat_id1);
        assert!(chat_storage.poll2chat(&poll_id2).await.is_none());
    }

    #[tokio::test]
    async fn test_load_dump_consistency() {
        let tempdir = tempfile::tempdir().unwrap();

        let source = ChatStorage::new();

        let chat_id = ChatId(101);
        source.add_chat(chat_id, MessageId(1)).await;
        source
            .add_user(chat_id, "usernamesome".to_string())
            .await
            .unwrap();
        source.add_user(chat_id, "user2".to_string()).await.unwrap();

        let chat_id = ChatId(100);
        source.add_chat(chat_id, MessageId(321)).await;
        source
            .add_user(chat_id, "user3312".to_string())
            .await
            .unwrap();
        source
            .add_user(chat_id, "someuser1234".to_string())
            .await
            .unwrap();
        source.update_poll(chat_id, "12345".to_string()).await;
        source.dump(tempdir.path()).await.unwrap();

        let target = ChatStorage::load(tempdir.path());

        assert_eq!(
            source.users.lock().await.clone(),
            target.users.lock().await.clone()
        );
        assert_eq!(
            source.messages.lock().await.clone(),
            target.messages.lock().await.clone()
        );
        assert_eq!(
            source.polls.lock().await.clone(),
            target.polls.lock().await.clone()
        );
    }
}
