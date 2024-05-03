use std::io::{stdout, Write};

use anyhow::Ok;
use chat::openai_stream;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Chat(AddArgs),
    Image,
}

#[derive(Args)]
struct AddArgs {
    prompt: String,
    #[arg(short, long, default_value = "gpt-4-turbo")]
    model: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Chat(args) => {
            let mut stream = openai_stream(&args.prompt, &args.model).await?;

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
        Commands::Image => {
            todo!("Implement image generation")
        }
    }
}
