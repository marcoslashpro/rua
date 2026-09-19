use std::io;
use std::io::Write;
use api_ollama::{ChatResponse};
use crate::conversation::{RuaConversation, RuaConversationMessage, RuaConversationMessageContent, RuaConversationMessageKind};
use crate::llm::{RuaOllamaClient};

pub struct RuaCliRunner {
    client: RuaOllamaClient,
}

impl RuaCliRunner {
    pub(crate) fn new(client: RuaOllamaClient) -> Self { Self { client } }

    pub(crate) async fn run(&mut self) -> Result<(), sqlx::error::Error> {
        loop {
            let input = match self.get_user_input() {
                Ok(input) => input,
                Err(e) => {
                    match e {
                        RuaRunnerError::Eof(message) => {
                            println!("{}", message);
                            
                            return Ok (());
                        }
                        RuaRunnerError::Flush(message) => {
                            println!("Error during stdin flushing {}", message);
                            
                            return Ok (());
                        }
                        RuaRunnerError::Exit => {
                            println!("Exiting...");
                            
                            return Ok (());
                        }
                        RuaRunnerError::Empty => { continue }
                        RuaRunnerError::Unknown(message) => {
                            println!("Unknown error: {}", message);
                            
                            return Ok (());
                        }
                    }
                }
            };

            RuaConversation::add_message(
                RuaConversationMessage::new(
                    RuaConversationMessageKind::User,
                    input.to_string()
                )
            ).await?;

            let response = match self.client.chat(RuaConversation::get_all_message(50).await?).await {
                Ok(response) => response,
                Err(e) => {
                    println!("Error while generating model's response: {:?}", e.to_string());
                    
                    return Ok (());
                }
            };

            RuaConversation::add_message(
                RuaConversation::convert_ollama_message_to_rua_message_compatible(
                    response.clone().message)
            ).await?;

            match self.print_model_response(response) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error while printing model response: {:?}", e);
                    
                    return Ok (());
                }
            }
        }
    }

    fn get_user_input(&self) -> RuaRunnerResult<RuaConversationMessageContent> {
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
            "q" => RuaRunnerExitCommand::Exit,
            &_ => RuaRunnerExitCommand::NotFound
        }
    }
}