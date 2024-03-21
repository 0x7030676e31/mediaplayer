use crate::stream::create_client;

use std::collections::HashSet;
use std::sync::OnceLock;
use std::{env, fs};

use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

fn path() -> &'static str {
  static PATH: OnceLock<String> = OnceLock::new();
  PATH.get_or_init(|| {
    let local_app_data = env::var("LOCALAPPDATA").unwrap();
    let path = format!("{}/.mediaplayer", local_app_data);
    
    if !fs::metadata(&path).is_ok() {
      fs::create_dir_all(&path).unwrap();
    }

    path
  })
}

#[derive(Serialize, Deserialize)]
pub struct Data {
  pub library: HashSet<u16>,
  pub id: u16,
}

impl Data {
  fn new(id: u16) -> Self {
    Self {
      library: HashSet::new(),
      id,
    }
  }

  pub async fn init() -> Self {
    let path = format!("{}/data.json", path());
    if fs::metadata(&path).is_ok() {
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
    let content = fs::read_to_string(format!("{}/data.json", path())).unwrap();
    serde_json::from_str(&content).unwrap()
  }

  fn write(&self) {
    let content = serde_json::to_string(&self).unwrap();
    fs::write(format!("{}/data.json", path()), content).unwrap();
  }
}
