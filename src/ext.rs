use teloxide::types::Message;

pub trait MessageExt {
    fn text_or_caption(&self) -> Option<&str>;
}

impl MessageExt for Message {
    fn text_or_caption(&self) -> Option<&str> {
        self.text().or(self.caption())
    }
}