#![allow(dead_code)]

use anyhow::Result;
use std::fs;

mod cedict;
mod config;
mod gui;
mod chat;
mod anki;

fn main() -> Result<(), iced::Error> {
    let conf: config::Config =  toml::from_str( fs::read_to_string( "./app.toml" ).unwrap().as_str() ).unwrap();
    gui::run(conf.window.theme.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn read_config() {

        let now = Instant::now();
        let conf: config::Config =  toml::from_str( fs::read_to_string( "./app.toml" ).unwrap().as_str() ).unwrap();
        let elapsed = now.elapsed();
        println!("Test 1: {}ms", elapsed.as_millis());

        assert_eq!(conf.openai_model.as_str(), "gpt-4");
        
    }
}
