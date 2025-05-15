use rusqlite::{Connection, Row};
use std::fmt;
use crate::Dupa;
use rayon::prelude::*;
use tracing::debug;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Entry {
    sim: String,
    tra: String,
    pin: String,
    mea: String,
    hsk: Option<u32>,
    chr: bool,
    idx: char,
}

impl Entry {
    pub fn from_row(r: &Row) -> Self {
        let sim: String = r.get_unwrap(0);
        let chr = sim.chars().count() == 1;
        let idx = sim.chars().nth(0).unwrap_or('?');
        Self {
            sim,
            tra: r.get_unwrap(1),
            pin: r.get_unwrap(2),
            mea: r.get_unwrap(3),
            hsk: r.get_unwrap(4),
            chr,
            idx,
        }
    }

    pub fn index(&self) -> char {
        self.idx
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hsk = if self.hsk.is_some() { format!("HSK{}", self.hsk.unwrap()) } else { String::new() };
        write!(f, "- {} | {} [{}] {}\n- {}", self.sim, self.tra, self.pin, hsk, self.mea.replace("/","\n- "))
    }
}

pub struct Cedict {
    data_t: BTreeMap<char, Vec<Entry>>,
}

impl Cedict {
    pub fn new(fname: &str) -> Dupa<Self> {
        let conn = Connection::open_with_flags(fname, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let mut st = conn.prepare("SELECT * from Cedict")?;
        
        let mut data_t: BTreeMap<char, Vec<Entry>> = BTreeMap::new();
        let mut data_tr = st.query([])?;
        while let Ok(next) = data_tr.next() {
            if let Some(row) = next {
                let e = Entry::from_row(row);
                let k = e.index();
                data_t.entry(k).or_default().push(e);
            } else {
                break;
            }
        }
        Ok(Self { 
            data_t,
        })
    }

    pub fn characters(&self) -> Vec<&Entry> {
        self.data_t.par_iter()
            .map(|(_,v)| v.iter().filter(|&e| e.chr).collect())
            .reduce(|| vec![], |a,b| ([a,b]).concat() )
    }

    fn characters_filtered(&self, s: &str) -> Vec<&Entry> {
        self.data_t.par_iter()
            .map(|(_,v)| v.iter().filter(|&e| e.chr && (s.contains(e.sim.as_str()) || s.contains(e.tra.as_str())) ).collect())
            .reduce(|| vec![], |a,b| ([a,b]).concat() )

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
        self.data_t.len()
    }

    /// Search all containing
    pub fn search(&self, s: &str) -> Vec<&Entry> {
        self.data_t.par_iter()
            .map(|(_,v)| v.iter().filter(|&e| e.sim.contains(s)).collect() )
            .reduce(|| vec![], |a,b| ([a,b]).concat() )
    }

    /// Search exact match
    pub fn find(&self, s: &str) -> Vec<&Entry> {
        debug!("find: {}", s);
        let c = s.chars().nth(0).unwrap();
        let r = self.data_t.get(&c).unwrap();
        r.iter()
            .filter(|e| e.sim.as_str() == s)
            .collect()
    }



}
