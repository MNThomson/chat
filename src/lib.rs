use async_openai::{
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
    Client,
};
use futures::StreamExt;
use tokio::sync::mpsc::UnboundedReceiver;

pub async fn openai_stream(
    prompt: &str,
    model: &str,
) -> anyhow::Result<UnboundedReceiver<Option<String>>> {
    let client = Client::new();

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .max_tokens(1024u16)
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
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
