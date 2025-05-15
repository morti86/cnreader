use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};

type Label = HashMap<String, String>;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Keys {
    pub elevenlabs: String,
    pub deepl: String,
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
pub struct AiChatConfiguration {
    pub name: String,
    pub key: String,
    pub url: String,
    pub model: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub ocr_models: String,
    pub api_keys: Keys,
    pub window: Window,
    pub anki: String,

    pub voice: String,
    pub sel_chat: String,

    pub rec_min_score: Option<f32>,

    pub ai_chats: BTreeMap<String, AiChatConfiguration>,
}

