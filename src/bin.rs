use std::{
    env,
    io::{self, stdout, IsTerminal, Read, Write},
};

use anyhow::Ok;
use chat::{OpenAI, Provider, ProviderOptions};
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(short, long, default_value = "gpt-4o")]
    model: String,
    prompt: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let stdin_str = if !io::stdin().is_terminal() {
        let mut stdin_str = String::new();
        let _ = io::stdin().read_to_string(&mut stdin_str);
        stdin_str
    } else {
        String::from("")
    };

    let options = ProviderOptions {
        prompt: format!("{}\n\n{}", cli.prompt, stdin_str)
            .trim_end()
            .to_string(),
        model: cli.model,
        api_key: env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set"),
    };
    let mut stream = OpenAI::chat_stream(options).await?;

    let mut lock = stdout().lock();
    while let Some(result) = stream.recv().await {
        match result {
            Some(response) => {
                write!(lock, "{}", response).unwrap();
            }
            None => {
                writeln!(lock).unwrap();
                break;
            }
        }
        stdout().flush()?;
    }

    Ok(())
}
