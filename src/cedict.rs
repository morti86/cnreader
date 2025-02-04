use std::fmt;
use regex::{Regex, Captures};
use std::io::{BufRead, BufReader};
use std::fs;
use rayon::prelude::*;

#[derive(Clone, Debug)]
pub struct Entry {
    sim: String,
    tra: String,
    pin: String,
    mea: String,
    chr: bool,
}

impl Entry {
    pub fn from_captures(c: &Captures) -> Self {
        Self {
            sim: String::from(c.name("sim").unwrap().as_str()),
            tra: String::from(c.name("tra").unwrap().as_str()),
            pin: String::from(c.name("pin").unwrap().as_str()),
            mea: String::from(c.name("mea").unwrap().as_str()),
            chr: c.name("sim").unwrap().as_str().chars().count() == 1,
        }
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "- {} | {} [{}]\n\t- {}", self.sim, self.tra, self.pin, self.mea.replace("/","\n\t- "))
    }
}

#[derive(Clone, Debug)]
pub struct Cedict {
    entries: Vec<Entry>,
}

impl Cedict {
    pub fn new(fname: &str) -> Self {
        let pattern = r"^(?P<tra>.+) (?P<sim>.+) \[(?P<pin>.+)\] /(?P<mea>.+)/$";
        let re = Regex::new(pattern).unwrap();
        let file = fs::File::open(fname).unwrap();
        let reader = BufReader::new(file);
        let entries: Vec<Entry> = reader.lines().into_iter()
            .filter(|e| { e.is_ok() && !e.as_ref().unwrap().starts_with("#") })
            .map(|e| { 
                let e = e.unwrap();
                Entry::from_captures( &re.captures(e.as_str()).unwrap() )

            }).collect();
        Self { entries }
    }

    pub fn par_new(fname: &str) -> Self {
        let pattern = r"^(?P<tra>.+) (?P<sim>.+) \[(?P<pin>.+)\] /(?P<mea>.+)/$";
        let re = Regex::new(pattern).unwrap();
        //let file = fs::File::open(fname).unwrap();
        let data = fs::read_to_string(fname).unwrap_or_default();
        let entries = data.par_lines()
            .filter(|e| { !e.starts_with("#") })
            .map(|e| { Entry::from_captures( &re.captures(e).unwrap() ) } )
            .collect();
        Self { entries }
    }

    pub fn characters(&self) -> Vec<&Entry> {
        self.entries.iter().filter(|e| e.chr).collect()
    }

    fn characters_filtered(&self, s: &str) -> Vec<&Entry> {
        self.entries.iter().filter(|e| { e.chr && s.contains(e.tra.as_str()) }).collect()
    }

    /// Search all containing
    pub fn search(&self, s: &str) -> Vec<&Entry> {
        self.entries.iter().filter(|e| { e.sim.contains(s) || e.tra.contains(s) }).collect()
    }

    /// Search exact match
    pub fn find(&self, s: &str) -> Vec<&Entry> {
        self.entries.iter().filter(|e| { e.sim.as_str() == s || e.tra.as_str() == s }).collect()
    }

    /// Convert traditional to simplified
    pub fn to_sim(&self, s: &str) -> String {
        let chr_list = self.characters_filtered(s);
        let ss = s.chars()
            .map(|z| {
                let c = chr_list.iter().find(|x| x.tra == z.to_string());
                match c {
                    Some(c) => c.sim.chars().nth(0).unwrap_or(z),
                    None => z,
                }
                
            }).collect();
        ss
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_cedict() {
        let fname = "cedict_1_0_ts_utf-8_mdbg.txt";

        let now = Instant::now();
        let c = Cedict::par_new(fname);
        let elapsed = now.elapsed();
        println!("Test 1: {}ms", elapsed.as_millis());

        let now = Instant::now();
        let c_chars = c.characters().len();
        assert!(c_chars > 1000);
        let elapsed = now.elapsed();
        println!("Test 2: {}ms\tresult: {}", elapsed.as_millis(), c_chars);

        let now = Instant::now();
        let c_search = c.search("以").len();
        assert!(c_search > 0);
        let elapsed = now.elapsed();
        println!("Test 3: {}ms\tresult: {}", elapsed.as_millis(), c_search);

        let tra = "楊武蹬踏在地面上發出低沉的聲音，有力的甩臂，強勁的蹬踏，令楊武的速度很快就加速到他的極限，他面色猙獰，額頭青筋一突一突的，咬着牙瘋狂奔跑着。在從速度測試區域跑過的時候，還發出一聲壓抑的低吼！";
        let sim = "杨武蹬踏在地面上发出低沉的声音，有力的甩臂，强劲的蹬踏，令杨武的速度很快就加速到他的极限，他面色狰狞，额头青筋一突一突的，咬着牙疯狂奔跑着。在从速度测试区域跑过的时候，还发出一声压抑的低吼！";

        let now = Instant::now();
        let conv = c.to_sim(tra);
        assert_eq!(sim, conv.as_str());
        let elapsed = now.elapsed();
        println!("Test 4: {}ms", elapsed.as_millis());

    }
}

