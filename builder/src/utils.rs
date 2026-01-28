use std::sync::Mutex;
use regex::Regex;

static KEY: Mutex<Vec<u8>> = Mutex::new(Vec::new());

pub fn set_key(new_key: &[u8]) {
    let mut key = KEY
        .lock()
        .unwrap();

    *key = new_key
        .to_vec();
}

pub fn encrypt(data: &str) -> Vec<u8> {
    let key = KEY
        .lock()
        .unwrap();

    lib::crypto::encrypt(data.as_bytes(), &key)
        .unwrap()
}

pub fn edit_array(data: &mut String, name: &str, arr: &[u8]) {
    let pattern = format!(
        "(pub\\s+const\\s+{}\\s*:\\s*&\\[\\s*u8\\s*\\]\\s*=\\s*)&\\[[^\\]]*\\]\\s*;",
        regex::escape(name)
    );

    let regex = Regex::new(&pattern)
        .unwrap();

    let new_val = arr
        .iter()
        .map(|num| num.to_string())
        .collect::<Vec<String>>()
        .join(", ");

    let new_data = format!("$1&[{}];", new_val);

    *data = regex
        .replace(data, new_data)
        .to_string();
}

pub fn replace_str(data: &mut String, old: &str, new: &str) {
    let pattern = format!(
        "<BUILDER_{}>",
        regex::escape(old)
    );

    let regex = Regex::new(&pattern)
        .unwrap();

    *data = regex
        .replace_all(data, new)
        .to_string();
}