use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use futures::StreamExt;
use tokio::sync::mpsc::UnboundedReceiver;

pub enum Provider {
    Anthropic(AnthropicModel),
    OpenAI(OpenAIModel),
}

pub enum AnthropicModel {
    Claude35Haiku,
    Claude37Sonnet,
    Custom(String),
}
impl From<AnthropicModel> for String {
    fn from(val: AnthropicModel) -> Self {
        use AnthropicModel::*;
        match val {
            Claude35Haiku => String::from("claude-3-5-haiku-latest"),
            Claude37Sonnet => String::from("claude-3-7-sonnet-latest"),
            Custom(s) => s,
        }
    }
}

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

pub struct LLMOptions {
    pub prompt: String,
    pub system_prompt: String,
    pub model: Provider,
    //pub messages:
    pub api_key: String,
}

pub struct LLM {}
impl LLM {
    pub async fn chat_stream(options: LLMOptions) -> Result<UnboundedReceiver<String>, ()> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        match options.model {
            Provider::Anthropic(_model) => {}

            Provider::OpenAI(model) => {
                let client = Client::with_config(OpenAIConfig::new().with_api_key(options.api_key));

                let request = CreateChatCompletionRequestArgs::default()
                    .model(model)
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
