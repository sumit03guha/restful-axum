use dotenvy::dotenv;
use once_cell::sync::Lazy;
use std::env;

pub fn load_dotenv() {
    dotenv().ok();
}

pub static SECRET_KEY: Lazy<String> =
    Lazy::new(|| env::var("SECRET_KEY").expect("SECRET_KEY env not set."));

pub static HOST: Lazy<String> = Lazy::new(|| env::var("HOST").expect("HOST env not set."));

pub static PORT: Lazy<String> = Lazy::new(|| env::var("PORT").expect("PORT env not set."));

pub static MONGO_URI: Lazy<String> =
    Lazy::new(|| env::var("MONGO_URI").expect("MONGO_URI env not set."));
