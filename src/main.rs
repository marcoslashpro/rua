use crate::configuration::CONFIGURATION;

mod llm;
mod cli;
mod configuration;
mod conversation;
mod db;

#[tokio::main]
async fn main() -> Result<(), Box< dyn std::error::Error >> {
    let client = llm::RuaOllamaClient::new(
        &CONFIGURATION.model,
        &CONFIGURATION.base_url,
        None,
    ).await;
    let mut runner = cli::RuaCliRunner::new(client);
    runner.run().await?;
    Ok(( ))
}