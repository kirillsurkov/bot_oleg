use super::OlegCommand;
use async_trait::async_trait;
use chrono::Utc;
use openai::chat::*;
use teloxide::prelude::*;

pub struct GetTime;

pub struct Args {
    pub offset_h: i32,
    pub offset_m: i32,
}

#[async_trait]
impl OlegCommand<Args> for GetTime {
    fn desc() -> ChatCompletionFunctionDefinition {
        ChatCompletionFunctionDefinition {
            name: "get_time".to_owned(),
            description: Some("Get current time".to_owned()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "offset_hours": {
                        "type": "integer",
                        "description": "UTC time offset in hours"
                    },
                    "offset_minutes": {
                        "type": "integer",
                        "description": "UTC time offset in minutes"
                    }
                },
                "required": ["offset_hours", "offset_minutes"],
            })),
        }
    }

    async fn execute(args: Args) -> (Option<Message>, Option<String>) {
        (
            None,
            match chrono::FixedOffset::east_opt(args.offset_h * 3600 + args.offset_m * 60) {
                Some(offset) => Some(
                    Utc::now()
                        .with_timezone(&offset)
                        .format("{\"date\":\"%Y-%m-%d\",\"time\":\"%H:%M:%S\"}")
                        .to_string(),
                ),
                None => Some("Invalid timezone".to_owned()),
            },
        )
    }
}
