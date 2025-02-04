use openai::{
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole}, Credentials
};
use std::sync::Arc;
use deepl::DeepLApi;
use anyhow::{Result, anyhow};
use paddleocr_rs;
use image;
use elevenlabs_rs::*;
use ollama_rs::{
    generation::completion::{
        request::GenerationRequest,
    },
    Ollama,
};

#[derive(Copy, Debug, Clone)]
pub enum Question {
    Meaning,
    Examples,
}

#[derive(Copy, Debug, Clone)]
pub enum Chat {
    Openai,
    Deepseek,
    Ollama,
}

impl std::fmt::Display for Chat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::Openai => "Openai",
            Self::Deepseek => "Deepseek",
            Self::Ollama => "Ollama",
        })
    }
}

// Sends a request to Chat GPT/Deepseek
pub async fn ask_gpt_a(q: Question, ch: Chat, config: Arc<crate::config::Config>, w: Arc<String>) -> String {
    let key = match ch {
        Chat::Openai => config.api_keys.openai.as_str(),
        Chat::Deepseek => config.api_keys.deepseek.as_str(),
        Chat::Ollama => "",
    };
    let url = match ch { Chat::Openai => config.gpt.as_str(), Chat::Deepseek => config.deepseek.as_str(), _ => "", };
    let c = Credentials::new(key, url);
    let question = match q {
        Question::Meaning => config.questions.meaning.get( config.window.lang.as_str() ).unwrap().as_str(),
        Question::Examples => config.questions.examples.get( config.window.lang.as_str() ).unwrap().as_str(),
    };
    let messages = vec![ChatCompletionMessage {
         role: ChatCompletionMessageRole::User,
         content: Some(question.to_string()+w.as_str()),
         name: None,
         function_call: None,
         tool_calls: vec![],
         tool_call_id: None,
     }];

     let model = match ch {
         Chat::Openai => config.openai_model.as_str(),
         Chat::Deepseek => config.deepseek_model.as_str(),
         _ => "",
     };

     let chat_completion = ChatCompletion::builder(model, messages.clone())
         .credentials(c.clone())
         .create()
         .await
         .unwrap();
     let returned_message = chat_completion.choices.first().unwrap().message.clone();
     let content = returned_message.content.unwrap();

     content

}

pub async fn ask_ollama(q: Question, config: Arc<crate::config::Config>, w: Arc<String>) -> String {
    let ollama = Ollama::new(config.ollama_url.as_str(), config.ollama_port);
    let question = match q {
        Question::Meaning => config.questions.meaning.get( config.window.lang.as_str() ).unwrap().as_str(),
        Question::Examples => config.questions.examples.get( config.window.lang.as_str() ).unwrap().as_str(),
    };

    let prompt = format!("{} {}", question, w.as_str());

    let res = ollama.generate(GenerationRequest::new(config.ollama_model.to_owned(), prompt)).await;

    match res {
        Ok(res) => res.response,
        Err(e) => e.to_string(),
    }
}

pub async fn ask_deepl_a(question: Arc<String>, conf: Arc<crate::config::Config>) -> Result<deepl::TranslateTextResp, deepl::Error> {
    let api = DeepLApi::with(conf.api_keys.deepl.as_str()).new();
    let lang = conf.window.lang.as_str();
    let lang = match lang {
        "pol" => deepl::Lang::PL,
        _ => deepl::Lang::EN_US,
    };

    let res = api.translate_text(question, lang).await;
    res
}

pub fn ocr(conf: Arc<crate::config::Config>, content: &Vec<u8>) -> Result<String> {
    let ocr_path = conf.ocr_models.as_str();
    let det_path = ocr_path.to_owned()+"ch_PP-OCRv4_det_infer.onnx";
    let keys = ocr_path.to_owned() + "ppocr_keys_v1.txt";
    let rec_path = ocr_path.to_owned()+"ch_PP-OCRv4_rec_infer.onnx";
    
    let det = paddleocr_rs::Det::from_file(det_path.as_str())?;
    let rec = paddleocr_rs::Rec::from_file(rec_path.as_str(),keys.as_str())?;
    let img = image::load_from_memory(content.as_slice())?;

    let mut res = String::from("");
    for sub in det.find_text_img(&img)? {
        res.push_str(rec.predict_str(&sub)?.as_str());
    }
    Ok(res)
}

pub fn ocr_file(conf: Arc<crate::config::Config>, file_name: &std::path::PathBuf) -> Result<String> {
    let ocr_path = conf.ocr_models.as_str();
    let det_path = ocr_path.to_owned()+"ch_PP-OCRv4_det_infer.onnx";
    let keys = ocr_path.to_owned() + "ppocr_keys_v1.txt";
    let rec_path = ocr_path.to_owned()+"ch_PP-OCRv4_rec_infer.onnx";
    let det = paddleocr_rs::Det::from_file(det_path.as_str())?;
    let rec = paddleocr_rs::Rec::from_file(rec_path.as_str(),keys.as_str())?;
    let img = image::ImageReader::open(file_name)?;

    let mut res = String::new();
    let r = img.decode()?;
    for sub in det.find_text_img(&r)? {
        res.push_str(rec.predict_str(&sub)?.as_str());
    }
    Ok(res)
}

pub async fn el_play(conf: Arc<crate::config::Config>, text: Arc<String>) -> Result<elevenlabs_rs::Bytes> {
    let key = conf.api_keys.elevenlabs.as_str();
    let voice = conf.voice.clone();
    let client = ElevenLabsClient::new(key);
    let body = TextToSpeechBody::new(text.as_str(), Model::ElevenMultilingualV2);
    let endpoint = TextToSpeech::new(voice, body);   
    let speech = client.hit(endpoint).await;
    match speech {
        Ok(r) => Ok(r),
        Err(e) => Err(anyhow!(e.to_string())),
    }
}
