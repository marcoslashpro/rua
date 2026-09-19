use std::time::Duration;
use api_ollama::{ChatRequest, ChatResponse, OllamaClient, OllamaResult};
use crate::configuration::{RuaConfigurationBaseUrl, RuaConfigurationModel};
use crate::conversation::{RuaConversation, RuaConversationMessage};

pub struct RuaOllamaClient {
    model: &'static RuaConfigurationModel,
    _client: OllamaClient,
}

impl RuaOllamaClient {
    pub(crate) async fn new(model: &'static RuaConfigurationModel, base_url: &RuaConfigurationBaseUrl, timeout: Option<Duration>) -> Self {
        RuaOllamaClient {
            model,
            _client: RuaOllamaClient::init_client(base_url, timeout).await,
        }
    }

    async fn init_client(base_url: &RuaConfigurationBaseUrl, timeout: Option<Duration>) -> OllamaClient {
        let timeout = timeout.unwrap_or_else(|| OllamaClient::recommended_timeout_default());

        let mut client = OllamaClient::new(base_url.to_string(), timeout);

        if !client.is_available().await {
            panic!("Can't connect to the Ollama server at url: {}", &base_url);
        } else {
            client
        }
    }

    pub(crate) async fn chat(&mut self, messages: Vec<RuaConversationMessage>) -> OllamaResult<ChatResponse> {
        self._client.chat(
            ChatRequest {
                model: self.model.to_string(),
                messages: RuaConversation::convert_all_message_to_ollama_compatible(messages),
                stream: Some(false),
                options: None,
                tools: None,
                tool_messages: None,
            }
        ).await
    }
}