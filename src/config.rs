use dotenvy::dotenv;
use once_cell::sync::Lazy;
use std::env;

pub fn load_dotenv() {
    dotenv().ok();
}

pub static SECRET_KEY: Lazy<String> =
    Lazy::new(|| env::var("SECRET_KEY").expect("SECRET_KEY env not set."));
