//! Shared multi-provider model factory for ADK-UI examples.
//!
//! Default: **OpenAI** (`OPENAI_API_KEY`, model `gpt-4o-mini` or `ADK_UI_MODEL`).
//! Switch with `ADK_UI_PROVIDER=openai|gemini` (or aliases).

use adk_core::Llm;
use adk_model::gemini::GeminiModel;
use adk_model::openai::{OpenAIClient, OpenAIConfig};
use anyhow::{Context, Result, bail};
use std::sync::Arc;

/// Supported LLM providers for demos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmProvider {
    OpenAi,
    Gemini,
}

impl LlmProvider {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "openai" | "oai" | "gpt" => Some(Self::OpenAi),
            "gemini" | "google" => Some(Self::Gemini),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Gemini => "gemini",
        }
    }
}

/// Resolve provider from env. Defaults to OpenAI.
pub fn resolve_provider() -> LlmProvider {
    std::env::var("ADK_UI_PROVIDER")
        .ok()
        .as_deref()
        .and_then(LlmProvider::parse)
        .unwrap_or(LlmProvider::OpenAi)
}

/// Build a model for the resolved (or overridden) provider.
pub fn build_model(provider: LlmProvider) -> Result<(Arc<dyn Llm>, String, LlmProvider)> {
    match provider {
        LlmProvider::OpenAi => {
            let api_key = std::env::var("OPENAI_API_KEY").context(
                "OPENAI_API_KEY is required when ADK_UI_PROVIDER=openai (default). \
                 Export OPENAI_API_KEY, or set ADK_UI_PROVIDER=gemini with GOOGLE_API_KEY.",
            )?;
            let model_name = std::env::var("ADK_UI_MODEL")
                .or_else(|_| std::env::var("OPENAI_MODEL"))
                .unwrap_or_else(|_| "gpt-4o-mini".to_string());
            let client = OpenAIClient::new(OpenAIConfig::new(api_key, &model_name))
                .context("failed to create OpenAI client")?;
            Ok((Arc::new(client), model_name, provider))
        }
        LlmProvider::Gemini => {
            let api_key = std::env::var("GOOGLE_API_KEY")
                .or_else(|_| std::env::var("GEMINI_API_KEY"))
                .context(
                    "GOOGLE_API_KEY (or GEMINI_API_KEY) is required when ADK_UI_PROVIDER=gemini",
                )?;
            let model_name = std::env::var("ADK_UI_MODEL")
                .or_else(|_| std::env::var("GEMINI_MODEL"))
                .unwrap_or_else(|_| "gemini-2.5-flash".to_string());
            let model =
                GeminiModel::new(&api_key, &model_name).context("failed to create Gemini model")?;
            Ok((Arc::new(model), model_name, provider))
        }
    }
}

/// Build the default demo model (OpenAI unless ADK_UI_PROVIDER overrides).
pub fn build_default_model() -> Result<(Arc<dyn Llm>, String, LlmProvider)> {
    build_model(resolve_provider())
}

/// Validate the API key for the provider that will actually be selected.
pub fn require_any_provider_env() -> Result<()> {
    match resolve_provider() {
        LlmProvider::OpenAi if std::env::var_os("OPENAI_API_KEY").is_none() => {
            bail!(
                "OPENAI_API_KEY is required for the default OpenAI provider. Set it, or select Gemini with ADK_UI_PROVIDER=gemini."
            );
        }
        LlmProvider::Gemini
            if std::env::var_os("GOOGLE_API_KEY").is_none()
                && std::env::var_os("GEMINI_API_KEY").is_none() =>
        {
            bail!("GOOGLE_API_KEY (or GEMINI_API_KEY) is required when ADK_UI_PROVIDER=gemini.");
        }
        _ => {}
    }
    Ok(())
}
