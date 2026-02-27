use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
};

pub struct OpenAiClient {
    inner: Client<OpenAIConfig>,
}

impl OpenAiClient {
    pub fn new(api_key: &str) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        OpenAiClient { inner: Client::with_config(config) }
    }

    pub async fn complete(&self, system: &str, user: &str) -> Result<String, String> {
        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o-mini")
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(system)
                    .build()
                    .map_err(|e| format!("failed to build system message: {e}"))?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(user)
                    .build()
                    .map_err(|e| format!("failed to build user message: {e}"))?
                    .into(),
            ])
            .build()
            .map_err(|e| format!("failed to build AI request: {e}"))?;

        let response = self
            .inner
            .chat()
            .create(request)
            .await
            .map_err(|e| format!("OpenAI API error: {e}"))?;

        response
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .ok_or_else(|| "empty AI response".to_string())
    }
}
