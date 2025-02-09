use iced::widget::{button, column, row, text_editor, Button,
    text, stack, container, opaque, center, mouse_area, combo_box, ComboBox};
use iced::Element;
use crate::config;
use crate::anki;
use crate::cedict;
use std::io::Read;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use elevenlabs_rs::utils::play;

use crate::chat;

#[cfg(target_family="unix")]
use wl_clipboard_rs::paste::{get_contents, ClipboardType, MimeType, Seat};
#[cfg(target_family="windows")]
use clipboard_win::{formats, get_clipboard};

#[cfg(target_family="windows")]
fn get_image() -> Vec<u8> {
    if let Ok(x) = get_clipboard(formats::Bitmap) {
        return x
    }
    vec![]
}

#[cfg(target_family="unix")]
fn get_image() -> Vec<u8> {
    let c = get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Specific("image/png"));
    let mut content = vec![];
    if let Ok((mut pipe, _)) = c {
        pipe.read_to_end(&mut content).unwrap();
        return content
    }
    vec![]
}

#[derive(Debug, Clone)]
pub enum AiEngine {
    Openai,
    Deepseek,
}

impl std::fmt::Display for AiEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::Openai => "Openai",
            Self::Deepseek => "Deepseek",
        })
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Ocr,
    OcrFile,
    EditAction(text_editor::Action),
    ResultAction(text_editor::Action),
    ChatMeaning,
    ChatExamples,
    ShowModal,
    HideModal,
    ChatResult(Arc<String>),
    Deepl,
    Play,
    ShowError(Arc<String>),
    AiSelected(chat::Chat),
    ToSimplified,
    ShowAnki,
}

pub struct Reader {
    text: text_editor::Content,
    result: text_editor::Content,
    config: Arc<config::Config>,
    cedict: cedict::Cedict,
    anki: anki::Anki,

    show_modal: bool,
    show_anki: bool,
    modal_text: String,

    ai_states: combo_box::State<chat::Chat>,
    ai: Option<chat::Chat>,
}

impl Default for Reader {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! init_button {
    ($ids:ident, $lang:expr, $s:expr, $msg:expr) => {
        button( $s.config.window.$ids.get($lang).unwrap_or(&$s.config.window.default).as_str() ).on_press($msg)
    };
    ($ids:ident, $lang:expr, $s:expr) => {
        button( $s.config.window.$ids.get($lang).unwrap_or(&$s.config.window.default).as_str() )
    };
}

macro_rules! get_label {
    ($ids:ident, $s:expr) => {
        $s.window.$ids.get($s.window.lang.as_str()).unwrap_or(&$s.window.default)
    }
}


pub fn run(theme: &str) -> Result<(), iced::Error> {
    for e in iced::Theme::ALL {
        if theme.to_string() == e.to_string() {
            return iced::application("Reader", Reader::update, Reader::view)
                .theme(|_| e.clone())
                .run();
        }
    }
    iced::application("Reader", Reader::update, Reader::view)
        .theme(|_| iced::Theme::Light)
        .run()
}

impl Reader {
    const FNAME: &'static str = "cedict_1_0_ts_utf-8_mdbg.txt";
    const CONFIG: &'static str = "./app.toml";

    pub fn new() -> Self {
        let conf: Arc<config::Config> = Arc::new(toml::from_str( fs::read_to_string( Self::CONFIG ).unwrap().as_str() ).unwrap());
        let anki = anki::Anki::new(&*shellexpand::tilde(conf.anki.as_str()));
        Self {
            text: text_editor::Content::new(),
            result: text_editor::Content::new(), 
            config: conf, 
            cedict: cedict::Cedict::par_new(Self::FNAME),
            show_modal: false,
            show_anki: false,
            modal_text: String::new(),
            ai: None,
            ai_states: combo_box::State::new(vec![chat::Chat::Openai, chat::Chat::Deepseek, chat::Chat::Ollama]),
            anki,
        }
    }

    fn save_config(&mut self) {
        let config = self.config.clone();
        match  toml::to_string(config.as_ref()) {
            Ok(_) => {},
            Err(e) => {
                self.display_av(e.to_string().as_str());
            }
        }

    }
    
    fn display_av(&mut self, msg: &str) {
        self.modal_text = msg.to_string();
        self.show_modal = true;
    }

    pub fn view(&self) -> Element<'_, Message> {
        //let idsself.config.window.ids_ocr.get(self.lang)
        //init_idc!(ids_ocr, idc_ocr, self.config, Message::Ocr, self.config.window.lang.as_str());
        let h = self.config.window.h;
        let w = self.config.window.w;
        let lang = self.config.window.lang.as_str();
        let font_size = self.config.window.font_size;

        let idc_text: Element<'_, Message> = text_editor( &self.text )
            .placeholder("")
            .on_action(Message::EditAction)
            .height(h*0.50)
            .size(font_size)
            .into();

        let idc_result: Element<'_, Message> = text_editor( &self.result )
            .placeholder("")
            .on_action(Message::ResultAction)
            .height(h*0.40)
            .size(font_size-3.0)
            .into();
        let mut ocr_ex = false;      
        if Path::new(&format!("{}{}", self.config.ocr_models,"ch_PP-OCRv4_det_infer.onnx")).exists()
            && Path::new(&format!("{}{}", self.config.ocr_models, "ch_PP-OCRv4_det_infer.onnx")).exists()
            && Path::new(&format!("{}{}", self.config.ocr_models, "ch_PP-OCRv4_det_infer.onnx")).exists() {
                ocr_ex = true;
        }

        let idc_ocr : Button<Message> = if ocr_ex { init_button!(ids_ocr, lang, &self, Message::Ocr) } else { init_button!(ids_ocr, lang, &self) };
        let idc_ocr_file : Button<Message> = if ocr_ex { init_button!(ids_ocr_file, lang, &self, Message::OcrFile) } else { init_button!(ids_ocr_file, lang, &self) };

        let idc_ai: ComboBox<chat::Chat, Message> = combo_box(&self.ai_states, "", self.ai.as_ref(), Message::AiSelected).width(100.0);

        let idc_deepl = if self.config.api_keys.deepl.is_empty() {
            init_button!(ids_deepl, lang, &self)
        } else {
            init_button!(ids_deepl, lang, &self, Message::Deepl)
        };

        let idc_meaning: Button<Message> = init_button!(ids_meaning, lang, &self, Message::ChatMeaning);
        let idc_examples: Button<Message> = init_button!(ids_examples, lang, &self, Message::ChatExamples);

        let idc_play: Button<Message> = if self.config.api_keys.elevenlabs.is_empty() {
            init_button!(ids_play, lang, &self)
        } else {
            init_button!(ids_play, lang, &self, Message::Play)
        };
        let idc_to_simplified: Button<Message> = init_button!(ids_to_sim, lang, &self, Message::ToSimplified);

        let idc_anki: Button<Message> = match self.anki {
            anki::Anki::AnkiDb{..} => init_button!(ids_anki, lang, &self, Message::ShowAnki),
            _ => init_button!(ids_anki, lang, &self),
        };

        let button_row = row![ idc_ocr, idc_ocr_file, idc_deepl, idc_ai, idc_meaning, idc_examples, idc_play, idc_to_simplified, idc_anki ]
            .height(h*0.1)
            .spacing(5)
            .padding(self.config.window.padding)
            .align_y(iced::Alignment::Center);
        
        let controls = column![
            idc_text,
            idc_result,
            button_row
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
            Message::ShowModal => {
                self.show_modal = true;
                
                iced::widget::focus_next()
            },
            Message::HideModal => {
                self.show_modal = false;
                iced::widget::focus_next()
            },

            Message::Ocr => {
                self.text = text_editor::Content::with_text("");
                let content = get_image();
                let res = chat::ocr(self.config.clone(), &content);
                match res {
                    Ok(res) => self.text.perform( text_editor::Action::Edit(text_editor::Edit::Paste( Arc::new( res ) ) ) ),
                    Err(e) => self.display_av(e.to_string().as_str()),
                }
                iced::Task::none()
            },
            Message::OcrFile => {
                let file = rfd::FileDialog::new()
                    .add_filter("image", &["png", "bmp", "jpg"])
                    .pick_file();
                if file == None {
                    return iced::Task::none()
                }
                if let Some(r) = file {
                    let res = chat::ocr_file(self.config.clone(), &r);
                    match res {
                        Ok(res) => self.text.perform( text_editor::Action::Edit(text_editor::Edit::Paste( Arc::new( res ) ) ) ),
                        Err(e) => self.display_av(e.to_string().as_str()),
                    }
                } else {
                    self.display_av(get_label!(er_no_file, self.config.clone()));
                }
                iced::Task::none()
            },
            Message::ChatMeaning => {
                self.result = text_editor::Content::with_text("");
                let config = self.config.clone();

                let s = self.text.selection();

                if let Some(s) = s {
                    let sel = Arc::new(s);

                    match self.ai {
                        Some(chat::Chat::Ollama) => {
                            return iced::Task::perform(
                                chat::ask_ollama(chat::Question::Meaning, config, sel),
                                |e| { Message::ChatResult(Arc::new(e)) }
                            )
                        },
                        Some(ai) => {
                            return iced::Task::perform(
                                chat::ask_gpt_a(chat::Question::Meaning, ai, config, sel),
                                |e| { Message::ChatResult(Arc::new(e)) }
                            )
                        },
                        None => { self.display_av(get_label!(er_no_chat_sel, self.config.clone())); },
                    }
                } else {
                    self.display_av( get_label!(er_no_word_sel, self.config.clone()) );
                }

                iced::Task::none()
            },
            Message::ChatExamples => {
                self.result = text_editor::Content::with_text("");
                let config = self.config.clone();

                let s = self.text.selection();
                if let Some(s) = s {
                    let sel = Arc::new(s);

                    match self.ai {
                        Some(chat::Chat::Ollama) => {
                            return iced::Task::perform(
                                chat::ask_ollama(chat::Question::Examples, config, sel),
                                |e| { Message::ChatResult(Arc::new(e)) }
                            )
                        },
                        Some(ai) => {
                            return iced::Task::perform(
                                chat::ask_gpt_a(chat::Question::Examples, ai, config, sel),
                                |e| { Message::ChatResult(Arc::new(e)) }
                            )
                        },
                        None => { self.display_av(get_label!(er_no_chat_sel, self.config.clone())) },
                    }
                } else {
                    self.display_av(get_label!(er_no_word_sel, self.config.clone()).as_str());
                }
                iced::Task::none()
            },
            Message::Deepl => {
                let s = self.text.selection().unwrap_or( self.text.text() );
                let sel = Arc::new(s);
                let config = self.config.clone();

                return iced::Task::perform(
                    chat::ask_deepl_a(sel, config),
                    |e| {
                        if let Ok(e) = e {
                            let r = e.translations.into_iter().map(|t| { t.text }).reduce(|a,b| format!("{}{}", a, b)).unwrap_or_default();
                            return Message::ChatResult(Arc::new(r));
                        }
                        Message::ChatResult(Arc::new( String::from("?") ) )
                    })
                        
            },
            Message::AiSelected(e) => {
                self.ai = Some(e);
                iced::Task::none()
            },
            Message::EditAction(a) => {
                match a {
                    text_editor::Action::Select(_) | text_editor::Action::Drag(_) => {
                        self.text.perform(a);
                        self.result = text_editor::Content::with_text("");
                        if let Some(s) = self.text.selection() {
                            if s.len() > 36 {
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
            },

            Message::ResultAction(a) => {
                match a {
                    text_editor::Action::Select(_) | text_editor::Action::Scroll { .. } | text_editor::Action::Click(_) | text_editor::Action::Drag(_) => self.result.perform(a),
                    _ => (),
                }
                iced::Task::none()
            },
            
            Message::ChatResult(m) => {
                self.result.perform( text_editor::Action::Edit(text_editor::Edit::Paste(m) ) );
                iced::Task::none()
            },
            Message::Play => {
                let s = self.text.selection();
                let sel = Arc::new(s.unwrap_or(self.text.text()));
                let config = self.config.clone();
                
                return iced::Task::perform(
                    chat::el_play(config, sel),
                    |e| { 
                        match e {
                            Ok(r) => {
                                let _ = play(r);
                            },
                            Err(r) => {
                                return Message::ShowError(Arc::new(r.to_string()));
                            },
                        }
                        Message::ChatResult(Arc::new( String::from("?") ) )
                    });
                    
                //iced::Task::none()
            },
            Message::ShowError(e) => {
                self.display_av(e.as_str());
                
                iced::Task::none()
            },
            Message::ToSimplified => {
                let s = self.text.text();
                self.text = text_editor::Content::with_text("");
                let res = self.cedict.to_sim(s.as_str());
                self.text.perform( text_editor::Action::Edit( text_editor::Edit::Paste( Arc::new(res) ) ) );
                iced::Task::none()
            },
            Message::ShowAnki => {
                let s = self.text.selection();
                self.result = text_editor::Content::new();
                match &s {
                    Some(s) => {
                        let r = self.anki.search(s.as_str()).unwrap_or(vec![]);
                        r.iter().for_each(|rl| self.result.perform( text_editor::Action::Edit( text_editor::Edit::Paste( Arc::new(format!("-\t{}{}", rl.trim(), "\n") ))) ));
                    },
                    None => {
                        return iced::Task::none();
                    },
                }
                iced::Task::none()
            },
        }
    }
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
