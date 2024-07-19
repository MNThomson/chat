use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
    Client,
};
use futures::StreamExt;
use tokio::sync::mpsc::UnboundedReceiver;

pub struct ProviderOptions {
    pub prompt: String,
    pub model: String,
    pub api_key: String,
}

pub trait Provider {
    fn chat_stream(
        options: ProviderOptions,
    ) -> impl std::future::Future<Output = anyhow::Result<UnboundedReceiver<Option<String>>>> + Send;
}

pub struct OpenAI {}
impl Provider for OpenAI {
    async fn chat_stream(
        options: ProviderOptions,
    ) -> anyhow::Result<UnboundedReceiver<Option<String>>> {
        let client = Client::with_config(OpenAIConfig::new().with_api_key(options.api_key));

        let request = CreateChatCompletionRequestArgs::default()
            .model(options.model)
            .max_tokens(1024u16)
            .messages([ChatCompletionRequestUserMessageArgs::default()
                .content(options.prompt)
                .build()?
                .into()])
            .build()?;

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let mut stream = client.chat().create_stream(request).await?;

        tokio::spawn(async move {
            while let Some(response) = stream.next().await {
                match response {
                    Ok(response) => {
                        let text = response.choices[0].delta.content.clone();
                        if tx.send(text).is_err() {
                            eprintln!("Error in channel");
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error in OpenAI stream: {}", e);
                        break;
                    }
                }
            }

            Ok::<(), anyhow::Error>(())
        });

        Ok(rx)
    }
}
