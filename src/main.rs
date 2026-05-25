use std::io;
use std::io::Write;
use std::time::Duration;
use api_ollama::{ OllamaClient, ChatRequest, ChatResponse, ChatMessage, MessageRole, OllamaResult };

const RUA_OLLAMA_BASE_URL: &str = "http://localhost:11434";
const RUA_OLLAMA_BASE_MODEL: &str = "ministral-3:8b";

type RuaOllamaModel = &'static str;

struct RuaOllamaClient {
    model: RuaOllamaModel,
    _client: OllamaClient,
}

type RuaChatMessageContent = String;

type RuaRunnerResult<T> = Result<T, RuaRunnerError>;

#[derive(Debug)]
enum RuaRunnerError {
    Eof(String),
    Flush(String),
    Exit,
    Empty,
    Unknown(String)
}

enum RuaRunnerExitCommand {
    Exit,
    NotFound
}

impl From<&str> for RuaRunnerExitCommand {
    fn from(value: &str) -> Self {
        match value {
            "Quit" => RuaRunnerExitCommand::Exit,
            "quit" => RuaRunnerExitCommand::Exit,
            "Exit" => RuaRunnerExitCommand::Exit,
            "exit" => RuaRunnerExitCommand::Exit,
            &_ => RuaRunnerExitCommand::NotFound
        }
    }
}

impl RuaOllamaClient {
    async fn new(model: RuaOllamaModel, base_url: Option<&str>, timeout: Option<Duration>) -> Self {
        RuaOllamaClient {
            model,
            _client: RuaOllamaClient::init_client(base_url, timeout).await,
        }
    }

    async fn init_client(base_url: Option<&str>, timeout: Option<Duration>) -> OllamaClient {
        let url = base_url.unwrap_or_else(|| RUA_OLLAMA_BASE_URL);
        let timeout = timeout.unwrap_or_else(|| OllamaClient::recommended_timeout_default());

        let mut client = OllamaClient::new(url.to_string(), timeout);

        if !client.is_available().await {
            panic!("Can't connect to the Ollama server at url: {}", &url);
        } else {
            client
        }
    }

    async fn chat(&mut self, messages: Vec<ChatMessage>) -> OllamaResult<ChatResponse> {
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

struct RuaCliRunner {
    client: RuaOllamaClient,
}

impl RuaCliRunner {
    fn new(client: RuaOllamaClient) -> Self { Self { client } }

    async fn run(&mut self, convo: Option<Vec<ChatMessage>>) -> Vec<ChatMessage> {
        let mut convo = convo.unwrap_or_else(|| Vec::new());

        loop {
            let input = match self.get_user_input() {
                Ok(input) => input, // input validated.
                Err(e) => {
                    match e {
                        RuaRunnerError::Eof(message) => {
                            println!("{}", message);
                            break;
                        }
                        RuaRunnerError::Flush(message) => {
                            println!("Error during stdin flushing {}", message);
                            break;
                        }
                        RuaRunnerError::Exit => {
                            println!("Exiting...");
                            break;
                        }
                        RuaRunnerError::Empty => { continue }
                        RuaRunnerError::Unknown(message) => {
                            println!("Unknown error: {}", message);
                            break;
                        }
                    }
                }
            };

            convo.push(
                ChatMessage {
                    role: MessageRole::User,
                    content: input.to_string(),
                    images: None,
                    tool_calls: None,
                }
            );

            let response = match self.client.chat(convo.clone()).await {
                Ok(response) => response,
                Err(e) => {
                    println!("Error while generating model's response: {:?}", e.to_string());
                    break;
                }
            };

            convo.push(response.clone().message);

            match self.print_model_response(response) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error while printing model response: {:?}", e);
                    break;
                }
            }
        }

        convo
    }

    fn get_user_input(&self) -> RuaRunnerResult<RuaChatMessageContent> {
        print!("You: ");
        match io::stdout().flush() {
            Ok(_) => (),
            Err(e) => return Err(RuaRunnerError::Flush(e.to_string()))
        }

        let mut input = String::new();
        let bytes_read = match io::stdin().read_line(&mut input) {
            Ok(n) => n,
            Err(e) => return Err(RuaRunnerError::Unknown(e.to_string())),
        };

        // Handle EOF (no input available in non-interactive mode)
        if bytes_read == 0 {
            return Err(
                RuaRunnerError::Eof(
                    format!(
                        "\n👋 No input available (EOF). Exiting gracefully.\n{}",
                        "Note : Use this example in interactive terminal only."
                    )
                )
            );
        }

        let message = input.trim();

        match RuaRunnerExitCommand::from(message) {
            RuaRunnerExitCommand::Exit => { return Err( RuaRunnerError::Exit ) },
            RuaRunnerExitCommand::NotFound => {} // No exit command found, continue conversation.
        }

        if message.is_empty() {
            return Err(RuaRunnerError::Empty);
        }

        Ok( message.to_string() )
    }

    fn print_model_response(&self, response: ChatResponse) -> RuaRunnerResult<()> {
        print!("AI: {}\n", response.message.content);
        match io::stdout().flush() {
            Ok(_) => Ok(()),
            Err(e) => Err(RuaRunnerError::Flush(e.to_string()))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box< dyn std::error::Error >> {
    let client = RuaOllamaClient::new(RUA_OLLAMA_BASE_MODEL, None, None).await;
    let mut runner = RuaCliRunner::new(client);
    runner.run(None).await;
    Ok(( ))
}