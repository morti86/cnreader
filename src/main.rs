#![allow(dead_code)]
use tokio::sync::Mutex;
use once_cell::sync::{Lazy, OnceCell};
use std::fs;
use tracing::{debug, error, info};
use std::sync::Arc;
use iced::widget::{button, column, row, text_editor, Button, scrollable,
text, stack, container, opaque, center, mouse_area, combo_box, ComboBox};
use iced::{Element, Subscription};
use helper::ChatQuestions;
use std::path::Path;

mod config;
mod anki;
mod chat;
mod helper;
mod cedict;

type Dupa<T> = Result<T, Box<dyn std::error::Error>>;

make_enum!(AiChat, [ChatGPT, Deepseek, Grok, Ollama]);

static RESULT: Lazy<Mutex<scrollable::Id>> = Lazy::new(|| {
    Mutex::new(scrollable::Id::unique())
});

static RECV: OnceCell<async_channel::Receiver<chat::ChatPrompt>> = OnceCell::new();
static CONFIG: OnceCell<config::Config> = OnceCell::new();

static OLLAMA: Lazy<Mutex<ollama_rs::Ollama>> = Lazy::new(|| {
    let url = CONFIG.get().unwrap().ollama_url.clone();
    let port = CONFIG.get().unwrap().ollama_port;
    Mutex::new( ollama_rs::Ollama::new(url, port) )
});

// Message

#[derive(Debug, Clone)]
enum Message {
    EditAction(text_editor::Action),
    ResultAction(text_editor::Action),
    AppendResult(String),
    AppendText(String),
    SetTextWithCursor(String,i32),
    ChatStreamEvent(chat::Event),
    AskChat(ChatQuestions),
    AiSelected(chat::AiChat),
    ShowError(Arc<String>),
    Void,
    Ocr,
    OcrFile,
    HideModal,
    Deepl,
    ToSimplified,
    ShowAnki,
    Play,
    SaveFile,
    ReadFile,
}

struct Reader {
    chat_sx: async_channel::Sender<chat::ChatPrompt>,

    text: text_editor::Content,
    result: text_editor::Content,
    cedict: cedict::Cedict,

    show_anki: bool,
    show_modal: bool,

    modal_text: String,

    ai_states: combo_box::State<chat::AiChat>,
    ai: Option<chat::AiChat>,

    anki: anki::Anki,
}

pub fn run(theme: &str) -> Result<(), iced::Error> {
    for e in iced::Theme::ALL {

        debug!("Run with theme {}", theme);
        if *theme.to_string() == e.to_string() {
            return iced::application(Reader::new, Reader::update, Reader::view)
                .subscription(Reader::subscription)
                .theme(|_| e.clone())
                .title(Reader::title)
                .run();
        }
    }
    iced::application(Reader::new, Reader::update, Reader::view)
        .subscription(Reader::subscription)
        .run()
}

impl Reader {
    const FNAME: &'static str = "dict.db";
    const SAVE: &'static str = "save";

    pub fn new() -> Self {
        let (chat_sx, chat_rx) = async_channel::unbounded();
        debug!("Set recv");
        let rr = RECV.set(chat_rx);

        debug!("Anki init");
        let anki = anki::Anki::new(&*shellexpand::tilde(CONFIG.get().unwrap().anki.as_str()));

        debug!("Anki done");
        match rr {
            Err(_e) => {
                error!("Cell already initialized");
            },
            Ok(_k) => {},
        }
        debug!("Self-init");

        let cedict = cedict::Cedict::new(Self::FNAME).expect("Failed to load the dictionary");
        debug!("Cedict: {}", cedict.len());
        Self {
            text: text_editor::Content::new(),
            result: text_editor::Content::new(),
            chat_sx,

            cedict,
            show_modal: false,
            show_anki: false,

            modal_text: String::new(),

            ai: None,
            ai_states: combo_box::State::new(chat::AiChat::ALL.to_vec()),

            anki,
        }
    }

    fn display_av(&mut self, msg: &str) {
        self.modal_text = msg.to_string();
        self.show_modal = true;
    }

    fn title(&self) -> String {
        "Chinese Reader".to_string()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(chat::connect).map(Message::ChatStreamEvent)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let wn = &CONFIG.get().unwrap().window;
        let h = wn.h;
        let w = wn.w;
        let font_size = wn.font_size;
        let but_w = wn.but_w.unwrap_or(95.0);
        let ocr_models = &CONFIG.get().unwrap().ocr_models;
        let is_deepl = !CONFIG.get().unwrap().api_keys.deepl.is_empty();
        let is_el = !CONFIG.get().unwrap().api_keys.elevenlabs.is_empty();
        let save_exists = Path::new(Self::SAVE).exists();

        let idc_text: Element<'_, Message> = text_editor( &self.text )
            .placeholder("Paste text here")
            .on_action(Message::EditAction)
            .height(h*0.50)
            .size(font_size)
            .into();

        let is_sel = self.text.selection().is_some();
        let mut ocr_ex = false;      
        if Path::new(&format!("{}{}", ocr_models,"ch_PP-OCRv4_det_infer.onnx")).exists()
            && Path::new(&format!("{}{}", ocr_models, "ppocr_keys_v1.txt")).exists()
            && Path::new(&format!("{}{}", ocr_models, "ch_PP-OCRv4_rec_infer.onnx")).exists() {
                ocr_ex = true;
        }

        let idc_ocr: Button<Message> = if ocr_ex { button("OCR").on_press(Message::Ocr) } else { button("OCR") }; 
        let idc_ocr_file: Button<Message> = if ocr_ex { button("OCR File").on_press(Message::OcrFile) } else { button("OCR") };
        let idc_deepl: Button<Message> = if is_deepl && is_sel {
            button("Deepl").on_press(Message::Deepl)
        } else {
            button("Deepl")
        };

        let idc_result: Element<'_, Message> = text_editor( &self.result )
            .placeholder("")
            .on_action(Message::ResultAction)
            .height(h*0.40)
            .size(font_size-3.0)
            .into();

        let idc_ai: ComboBox<chat::AiChat, Message> = combo_box(&self.ai_states, "", self.ai.as_ref(), Message::AiSelected).width(100.0);

        let idc_meaning: Button<Message> = if is_sel { button("Meaning").on_press(Message::AskChat(helper::ChatQuestions::MeaningWord)) } else { button("Meaning") }.width(but_w);
        let idc_examples: Button<Message> = if is_sel { button("Examples").on_press(Message::AskChat(helper::ChatQuestions::Example)) } else { button("Examples") }.width(but_w);
        let idc_synonyms: Button<Message> = if is_sel { button("Synonyms").on_press(Message::AskChat(helper::ChatQuestions::Synonyms)) } else { button("Synonyms") }.width(but_w);

        let idc_sim: Button<Message> = button("Simplified").width(100.0).on_press(Message::ToSimplified);
        let idc_anki: Button<Message> = match (&self.anki, is_sel) {
            (anki::Anki::AnkiDb{..},true) => button("Anki").width(but_w).on_press(Message::ShowAnki),
            _ => button("Anki").width(but_w),
        }.width(55.0);

        let idc_el: Button<Message> = if is_el {
            button("Play").on_press(Message::Play)
        } else {
            button("Play")
        };

        let idc_read: Button<Message> = if save_exists {
            button("Read").on_press(Message::ReadFile)
        } else {
            button("Read")
        };

        let idc_save: Button<Message> = button("Save").on_press(Message::SaveFile);

        let buttons = row![idc_ocr, idc_ocr_file, idc_ai, idc_meaning, idc_examples, idc_synonyms, idc_deepl, idc_sim, idc_anki, idc_el, idc_read, idc_save]
            .height(h * 0.1)
            .spacing(5)
            .align_y(iced::Alignment::Center);

        let controls = column![
            idc_text, 
            idc_result, 
            buttons
            ].align_x(iced::Alignment::Center);

        if self.show_modal {
            let alert = container(
                column![ 
                    text(self.modal_text.as_str()),
                    button(text("OK")).on_press(Message::HideModal) 
                ].align_x(iced::Alignment::Center)
                .spacing(10)
                ).width(w).height(h).padding(10).align_x(iced::Alignment::Center).align_y(iced::Alignment::Center);
            alert.into()
        } else {
            controls.into()
        }
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::Void => iced::Task::none(),
            Message::EditAction(a) => {
                match a {
                    text_editor::Action::Select(_) | text_editor::Action::Drag(_) => {
                        self.text.perform(a);
                        self.result = text_editor::Content::with_text("");
                        if let Some(s) = self.text.selection() {
                            if s.len() > 15 {
                                return iced::Task::none();
                            }
                            let res = self.cedict.find(s.as_str());
                            res.iter().for_each(|v| {
                                self.result.perform( text_editor::Action::Edit(text_editor::Edit::Paste( Arc::new( v.to_string() ) ) ) );
                            });
                        }

                    },
                    _ => self.text.perform(a),
                }
                iced::Task::none()
            }
            
            Message::ResultAction(a) => {
                match a {
                    text_editor::Action::Select(_) | text_editor::Action::Scroll { .. } | text_editor::Action::Click(_) | text_editor::Action::Drag(_) => self.result.perform(a),
                    _ => (),
                }
                iced::Task::none()
            }
            Message::AppendResult(r) => {
                self.result.perform( text_editor::Action::Edit(text_editor::Edit::Paste(Arc::new(r)) ) );
                iced::Task::none()
            }
            Message::AppendText(r) => {
                self.text.perform( text_editor::Action::Edit(text_editor::Edit::Paste(Arc::new(r)) ) );
                iced::Task::none()
            }
            Message::SetTextWithCursor(r,c) => {
                self.text = text_editor::Content::with_text(r.as_str());
                debug!("Trying to move cursor to line {}", c);
                self.text.perform( text_editor::Action::Scroll { lines: c });
                for _i in 0..c {
                    self.text.perform( text_editor::Action::Move( text_editor::Motion::Down )  );
                }
                iced::Task::none()
            }

            Message::ChatStreamEvent(e) => {
                match e {
                    chat::Event::MessageReceived(m) => {
                        self.result.perform( text_editor::Action::Edit(text_editor::Edit::Paste(Arc::new(m)) ) );
                    }
                    chat::Event::End => {
                        info!("Stream ended");
                    }
                    chat::Event::Error(e) => {
                        self.display_av(e.as_str());
                    }
                }
                iced::Task::none()
            }
            Message::AiSelected(e) => {
                self.ai = Some(e);
                
                iced::Task::none()
            }
            Message::ShowError(e) => {
                self.display_av(e.as_str());
                iced::Task::none()
            }
            Message::ToSimplified => {
                let s = self.text.text();
                self.text = text_editor::Content::with_text("");
                let res = self.cedict.to_sim(s.as_str());
                self.text.perform( text_editor::Action::Edit( text_editor::Edit::Paste( Arc::new(res) ) ) );
                iced::Task::none()
            }
            Message::AskChat(q) => {
                self.result = text_editor::Content::with_text("");
                let w = self.text.selection();
                if w.is_none() {
                    return iced::Task::none();
                }
                let w = w.unwrap();

                if let Some(ai) = self.ai {
                    let prompt = q.to_prompt(&ai, w.as_str());
                    debug!("Prompt: {}", prompt.prompt);
                    let chat_sx = self.chat_sx.clone();
                    return iced::Task::perform(async move {
                        report_err!( chat_sx.send(prompt).await );
                    },
                    |_e| {
                        Message::Void
                    });
                }
                iced::Task::none()
            }
            Message::ShowAnki => {
                let s = self.text.selection();
                self.result = text_editor::Content::new();
                match &s {
                    Some(s) => {
                        let r = self.anki.search(s.as_str());
                        r.iter().for_each(|rl| self.result.perform( text_editor::Action::Edit( text_editor::Edit::Paste( Arc::new(format!("-\t{}{}", rl.trim(), "\n") ))) ));
                    },
                    None => {
                        return iced::Task::none();
                    },
                }
                iced::Task::none()
            }
            Message::Ocr => {
                self.text = text_editor::Content::with_text("");
                let content = helper::get_image();
                iced::Task::perform(async move {
                    chat::ocr(&content).await
                }, |e| {
                    match e {
                        Ok(e) => Message::AppendResult(e),
                        Err(e) => Message::ShowError(Arc::new(e.to_string()))
                    }
                })
            }
            Message::OcrFile => {
                self.text = text_editor::Content::with_text("");
                let file = rfd::FileDialog::new()
                    .add_filter("image", &["png", "bmp", "jpg"])
                    .pick_file();
                if file.is_none() {
                    return iced::Task::none();
                }

                let file = file.unwrap();
                iced::Task::perform(async move {
                    chat::ocr_file(&file).await
                }, |e| {
                    match e {
                        Ok(e) => Message::AppendResult(e),
                        Err(e) => Message::ShowError(Arc::new(e.to_string()))
                    }

                })
            }
            Message::HideModal => {
                self.show_modal = false;
                iced::widget::focus_next()
            }
            Message::Deepl => {
                self.result = text_editor::Content::with_text("");
                let s = self.text.selection().unwrap_or( self.text.text() );
                let sel = Arc::new(s);
                iced::Task::perform(chat::ask_deepl_a(sel),
                    |e| {
                    match e {
                        Ok(e) => {
                            let r = e.translations.into_iter().map(|t| { t.text }).reduce(|a,b| format!("{}{}", a, b)).unwrap_or_default();
                            Message::AppendResult(r)
                        }
                        Err(e) => {
                            error!("{}", e.to_string());
                            Message::ShowError(Arc::new(e.to_string()))
                        }
                    }
                })
            }
            Message::Play => {
                let s = Arc::new(self.text.selection().unwrap_or(self.text.text()));
                if s.is_empty() {
                    return iced::Task::none();
                }

                iced::Task::perform(async move {
                    chat::el_play(s).await
                }, |r| {
                    match r {
                        Ok(r) => { 
                            let _ = elevenlabs_rs::utils::play(r);
                            Message::Void
                        },
                        Err(e) => {
                            Message::ShowError(Arc::new(e.to_string()))
                        },
                    }
                })
            }
            Message::ReadFile => {
                iced::Task::perform(async move {
                    tokio::fs::read_to_string(Self::SAVE).await
                }, |r| {
                    match r {
                        Ok(text) => {
                            let text = text.split_once('|')
                                .unwrap_or(("0",""));
                            let c: i32 = text.0.parse().unwrap_or(0);
                            Message::SetTextWithCursor(text.1.to_string(), c)
                        }
                        Err(e) => Message::ShowError(Arc::new(e.to_string())),
                    }
                })
            }
            Message::SaveFile => {
                let cursor = self.text.cursor_position();
                let text = format!("{}|{}", cursor.0, self.text.text() );
                iced::Task::perform(async move {
                    tokio::fs::write(Self::SAVE, text.as_bytes()).await
                }, |r| {
                    match r {
                        Ok(_) => Message::Void,
                        Err(e) => Message::ShowError(Arc::new(e.to_string())),
                    }

                })
            }
        }
    }
}

impl Default for Reader {
    fn default() -> Self {
        Self::new()
    }
}

//#[tokio::main]
fn main() -> Result<(), iced::Error> {
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_max_level(tracing::Level::INFO)
        .init();

    let t: config::Config = toml::from_str( fs::read_to_string( "./app.toml" ).unwrap().as_str() ).unwrap();
    let theme: String = t.window.theme.clone();
    debug!("Set config");
    match CONFIG.set(t) {
        Ok(_) => {}
        Err(_e) => {
            error!("Config already initialized");
        }
    }

    run(theme.as_str())
}

// Alert window

fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message) -> Element<'a, Message> where Message: Clone + 'a {
    stack![
        base.into(),
        opaque(
            mouse_area(center(opaque(content)).style(|_theme| {
                container::Style {
                    background: Some(
                                    iced::Color {
                                        a: 0.8,
                                        ..iced::Color::BLACK
                                    }
                                    .into(),
                                ),
                                ..container::Style::default()
                }
            }))
            .on_press(on_blur)
        )
    ]
    .into()
}
