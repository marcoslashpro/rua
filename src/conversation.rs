use api_ollama::{ChatMessage, MessageRole};
use uuid::Uuid;
use crate::db::{get_connection};

pub struct RuaConversation;

impl RuaConversation {
    pub async fn get_all_message(window: RuaConversationMessageWindow) -> Result<Vec<RuaConversationMessage>, sqlx::Error> {
        sqlx::query_as::<_, RuaConversationMessage>("SELECT * FROM messages ORDER BY id LIMIT $1")
            .bind(window)
            .fetch_all(&get_connection().await.pool)
            .await
    }

    pub async fn add_message(message: RuaConversationMessage) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO messages (id, kind, content) VALUES ($1, $2, $3)")
            .bind(&message.id)
            .bind(&message.kind)
            .bind(&message.content)
            .execute(&get_connection().await.pool)
            .await?;

        Ok(( ))
    }

    pub fn convert_all_message_to_ollama_compatible(messages: Vec<RuaConversationMessage>) -> Vec<ChatMessage> {
        messages.into_iter()
            .map(|message|
                ChatMessage {
                    role: Self::determine_ollama_role_by_message_kind(message.kind),
                    content: message.content,
                    images: None,
                    tool_calls: None,
                }
            )
            .collect()
    }

    fn determine_ollama_role_by_message_kind(kind: RuaConversationMessageKind) -> MessageRole {
        match kind {
            RuaConversationMessageKind::System => MessageRole::System,
            RuaConversationMessageKind::User => MessageRole::User,
            RuaConversationMessageKind::Assistant => MessageRole::Assistant,
            RuaConversationMessageKind::Tool => MessageRole::Tool,
        }
    }

    pub fn convert_ollama_message_to_rua_message_compatible(message: ChatMessage) -> RuaConversationMessage {
        RuaConversationMessage::new(
            Self::determine_message_kind_by_ollama_message_role(message.role),
            message.content,
        )
    }

    fn determine_message_kind_by_ollama_message_role(role: MessageRole) -> RuaConversationMessageKind {
        match role {
            MessageRole::User => RuaConversationMessageKind::User,
            MessageRole::Assistant => RuaConversationMessageKind::Assistant,
            MessageRole::System => RuaConversationMessageKind::System,
            MessageRole::Tool => RuaConversationMessageKind::Tool,
        }
    }
}

pub type RuaConversationMessageWindow = u8;
#[derive(sqlx::FromRow, Debug)]
pub struct RuaConversationMessage {
    id: RuaConversationMessageId,

    kind: RuaConversationMessageKind,
    content: RuaConversationMessageContent
}

impl RuaConversationMessage {
    pub fn new(kind: RuaConversationMessageKind, content: RuaConversationMessageContent) -> Self {
        Self {
            id: Uuid::now_v7(),
            kind,
            content,
        }
    }
}

type RuaConversationMessageId = Uuid;
#[derive(Debug, sqlx::Type)]
pub enum RuaConversationMessageKind {
    System,
    User,
    Assistant,
    Tool,
}
pub type RuaConversationMessageContent = String;