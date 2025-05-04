#[cfg(target_family="unix")]
use wl_clipboard_rs::paste::{get_contents, ClipboardType, MimeType, Seat};
#[cfg(target_family="windows")]
use clipboard_win::{formats, get_clipboard};
use std::io::Read;
use std::fmt;
use crate::chat::{AiChat, ChatPrompt};

#[derive(Clone, Debug, PartialEq)]
pub enum ChatQuestions {
    MeaningWord,
    Example,
    Synonyms,
}

impl fmt::Display for ChatQuestions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::MeaningWord => "Explain the meaning and the usage of the word: ",
            Self::Example => "Give some example sentences using the word: ",
            Self::Synonyms => "这个词的同义词是什么？:",
        })
    }
}

impl ChatQuestions {
    pub fn to_prompt(&self, ai: &AiChat, w: &str) -> ChatPrompt {
        ChatPrompt {
            chat: ai.clone(),
            prompt: format!("{} {}", self, w),
        }
    }
}


//--------------- Enums -------------

#[macro_export]
macro_rules! make_enum {
    ($name:ident, [$op1:ident, $($opt:ident),*]) => {
        #[derive(Clone, Debug, Copy, PartialEq)]
        pub enum $name {
            $op1,
            $(
                $opt,
            )*
        }

        impl Default for $name {
            fn default() -> Self {
                $name::$op1
            }
        }

        impl $name {
            // Fixed array with commas
            pub const ALL: &'static [Self] = &[$name::$op1, $($name::$opt),+];

            pub fn to_string(&self) -> String {
                match self {
                    $name::$op1 => stringify!($op1).to_string(),
                    $(
                        $name::$opt => stringify!($opt).to_string(),
                    )*
                }
            }

            pub fn as_str(&self) -> &str {
                match self {
                    $name::$op1 => stringify!($op1),
                    $(
                        $name::$opt => stringify!($opt),
                    )*
                }
            }
        }

        impl Into<$name> for String {
            fn into(self) -> $name {
                let s = self.as_str();
                match s {
                    stringify!($op1) => $name::$op1,
                    $(
                        stringify!($opt) => $name::$opt,
                    )*
                        _ => $name::$op1,
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str(self.to_string().as_str())
            }
        }
    };
}

#[macro_export]
macro_rules! report_err {
    ($ex:expr) => {
        if let Err(e) = $ex {
            error!("{}", e.to_string());
        }
    }
}

// Get image from clipboard
#[cfg(target_family="windows")]
pub fn get_image() -> Vec<u8> {
    if let Ok(x) = get_clipboard(formats::Bitmap) {
        return x
    }
    vec![]
}

#[cfg(target_family="unix")]
pub fn get_image() -> Vec<u8> {
    let c = get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Specific("image/png"));
    let mut content = vec![];
    if let Ok((mut pipe, _)) = c {
        pipe.read_to_end(&mut content).unwrap();
        return content
    }
    vec![]
}
// -- end: get image

