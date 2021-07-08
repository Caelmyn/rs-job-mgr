use std::fs;

use serde::{Deserialize, Serialize};

/* -- Key definition -- */

#[derive(Deserialize, Serialize, Default, Debug)]
struct Parameter {
    name: String,
    value: String,
}

/* -- Key public implementation -- */

impl Parameter {
    fn new(key: &str, value: &str) -> Self {
        let mut ret = Self {
            name: key.to_string(),
            value: value.to_string(),
        };

        ret.eval_file_content();

        ret
    }

    fn eval_file_content(&mut self) {
        if let Some(filename) = self.value.strip_prefix('@') {
            self.value = fs::read_to_string(filename).unwrap_or_default()
        }
    }
}

/* -- Parameter definition -- */

#[derive(Serialize, Default, Debug)]
pub struct ParameterList {
    #[serde(rename = "parameter")]
    parameters: Vec<Parameter>,
}

/* -- Parameters public implementation -- */

impl ParameterList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_string(s: &str) -> Self {
        let mut parameters: Vec<Parameter> = serde_json::from_str(s).unwrap_or_default();

        parameters
            .iter_mut()
            .for_each(|param| param.eval_file_content());

        Self { parameters }
    }

    pub fn add(&mut self, key: &str, value: &str) {
        self.parameters.push(Parameter::new(key, value))
    }

    pub fn add_list(&mut self, keys: &[(&str, &str)]) {
        keys.iter()
            .map(|(key, val)| Parameter::new(key, val))
            .for_each(|key| self.parameters.push(key));
    }
}
