use serde::{Deserialize, Serialize};
use elevenlabs_rs::PreMadeVoiceID;
use std::collections::HashMap;

type Label = HashMap<String, String>;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Keys {
    pub openai: String,
    pub deepl: String,
    pub deepseek: String,
    pub elevenlabs: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Copy)]
#[serde(remote = "PreMadeVoiceID")]
pub enum PreMadeVoiceIDDef {
    Adam,
    Alice,
    Antoni,
    Arnold,
    Bill,
    Brian,
    Callum,
    Charlie,
    Chris,
    Clyde,
    Daniel,
    Dave,
    Dorothy,
    Drew,
    Domi,
    Eli,
    Emily,
    Ethan,
    Fin,
    Freya,
    George,
    Gigi,
    Giovanni,
    Glinda,
    Grace,
    Harry,
    James,
    Jessie,
    Jeremy,
    Joseph,
    Josh,
    Liam,
    Lily,
    Matilda,
    Michael,
    #[default]
    Mimi,
    Nicole,
    Patrick,
    Paul,
    Rachel,
    Sam,
    Sarah,
    Serena,
    Thomas,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Window {
    pub w: f32,
    pub h: f32,
    pub font_size: f32,

    pub lang: String,
    pub theme: String,
    pub font: String,
    pub padding: u16,
    // Labels
    pub default: String,
    pub ids_ocr: Label,
    pub ids_ocr_file: Label,
    pub ids_gpt: Label,
    pub ids_deepseek: Label,
    pub ids_meaning: Label,
    pub ids_examples: Label,
    pub ids_deepl: Label,
    pub ids_play: Label,
    pub ids_to_sim: Label,
    pub ids_anki: Label,

    pub ids_s_openai_key: Label,

    pub er_no_word_sel: Label,
    pub er_no_chat_sel: Label,
    pub er_no_file: Label,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ChatQuestions {
    pub meaning: Label,
    pub examples: Label,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub ocr_models: String,
    pub openai_model: String,
    pub deepseek_model: String,
    pub api_keys: Keys,
    pub window: Window,

    pub deepseek: String,
    pub gpt: String,
    pub anki: String,

    pub ollama_url: String,
    pub ollama_port: u16,
    pub ollama_model: String,

    pub questions: ChatQuestions,
    #[serde(with = "PreMadeVoiceIDDef")]
    pub voice: PreMadeVoiceID,
}

