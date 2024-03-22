use crate::stream::create_client;

use std::collections::HashSet;
use std::sync::OnceLock;
use std::{env, fs};

use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

pub fn path() -> &'static str {
  static PATH: OnceLock<String> = OnceLock::new();
  PATH.get_or_init(|| {
    let path = format!("{}\\.mediaplayer", env::var("LOCALAPPDATA").unwrap());

    println!("Working directory: {}", path);
    if fs::metadata(&path).is_err() {
      match fs::create_dir(&path) {
        Ok(_) => println!("Created directory"),
        Err(e) => eprintln!("Error creating directory: {}", e),
      }
    }

    path
  })
}

#[derive(Serialize, Deserialize)]
pub struct Data {
  pub library: HashSet<u16>,
  #[serde(skip)]
  pub being_downloaded: HashSet<u16>,
  pub id: u16,
}

impl Data {
  fn new(id: u16) -> Self {
    Self {
      library: HashSet::new(),
      being_downloaded: HashSet::new(),
      id,
    }
  }

  pub async fn init() -> Self {
    let path = format!("{}\\data.json", path());
    if fs::metadata(&path).is_ok() {
      println!("State file exists, reading...");
      return Self::read();
    }

    let id = loop {
      match create_client().await {
        Ok(id) => break id,
        Err(_) => sleep(Duration::from_secs(5)).await,
      }
    };

    let data = Self::new(id);
    data.write();

    data
  }

  fn read() -> Self {
    let content = fs::read_to_string(format!("{}\\data.json", path())).unwrap();
    println!("Initiating data from file: {}", format!("{}\\data.json", path()));
    
    serde_json::from_str(&content).unwrap()
  }

  fn write(&self) {
    let content = serde_json::to_string(&self).unwrap();
    println!("Writing data to file: {}", format!("{}\\data.json", path()));

    fs::write(format!("{}\\data.json", path()), content).unwrap();
  }
}
