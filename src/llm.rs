use std::time::Duration;
use api_ollama::{ChatMessage, ChatRequest, ChatResponse, OllamaClient, OllamaResult};
use crate::configuration::{RuaConfigurationBaseUrl, RuaConfigurationModel};

pub struct RuaOllamaClient {
    model: RuaConfigurationModel,
    _client: OllamaClient,
}

pub type RuaChatMessageContent = String;

impl RuaOllamaClient {
    pub(crate) async fn new(model: RuaConfigurationModel, base_url: RuaConfigurationBaseUrl, timeout: Option<Duration>) -> Self {
        RuaOllamaClient {
            model,
            _client: RuaOllamaClient::init_client(base_url, timeout).await,
        }
    }

    async fn init_client(base_url: RuaConfigurationBaseUrl, timeout: Option<Duration>) -> OllamaClient {
        let timeout = timeout.unwrap_or_else(|| OllamaClient::recommended_timeout_default());

        let mut client = OllamaClient::new(base_url.to_string(), timeout);

        if !client.is_available().await {
            panic!("Can't connect to the Ollama server at url: {}", &base_url);
        } else {
            client
        }
    }

    pub(crate) async fn chat(&mut self, messages: Vec<ChatMessage>) -> OllamaResult<ChatResponse> {
        self._client.chat(
            ChatRequest {
                model: self.model.to_string(),
                messages,
                stream: Some(false),
                options: None,
                tools: None,
                tool_messages: None,
            }
        ).await
    }
}