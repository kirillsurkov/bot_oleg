#[derive(Clone)]
pub struct FunctionReq {
    pub name: String,
    pub args: String,
}

#[derive(Clone)]
pub struct FunctionRes {
    pub name: String,
    pub res: String,
}

#[derive(Clone)]
pub struct DBMessage {
    pub chat_id: i64,
    pub msg_id: i32,
    pub cause: String,
    pub sender_id: Option<u64>,
    pub reply_id: Option<i32>,
    pub file_id: Option<String>,
    pub function_req: Option<FunctionReq>,
    pub function_res: Option<FunctionRes>,
    pub sender: Option<String>,
    pub text: Option<String>,
}

impl From<&sqlite::Statement<'_>> for DBMessage {
    fn from(statement: &sqlite::Statement) -> Self {
        DBMessage {
            chat_id: statement.read::<i64, _>("chat_id").unwrap(),
            msg_id: statement.read::<i64, _>("msg_id").unwrap() as i32,
            cause: statement.read::<String, _>("cause").unwrap(),
            sender_id: statement
                .read::<i64, _>("sender_id")
                .map(|id| id as u64)
                .ok(),
            reply_id: statement
                .read::<i64, _>("reply_id")
                .map(|id| id as i32)
                .ok(),
            file_id: statement.read::<String, _>("file_id").ok(),
            function_req: None,
            function_res: None,
            sender: statement.read::<String, _>("sender").ok(),
            text: statement.read::<String, _>("text").ok(),
        }
    }
}

pub struct DB {
    connection: sqlite::Connection,
}

impl DB {
    pub fn new() -> Self {
        let connection = sqlite::open("DB.db").unwrap();
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS messages(
                    chat_id INTEGER,
                    msg_id INTEGER,
                    cause TEXT,
                    sender_id INTEGER,
                    reply_id INTEGER,
                    file_id TEXT,
                    sender TEXT,
                    text TEXT,
                    PRIMARY KEY(chat_id, msg_id)
                )",
            )
            .unwrap();
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS captions(
                    file_id TEXT PRIMARY KEY,
                    caption TEXT
                )",
            )
            .unwrap();
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS functions(
                    id INTEGER PRIMARY KEY,
                    chat_id INTEGER,
                    msg_id INTEGER,
                    name TEXT,
                    req TEXT,
                    res TEXT
                )",
            )
            .unwrap();
        Self { connection }
    }

    pub fn add_message(&self, cause: &str, msg: &teloxide::types::Message) {
        let mut statement = self
            .connection
            .prepare("INSERT INTO messages VALUES(?, ?, ?, ?, ?, ?, ?, ?)")
            .unwrap();
        statement.bind((1, msg.chat.id.0)).unwrap();
        statement.bind((2, msg.id.0 as i64)).unwrap();
        statement.bind((3, cause)).unwrap();
        statement
            .bind((4, msg.from().map(|m| m.id.0 as i64)))
            .unwrap();
        statement
            .bind((5, msg.reply_to_message().map(|r| i64::from(r.id.0))))
            .unwrap();
        statement
            .bind((
                6,
                msg.photo()
                    .and_then(|p| p.last())
                    .map(|p| p.file.id.as_str()),
            ))
            .unwrap();
        statement
            .bind((7, msg.from().map(|u| u.full_name()).as_deref()))
            .unwrap();
        statement.bind((8, msg.text().or(msg.caption()))).unwrap();
        statement.next().unwrap();

        if let Some(photo) = msg.photo().and_then(|p| p.last()) {
            self.add_caption(&photo.file.id, None);
        }
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

    pub fn add_caption(&self, file_id: &str, caption: Option<&str>) {
        if match self.get_caption(file_id) {
            Ok(caption) => caption,
            Err(_) => None,
        }
        .is_none()
        {
            let mut statement = self
                .connection
                .prepare("INSERT INTO captions(file_id, caption) VALUES(?, ?) ON CONFLICT(file_id) DO UPDATE SET caption=?")
                .unwrap();
            statement.bind((1, file_id)).unwrap();
            statement.bind((2, caption)).unwrap();
            statement.bind((3, caption)).unwrap();
            statement.next().unwrap();
        }
    }

    pub fn get_caption(&self, file_id: &str) -> Result<Option<String>, ()> {
        let mut statement = self
            .connection
            .prepare("SELECT * FROM captions WHERE file_id=?")
            .unwrap();
        statement.bind((1, file_id)).unwrap();
        match statement.next() {
            Ok(sqlite::State::Row) => Ok(statement.read::<String, _>("caption").ok()),
            _ => Err(()),
        }
    }

    pub fn add_function(
        &self,
        msg: &teloxide::types::Message,
        name: &str,
        req: Option<&str>,
        res: Option<&str>,
    ) {
        let mut statement = self
            .connection
            .prepare("INSERT INTO functions(chat_id, msg_id, name, req, res) VALUES(?, ?, ?, ?, ?)")
            .unwrap();
        statement.bind((1, msg.chat.id.0)).unwrap();
        statement.bind((2, msg.id.0 as i64)).unwrap();
        statement.bind((3, name)).unwrap();
        statement.bind((4, req)).unwrap();
        statement.bind((5, res)).unwrap();
        statement.next().unwrap();
    }

    pub fn get_functions(&self, chat_id: i64, msg_id: i32) -> Vec<DBMessage> {
        let mut statement = self
            .connection
            .prepare("SELECT * FROM functions WHERE chat_id=? and msg_id=? ORDER BY id DESC")
            .unwrap();
        statement.bind((1, chat_id)).unwrap();
        statement.bind((2, msg_id as i64)).unwrap();
        let mut functions = vec![];
        let re = regex::Regex::new(r"^[a-zA-Z0-9_-]{1,64}$").unwrap();
        while let Ok(sqlite::State::Row) = statement.next() {
            let name = statement.read::<String, _>("name").unwrap();
            if !re.is_match(&name) {
                continue;
            }
            let req = statement.read::<String, _>("req").map(|r| FunctionReq {
                name: name.clone(),
                args: r,
            });
            let res = statement.read::<String, _>("res").map(|r| FunctionRes {
                name: name.clone(),
                res: r,
            });
            functions.push(DBMessage {
                chat_id: statement.read::<i64, _>("chat_id").unwrap(),
                msg_id: statement.read::<i64, _>("msg_id").unwrap() as i32,
                cause: req
                    .as_ref()
                    .map(|_| "oleg_a")
                    .unwrap_or("oleg_f")
                    .to_owned(),
                sender_id: None,
                reply_id: None,
                file_id: None,
                function_req: req.ok(),
                function_res: res.ok(),
                sender: None,
                text: None,
            });
        }
        functions
    }

    pub fn unwind_thread<Filter: Fn(&str) -> bool>(
        &self,
        msg: &teloxide::types::Message,
        limit: usize,
        filter: Filter,
    ) -> Vec<DBMessage> {
        let mut msg_id = msg.id.0;
        let mut reply_id = msg.reply_to_message().map(|r| r.id.0);
        let mut messages = vec![];

        let text_with_id = |file_id: Option<&str>, text: &str| {
            match file_id {
                Some(file_id) => format!("{text}\n###ID###\n{{\"file_id\":{file_id}}}",),
                None => text.to_owned(),
            }
            .trim()
            .to_owned()
        };

        if let Some(text) = msg.text().or(msg.caption()) {
            let text = text_with_id(
                msg.photo()
                    .and_then(|p| p.last())
                    .map(|p| p.file.id.as_str()),
                text,
            );
            if filter(&text) {
                messages.extend(self.get_functions(msg.chat.id.0, msg.id.0));
                messages.push(DBMessage {
                    chat_id: msg.chat.id.0,
                    msg_id: msg.id.0,
                    cause: String::new(),
                    sender_id: msg.from().map(|u| u.id.0),
                    reply_id,
                    file_id: msg
                        .photo()
                        .and_then(|p| p.last())
                        .map(|p| p.file.id.clone()),
                    function_req: None,
                    function_res: None,
                    sender: msg.from().map(|u| u.full_name()),
                    text: Some(text),
                });
            }
        }

        while messages.len() < limit && reply_id.is_some() {
            if let Some(mut reply) = self.get_message(msg.chat.id.0, reply_id.unwrap()) {
                msg_id = reply_id.unwrap();
                reply_id = reply.reply_id;
                reply.text = Some(text_with_id(
                    reply.file_id.as_deref(),
                    &reply.text.unwrap_or_default(),
                ));
                if filter(reply.text.as_ref().unwrap()) {
                    messages.extend(self.get_functions(reply.chat_id, reply.msg_id));
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
        statement.bind((2, i64::from(msg_id))).unwrap();
        while let Ok(sqlite::State::Row) = statement.next() {
            if messages.len() >= limit {
                break;
            }
            let mut message = DBMessage::from(&statement);
            message.text = Some(text_with_id(
                message.file_id.as_deref(),
                &message.text.unwrap_or_default(),
            ));
            if filter(message.text.as_ref().unwrap()) {
                messages.extend(self.get_functions(message.chat_id, message.msg_id));
                messages.push(message.clone());
                if let Some(reply_id) = message.reply_id {
                    if let Some(mut reply) = self.get_message(msg.chat.id.0, reply_id) {
                        reply.text = Some(text_with_id(
                            reply.file_id.as_deref(),
                            &reply.text.unwrap_or_default(),
                        ));
                        if filter(reply.text.as_ref().unwrap()) {
                            messages.extend(self.get_functions(reply.chat_id, reply.msg_id));
                            messages.push(reply);
                        }
                    }
                }
            }
        }

        messages.reverse();
        messages
    }
}
