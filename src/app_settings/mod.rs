use std::{path, io::{self, Write}, fs};

use serde::{Deserialize, Serialize};
use magic_crypt::{new_magic_crypt, MagicCryptTrait};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AppSettings {
    pub binance_key: String,
    pub binance_secret_key: String,
}

pub fn load_or_create_settings() -> AppSettings {
    let mut pin = String::new();
    println!("Введите пин, для кодирования ключей");
    io::stdin().read_line(&mut pin).unwrap();
    pin = pin.trim().to_string();

    let mc = new_magic_crypt!(pin, 256);
    let path = path::Path::new(".//settings.json");

    let mut settings = AppSettings {
        binance_key: String::from(""),
        binance_secret_key: String::from("")
    };

    if !path.exists() {
        println!("Введите публичный ключ");
        let mut binance_key = String::new();
        io::stdin().read_line(&mut binance_key).unwrap();
        binance_key = binance_key.trim().to_string();
        let binance_key_encrypted = mc.encrypt_str_to_base64(&binance_key);

        println!("Введите секретный ключ");
        let mut binance_secret_key = String::new();
        io::stdin().read_line(&mut binance_secret_key).unwrap();
        binance_secret_key = binance_secret_key.trim().to_string();
        let binance_secret_key_encrypted = mc.encrypt_str_to_base64(&binance_secret_key);

        settings.binance_key = binance_key;
        settings.binance_secret_key = binance_secret_key;

        let settings_encrypted = AppSettings {
            binance_key: binance_key_encrypted,
            binance_secret_key: binance_secret_key_encrypted,
        };
        let settings_str = serde_json::to_string_pretty(&settings_encrypted).unwrap();
        let mut f = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .unwrap()
        ;
        let _r = f.write_all(settings_str.as_bytes());
    } else {
        let settings_str = fs::read_to_string(path).unwrap();
        settings = serde_json::from_str(&settings_str).unwrap();
        let binance_key_encrypted = settings.binance_key.to_string();
        let binance_secret_key_encrypted = settings.binance_secret_key.to_string();

        settings.binance_key = match mc.decrypt_base64_to_string(&binance_key_encrypted) {
            Ok(binance_key) => binance_key,
            Err(_) => {
                println!("!!! Введён неверный ПИН !!!");
                println!("Нажмите ENTER, чтобы выйти");
                let mut ui = String::new();
                let _r = io::stdin().read_line(&mut ui).unwrap();
                std::process::exit(1);
            }
        };
        settings.binance_secret_key = mc.decrypt_base64_to_string(&binance_secret_key_encrypted).unwrap();
    }

    settings
}