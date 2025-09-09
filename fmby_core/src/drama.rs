use rand::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct DramaConfig {
    pub people: Vec<String>,
    pub servers: Vec<String>,
    pub bsoftware: Vec<String>,
    pub phrases: Vec<String>,
    pub replacers: HashMap<String, String>,
}

impl DramaConfig {
    pub fn from_config() -> Self {
        serde_json::from_str(include_str!("../../resources/drama.json")).unwrap()
    }

    fn get(&self, key: &str) -> Option<&Vec<String>> {
        match key {
            "people" => Some(&self.people),
            "servers" => Some(&self.servers),
            "bsoftware" => Some(&self.bsoftware),
            _ => None,
        }
    }
}

pub fn fill_phrase(phrase: &str, data: &DramaConfig, rng: &mut impl Rng) -> String {
    phrase
        .split_whitespace()
        .map(|word| {
            data.replacers
                .iter()
                .find_map(|(prefix, target)| {
                    if word.starts_with(prefix) {
                        data.get(target)?
                            .choose(rng)
                            .map(|choice| word.replacen(prefix, choice, 1))
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| word.to_string())
        })
        .collect::<Vec<_>>()
        .join(" ")
}
