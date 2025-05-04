use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type Label = HashMap<String, String>;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Keys {
    pub openai: String,
    pub deepl: String,
    pub deepseek: String,
    pub elevenlabs: String,
    pub grok: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Window {
    pub w: f32,
    pub h: f32,
    pub font_size: f32,
    pub text_c_size: Option<f32>,
    pub but_w: Option<f32>,

    pub lang: String,
    pub theme: String,
    pub font: String,
    pub padding: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub ocr_models: String,
    pub openai_model: String,
    pub deepseek_model: String,
    pub grok_model: String,
    pub api_keys: Keys,
    pub window: Window,

    pub deepseek: String,
    pub gpt: String,
    pub anki: String,
    pub grok: String,

    pub ollama_url: String,
    pub ollama_port: u16,
    pub ollama_model: String,

    pub voice: String,

    pub rec_min_score: Option<f32>,
}

