use rusqlite::Connection;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{debug,info};

type Dupa<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Debug)]
pub enum Anki {
    AnkiDb { data: HashMap<String, DateTime<Utc>> },
    None
}

impl Anki {
    pub const PATTERN: &'static str = "<.*?>";
    pub const FLDS_PATTERN: &'static str = r"^(<div>)?(?P<key>[^<]+)(<\/div>)?\u{001F}(.+)<(div|br)>(?P<val>[^<\n]+)(<\/div>)?";
    pub const NO_ANKI: &'static str = "No Anki database found";
    pub fn new(fname: &str) -> Self {
        if let Ok(c) = Connection::open_with_flags(fname, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY) {
            let mut st = c.prepare("SELECT REPLACE(sfld, CHAR(10), ' '),id FROM notes").unwrap();
            let mut e_it = st.query([]).unwrap();
            let mut data = HashMap::new();
            debug!("Loading Anki");
            while let Ok(e) = e_it.next() {
                if let Some(e) = e {
                    let word: String = e.get_unwrap(0);
                    let ts: i64 = e.get_unwrap(1);
                    let date = DateTime::from_timestamp_millis(ts).unwrap();
                    data.insert(word, date);
                } else {
                    break;
                }
            }
            Self::AnkiDb { data }
        } else {
            info!("Anki disabled");
            return Self::None
        }

    }

    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            _ => false,
        }
    }

    pub fn is_db(&self) -> bool {
        match self {
            Self::AnkiDb{..} => true,
            _ => false,
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        match self {
            Self::None => false,
            Self::AnkiDb { data } => {
                data.contains_key(key)
            }
        }
    }

    pub fn search(&self, key: &str) -> Vec<&String> {
        match self {
            Self::None => vec![],
            Self::AnkiDb { data } => {
                data.keys()
                    .filter(|&k| k.contains(key))
                    .collect()
            }
        }
    }


}
