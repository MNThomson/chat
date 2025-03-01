use std::io::{self, stdout, IsTerminal, Read, Write};

use anyhow::Ok;
use chat::{LLMOptions, LLM};
use clap::Parser;
use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::Deserialize;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(short, long, default_value = "gpt-4o")]
    model: String,
    #[arg(long, default_value = "false")]
    code: bool,
    prompt: String,
}

#[derive(Debug, PartialEq, Deserialize)]
struct AppConfig {
    api_key: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let dir = dirs::config_dir()
        .expect("The config dir to exist")
        .join("chat.toml");

    let figment: AppConfig = Figment::new().merge(Toml::file(dir)).extract().unwrap();

    let stdin_str = if !io::stdin().is_terminal() {
        let mut stdin_str = String::new();
        let _ = io::stdin().read_to_string(&mut stdin_str);
        stdin_str
    } else {
        String::from("")
    };

    let mut prompt = cli.prompt;
    if !stdin_str.is_empty() {
        prompt = format!("{}\n\n{}", prompt, stdin_str)
            .trim_end()
            .to_string();
    }
    let system_prompt = if cli.code {
        String::from("Provide only code as output without any description.\nProvide only code in plain text format without Markdown formatting.\nDo not include symbols such as ``` or ```language.\nIf there is a lack of details, provide most logical solution.\nYou are not allowed to ask for more details.\nFor example if the prompt is \"Hello world Rust\", you should return \"fn main() {\\n    println!(\"Hello, world!\");\\n}\".")
    } else {
        String::from("")
    };

    let options = LLMOptions {
        prompt,
        system_prompt,
        model: chat::Provider::OpenAI(chat::OpenAIModel::Gpt4o), //cli.model,
        api_key: figment.api_key,
    };
    let mut stream = LLM::chat_stream(options).await.unwrap();

    let mut lock = stdout().lock();
    while let Some(fragment) = stream.recv().await {
        write!(lock, "{}", fragment).unwrap();
        stdout().flush()?;
    }

    Ok(())
}
