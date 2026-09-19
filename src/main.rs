use crate::configuration::RuaConfiguration;

mod llm;
mod cli;
mod configuration;

#[tokio::main]
async fn main() -> Result<(), Box< dyn std::error::Error >> {
    let configuration = RuaConfiguration::new()?;
    let client = llm::RuaOllamaClient::new(configuration.model, configuration.base_url, None).await;
    let mut runner = cli::RuaCliRunner::new(client);
    runner.run(None).await;
    Ok(( ))
}