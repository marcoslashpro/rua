use std::io;
use std::io::Write;
use api_ollama::{ChatMessage, ChatResponse, MessageRole};
use crate::llm::{RuaOllamaClient, RuaChatMessageContent};

pub struct RuaCliRunner {
    client: RuaOllamaClient,
}

impl RuaCliRunner {
    pub(crate) fn new(client: RuaOllamaClient) -> Self { Self { client } }

    pub(crate) async fn run(&mut self, convo: Option<Vec<ChatMessage>>) -> Vec<ChatMessage> {
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