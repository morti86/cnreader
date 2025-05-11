use rusqlite::{Connection, Row};
use std::fmt;
use crate::Dupa;
use rayon::prelude::*;
use tracing::debug;

#[derive(Clone, Debug)]
pub struct Entry {
    sim: String,
    tra: String,
    pin: String,
    mea: String,
    hsk: Option<u32>,
    chr: bool,
}

impl Entry {
    pub fn from_row(r: &Row) -> Self {
        let sim: String = r.get_unwrap(0);
        let chr = sim.chars().count() == 1;
        Self {
            sim,
            tra: r.get_unwrap(1),
            pin: r.get_unwrap(2),
            mea: r.get_unwrap(3),
            hsk: r.get_unwrap(4),
            chr,
        }
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hsk = if self.hsk.is_some() { format!("HSK{}", self.hsk.unwrap()) } else { String::new() };
        write!(f, "- {} | {} [{}] {}\n- {}", self.sim, self.tra, self.pin, hsk, self.mea.replace("/","\n- "))
    }
}

pub struct Cedict {
    data: Vec<Entry>,
}

impl Cedict {
    pub fn new(fname: &str) -> Dupa<Self> {
        let conn = Connection::open_with_flags(fname, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let mut st = conn.prepare("SELECT * from Cedict")?;
        let data = st.query_map([], |r|
            Ok(Entry::from_row(r))
            )?
            .map(|e| e.unwrap())
            .collect();
        Ok(Self { data })
    }

    pub fn characters(&self) -> Vec<&Entry> {
        self.data.par_iter().filter(|e| e.chr).collect()
    }

    fn characters_filtered(&self, s: &str) -> Vec<&Entry> {
        self.data.par_iter().filter(|e| { e.chr && s.contains(e.tra.as_str()) }).collect()
    }

    /// Convert traditional to simplified
    pub fn to_sim(&self, s: &str) -> String {
        let chr_list = self.characters_filtered(s);
        let ss = s.par_chars()
            .map(|z| {
                let c = chr_list.iter().find(|x| x.tra == z.to_string());
                match c {
                    Some(c) => c.sim.chars().nth(0).unwrap_or(z),
                    None => z,
                }
                
            }).collect();
        ss
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Search all containing
    pub fn search(&self, s: &str) -> Vec<&Entry> {
        self.data.par_iter().filter(|e| { e.sim.contains(s) || e.tra.contains(s) }).collect()
    }

    /// Search exact match
    pub fn find(&self, s: &str) -> Vec<&Entry> {
        debug!("find: {}", s);
        self.data.par_iter().filter(|e| { e.sim.as_str() == s || e.tra.as_str() == s }).collect()
    }



}
