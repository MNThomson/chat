use futures::StreamExt;
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Debug)]
pub enum Provider {
    Anthropic(AnthropicModel),
    OpenAI(OpenAIModel),
}

#[derive(Debug, Eq, PartialEq)]
pub enum AnthropicModel {
    Claude35Haiku,
    Claude37Sonnet,
    Claude4Sonnet,
    Custom(String),
}
impl From<AnthropicModel> for String {
    fn from(val: AnthropicModel) -> Self {
        use AnthropicModel::*;
        match val {
            Claude35Haiku => String::from("claude-3-5-haiku-latest"),
            Claude37Sonnet => String::from("claude-3-7-sonnet-latest"),
            Claude4Sonnet => String::from("claude-sonnet-4-20250514"),
            Custom(s) => s,
        }
    }
}

#[derive(Debug)]
pub enum OpenAIModel {
    Gpt4o,
    Gpt4oMini,
    Custom(String),
}
impl From<OpenAIModel> for String {
    fn from(val: OpenAIModel) -> Self {
        use OpenAIModel::*;
        match val {
            Gpt4o => String::from("gpt-4o"),
            Gpt4oMini => String::from("gpt-4o-mini"),
            Custom(s) => s,
        }
    }
}

#[derive(Debug)]
pub struct LLMOptions {
    pub prompt: String,
    pub system_prompt: String,
    pub model: Provider,
    //pub messages:
    pub api_key: String,
    pub max_tokens: u16,
}

pub struct LLM {}
impl LLM {
    pub async fn chat_stream(options: LLMOptions) -> Result<UnboundedReceiver<String>, ()> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        match options.model {
            Provider::Anthropic(model) => {
                use misanthropy::{Anthropic, Content, MessagesRequest};
                let client = Anthropic::new(&options.api_key);

                let mut request = MessagesRequest::default()
                    .with_model(model)
                    .with_stream(true)
                    .with_max_tokens(options.max_tokens.into());
                request.add_system(Content::text(options.system_prompt));
                request.add_user(Content::text(options.prompt));

                let mut stream = client.messages_stream(&request).unwrap();

                tokio::spawn(async move {
                    while let Some(event) = stream.next().await {
                        let event = event.unwrap();
                        match event {
                            misanthropy::StreamEvent::ContentBlockDelta { delta, .. } => {
                                if let misanthropy::ContentBlockDelta::TextDelta { text } = delta {
                                    if tx.send(text).is_err() {
                                        eprintln!("Error in channel");
                                        break;
                                    }
                                }
                            }
                            _ => {} // Ignore other event types
                        }
                    }
                });
            }

            Provider::OpenAI(model) => {
                use async_openai::{
                    config::OpenAIConfig,
                    types::{
                        ChatCompletionRequestSystemMessageArgs,
                        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
                    },
                    Client,
                };
                let client = Client::with_config(OpenAIConfig::new().with_api_key(options.api_key));

                let request = CreateChatCompletionRequestArgs::default()
                    .model(model)
                    .max_tokens(options.max_tokens)
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
