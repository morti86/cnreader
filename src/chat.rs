use openai::{
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole}, Credentials
};
use iced::task::{Never, Sipper, sipper};
use std::sync::Arc;
use deepl::DeepLApi;
use anyhow::{Result, anyhow};
use paddleocr_rs::{Det, Rec};
use tokio_stream::StreamExt;
use elevenlabs_rs::*;
use ollama_rs::generation::completion::request::GenerationRequest;
use crate::make_enum;
use tracing::{debug, error, info};
use tokio::sync::mpsc::error::TryRecvError;

make_enum!(AiChat, [ChatGPT, Deepseek, Grok, Ollama]);

pub struct ChatPrompt {
    pub chat: AiChat,
    pub prompt: String,
}

#[derive(Debug, Clone)]
pub enum Event {
    MessageReceived(String),
    Error(String),
    End,
}

/// Return (URL, model, key)
fn get_ai_conf(chat: &AiChat) -> Option<(&String, &String, &String)> {
    let conf = crate::CONFIG.get().unwrap();
    match chat {
        AiChat::Grok => Some((&conf.grok, &conf.grok_model, &conf.api_keys.grok)),
        AiChat::ChatGPT => Some((&conf.gpt, &conf.openai_model, &conf.api_keys.openai)),
        AiChat::Deepseek => Some((&conf.deepseek, &conf.deepseek_model, &conf.api_keys.deepseek)),
        _ => {
            error!("Invalid AI configuration, it shouldn'e even be here");
            None
        }
    }
}

pub fn connect() -> impl Sipper<Never, Event> {
    sipper(async |mut output| {
        loop {
            let recv = crate::RECV.wait();
            // Receive prompt (pressed ask chat button)
            if let Ok(prompt) = recv.recv().await {
                info!("Received prompt");
                match prompt.chat {
                    AiChat::Ollama => {
                        info!("Ollama");
                        let model = crate::CONFIG.get().unwrap().ollama_model.clone();
                        let prompt = prompt.prompt;
                        debug!("Prompt: {}", prompt);
                        let request = GenerationRequest::new(model, prompt.as_str());
                        let ollama = crate::OLLAMA.lock().await;
                        let mut stream = ollama.generate_stream(request).await.unwrap();
                        while let Some(res) = stream.next().await {
                            match res {
                                Ok(responses) => {
                                    for r in responses {
                                        let content = &r.response;
                                        debug!("Received chat: {}", content);
                                        output.send(Event::MessageReceived(content.clone())).await;
                                    }
                                }
                                Err(e) => {
                                    error!("Error sending: {}", e.to_string());
                                    output.send(Event::Error(e.to_string())).await;
                                }
                            }
                        }
                    }
                    _ => {
                        let ai_chat = prompt.chat;
                        info!("Ai: {}", ai_chat);
                        if let Some((url, model, key)) = get_ai_conf(&ai_chat) {
                            let prompt = prompt.prompt;
                            debug!("Prompt: {}", prompt);
                            debug!("Key {}", key);

                            let c = Credentials::new(key, url);
                            let messages = vec![ChatCompletionMessage {
                                role: ChatCompletionMessageRole::User,
                                content: Some(prompt),
                                name: None,
                                function_call: None,
                                tool_calls: None,
                                tool_call_id: None,
                            }];
                            let dur = std::time::Duration::from_millis(200);
                            let cc = ChatCompletion::builder(model.as_str(), messages.clone())
                                .credentials(c.clone())
                                .stream(true)
                                .create_stream()
                                .await;

                            match cc {
                                Ok(mut cc) => {
                                    
                                    let mut d = true;
                                    while d {
                                        let r = cc.try_recv();
                                        match r {
                                            Ok(r) => {
                                                debug!("Got OK");
                                                let choice = &r.choices[0];
                                                if let Some(content) = &choice.delta.content {
                                                    debug!("Received chat content: {}", content);
                                                    output.send(Event::MessageReceived(content.clone())).await;
                                                }
                                            }
                                            Err(TryRecvError::Empty) => {
                                                debug!("Empty stream");
                                                tokio::time::sleep(dur).await;
                                                debug!("Empty stream: awake");
                                            }
                                            Err(TryRecvError::Disconnected) => {
                                                debug!("** DC **");
                                                d = false;
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Error requesting: {}", e.to_string());
                                    output.send(Event::Error(e.to_string())).await;
                                }
                            };

                        } else {
                            error!("Failed to get chat configuration");
                        }
                        
                    }
                }
                debug!("Finished streaming");
                output.send(Event::End).await;
            } else {
                error!("Error getting prompt");
            }
        }
    })
}

//------- DEEPL --------------

pub async fn ask_deepl_a(question: Arc<String>) -> Result<deepl::TranslateTextResp, deepl::Error> {
    let key = crate::CONFIG.wait().api_keys.deepl.as_str();
    let api = DeepLApi::with(key).new();
    api.translate_text(question, deepl::Lang::EN_US).await
}

//------- OCR -------------

pub async fn ocr(content: &Vec<u8>) -> Result<String> {
    debug!("Waiting for CONFIG");
    let ocr_path = &crate::CONFIG.wait().ocr_models;
    let rec_min_score = crate::CONFIG.get().unwrap().rec_min_score;
    let det_path = ocr_path.to_owned()+"ch_PP-OCRv4_det_infer.onnx";
    let keys = ocr_path.to_owned() + "ppocr_keys_v1.txt";
    let rec_path = ocr_path.to_owned()+"ch_PP-OCRv4_rec_infer.onnx";
    
    debug!("Init OCR");
    let det = Det::from_file(det_path.as_str())?;
    let rec = Rec::from_file(rec_path.as_str(),keys.as_str())?.with_min_score(rec_min_score.unwrap_or(0.8));
    let img = image::load_from_memory(content.as_slice())?;
    debug!("Image loaded from memory");

    let mut res = String::from("");
    for sub in det.find_text_img(&img)? {
        res.push_str(rec.predict_str(&sub)?.as_str());
    }
    Ok(res)
}

pub async fn ocr_file(file_name: &std::path::PathBuf) -> Result<String> {
    let ocr_path = &crate::CONFIG.wait().ocr_models;
    //let ocr_path = conf.ocr_models.as_str();
    let det_path = ocr_path.to_owned()+"ch_PP-OCRv4_det_infer.onnx";
    let keys = ocr_path.to_owned() + "ppocr_keys_v1.txt";
    let rec_path = ocr_path.to_owned()+"ch_PP-OCRv4_rec_infer.onnx";

    debug!("Init OCR");
    let det = Det::from_file(det_path.as_str())?;
    let rec = Rec::from_file(rec_path.as_str(),keys.as_str())?;
    let img = image::ImageReader::open(file_name)?;
    debug!("Image file opened");

    let mut res = String::new();
    let r = img.decode()?;
    debug!("Decoded image");
    for sub in det.find_text_img(&r)? {
        res.push_str(rec.predict_str(&sub)?.as_str());
    }
    Ok(res)
}

pub async fn el_play(text: Arc<String>) -> Result<elevenlabs_rs::Bytes> {
    debug!("Waiting for CONFIG");
    let key = crate::CONFIG.wait().api_keys.elevenlabs.as_str();
    let voice = crate::CONFIG.wait().voice.as_str();
    debug!("Init client");
    let client = ElevenLabsClient::new(key);
    let body = TextToSpeechBody::new(text.as_str(), Model::ElevenMultilingualV2);
    let endpoint = TextToSpeech::new(voice, body);   
    let speech = client.hit(endpoint).await;
    debug!("received speech");
    match speech {
        Ok(r) => Ok(r),
        Err(e) => Err(anyhow!(e.to_string())),
    }
}
