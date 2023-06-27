use sqlite;

#[derive(Clone)]
pub struct DBMessage {
    pub cause: String,
    pub sender: String,
    pub text: String,
    pub reply_id: Option<i32>,
}

impl From<&sqlite::Statement<'_>> for DBMessage {
    fn from(statement: &sqlite::Statement) -> Self {
        DBMessage {
            cause: statement.read::<String, _>("cause").unwrap(),
            sender: statement
                .read::<String, _>("sender")
                .unwrap_or("".to_owned()),
            text: statement
                .read::<String, _>("message")
                .unwrap_or("".to_owned()),
            reply_id: statement
                .read::<i64, _>("reply_id")
                .and_then(|id| Ok(id as i32))
                .ok(),
        }
    }
}

pub struct DB {
    connection: sqlite::Connection,
}

impl DB {
    pub fn new() -> Self {
        let connection = sqlite::open("DB.db").unwrap();
        connection.execute("CREATE TABLE IF NOT EXISTS messages(id INTEGER PRIMARY KEY, cause TEXT, chat_id INTEGER, msg_id INTEGER, reply_id INTEGER, sender TEXT, message TEXT)").unwrap();
        connection.execute("CREATE TABLE IF NOT EXISTS captions(id INTEGER PRIMARY KEY, chat_id INTEGER, msg_id INTEGER, caption TEXT)").unwrap();
        Self { connection }
    }

    pub fn add_message(&self, cause: &str, msg: &teloxide::types::Message) {
        let mut statement = self
            .connection
            .prepare("INSERT INTO messages(cause, chat_id, msg_id, reply_id, sender, message) VALUES(?, ?, ?, ?, ?, ?)")
            .unwrap();
        statement.bind((1, cause)).unwrap();
        statement.bind((2, msg.chat.id.0)).unwrap();
        statement.bind((3, msg.id.0 as i64)).unwrap();
        statement
            .bind((4, msg.reply_to_message().and_then(|r| Some(r.id.0 as i64))))
            .unwrap();
        {
            let name = msg.from().and_then(|u| Some(u.full_name()));
            statement.bind((5, name.as_deref())).unwrap();
        }
        {
            let text = msg.text();
            statement.bind((6, text.as_deref())).unwrap();
        }
        statement.next().unwrap();
    }

    pub fn get_message(&self, chat_id: i64, msg_id: i32) -> Option<DBMessage> {
        let mut statement = self
            .connection
            .prepare("SELECT * from messages WHERE chat_id=? AND msg_id=?")
            .unwrap();
        statement.bind((1, chat_id)).unwrap();
        statement.bind((2, msg_id as i64)).unwrap();
        match statement.next() {
            Ok(sqlite::State::Row) => Some(DBMessage::from(&statement)),
            _ => None,
        }
    }

    pub fn unwind_thread<Filter: Fn(&str) -> bool>(
        &self,
        msg: &teloxide::types::Message,
        limit: usize,
        filter: Filter,
    ) -> Vec<DBMessage> {
        let mut msg_id = msg.id.0;
        let mut reply_id = msg.reply_to_message().and_then(|r| Some(r.id.0));
        let mut messages = vec![];

        if let Some(text) = msg.text() {
            if filter(text) {
                messages.push(DBMessage {
                    cause: "".to_owned(),
                    sender: msg
                        .from()
                        .and_then(|u| Some(u.full_name()))
                        .unwrap_or("".to_owned()),
                    text: text.to_owned(),
                    reply_id,
                });
            }
        }

        while messages.len() < limit && reply_id.is_some() {
            if let Some(reply) = self.get_message(msg.chat.id.0, reply_id.unwrap()) {
                msg_id = reply_id.unwrap();
                reply_id = reply.reply_id;
                if filter(&reply.text) {
                    messages.push(reply);
                }
            } else {
                break;
            }
        }

        let mut statement = self
            .connection
            .prepare("SELECT * from messages WHERE chat_id=? AND msg_id<? ORDER BY msg_id DESC")
            .unwrap();
        statement.bind((1, msg.chat.id.0)).unwrap();
        statement.bind((2, msg_id as i64)).unwrap();
        while let Ok(sqlite::State::Row) = statement.next() {
            if messages.len() >= limit {
                break;
            }
            let message = DBMessage::from(&statement);
            if filter(&message.text) {
                messages.push(message.clone());
                if let Some(reply_id) = message.reply_id {
                    if let Some(reply) = self.get_message(msg.chat.id.0, reply_id) {
                        messages.push(reply);
                    }
                }
            }
        }

        messages.reverse();
        messages
    }
}
