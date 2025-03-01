pub enum Provider {
    Anthropic(AnthropicModel),
    OpenAI(OpenAIModel),
}

pub enum AnthropicModel {
    Claude35,
    Claude37,
    Custom(String),
}

pub enum OpenAIModel {
    Gpt35,
    Gpt4o,
    Custom(String),
}

use async_openai::{
    config::OpenAIConfig,
    //error::OpenAIError,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use futures::StreamExt;
use tokio::sync::mpsc::UnboundedReceiver;

pub struct ProviderOptions {
    pub prompt: String,
    pub system_prompt: String,
    pub model: Provider,
    //pub messages:
    pub api_key: String,
}

/*
impl Default for ProviderOptions {
    fn default() -> Self {
        ProviderOptions {
            prompt: String::from(""),
            system_prompt: String::from(""),
            model: Provider::Anthropic(AnthropicModel::Claude37),
            api_key: String::from(""),
        }
    }
}
*/

pub trait LLMProvider {
    fn chat_stream(
        options: ProviderOptions,
    ) -> impl std::future::Future<Output = Result<UnboundedReceiver<String>, ()>> + Send;
}

pub struct LLM {}

impl LLMProvider for LLM {
    async fn chat_stream(options: ProviderOptions) -> Result<UnboundedReceiver<String>, ()> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        match options.model {
            Provider::Anthropic(model) => {}

            Provider::OpenAI(model) => {
                let client = Client::with_config(OpenAIConfig::new().with_api_key(options.api_key));

                let request = CreateChatCompletionRequestArgs::default()
                    .model("gpt-4o")
                    .max_tokens(4096u16)
                    .messages([
                        ChatCompletionRequestSystemMessageArgs::default()
                            .content(options.system_prompt)
                            .build()
                            .unwrap()
                            .into(),
                        ChatCompletionRequestUserMessageArgs::default()
                            .content(options.prompt)
                            .build()
                            .unwrap()
                            .into(),
                    ])
                    .build()
                    .unwrap();

                let mut stream = client.chat().create_stream(request).await.unwrap();

                tokio::spawn(async move {
                    while let Some(response) = stream.next().await {
                        match response {
                            Ok(response) => {
                                if let Some(t) = response.choices[0].delta.content.clone() {
                                    if tx.send(t).is_err() {
                                        eprintln!("Error in channel");
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Error in OpenAI stream: {}", e);
                                break;
                            }
                        }
                    }
                });
            }
        };

        Ok(rx)
    }
}
