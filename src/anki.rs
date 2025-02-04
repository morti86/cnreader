use rusqlite::Connection;
use regex::Regex;
use std::fmt;
use chrono::DateTime;
use anyhow::Result;
use std::sync::Arc;

#[derive(Clone)]
pub struct WordsDates {
    pub sim: String,
    pub date_added: DateTime<chrono::Utc>,
}

impl WordsDates {
    pub fn new(sim: String, ts: i64) -> Self {
        Self {
            sim,
            date_added: DateTime::from_timestamp_millis(ts).unwrap()
        }
    }
}

impl fmt::Display for WordsDates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}\t{}", self.sim, self.date_added.format("%Y-%m-%d")  )
    }
}

#[derive(Clone, Debug)]
pub enum Anki {
    AnkiDb {conn: Arc<Connection>, pattern: Regex, flds_pattern: Regex},
    None,
}

impl Anki {
    pub const PATTERN: &'static str = "<.*?>";
    pub const FLDS_PATTERN: &'static str = r"^(<div>)?(?P<key>[^<]+)(<\/div>)?\u{001F}(.+)<(div|br)>(?P<val>[^<\n]+)(<\/div>)?";
    pub const NO_ANKI: &'static str = "No Anki database found";
    pub fn new(fname: &str) -> Self {
        match Connection::open_with_flags(fname, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY) {
            Ok(c) => Self::AnkiDb {conn: Arc::new(c), pattern: Regex::new(Self::PATTERN).unwrap(), flds_pattern: Regex::new(Self::FLDS_PATTERN).unwrap(), },
            _ => Self::None,
        }
    }

    fn clean_str<'b>(&self, s: &'b str) -> String {
        match self {
            Self::AnkiDb{ .. } => {
                let c = s.replace("&nbsp;","");
                c
            },
            Self::None => s.to_string(),
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

    /// Checks if the base contains the given word
    pub fn contains(&self, val: &str) -> Result<bool> {
        match self {
            Self::AnkiDb{ conn, .. } => {
                let mut st = conn.prepare("SELECT REPLACE(sfld, CHAR(10), ' ') FROM notes WHERE REPLACE(sfld, CHAR(10), '') = ?")?;
                let mut rows = st.query([val])?;
                if let Some(_) = rows.next()? {
                    return Ok(true);
                }
                anyhow::Ok(false)
            },
            Self::None => Err(anyhow::anyhow!(Self::NO_ANKI)),
        }
    }

    /// Searches for all the entries containing given word
    pub fn search(&self, val: &str) -> Result<Vec<String>> {
        match self {
            Self::AnkiDb{ conn,.. } => {
                let mut res = vec![];
                let s = format!("%{val}%");
                let mut st = conn.prepare("SELECT REPLACE(sfld, CHAR(10), ' ') FROM notes WHERE REPLACE(sfld, CHAR(10), '') LIKE ?")?;
                let mut e_it = st.query([s])?;
                while let Some(e) = e_it.next()? {
                    res.push(e.get_unwrap(0));
                }
                Ok(res)
            },
            Self::None => Err(anyhow::anyhow!(Self::NO_ANKI)),
        }
    }
}




