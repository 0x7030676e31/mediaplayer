use super::stream::{DashboardPayload, Payload};
use crate::AppState;

use std::collections::HashSet;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::OnceLock;
use std::{env, fs};

use actix_web::web::Bytes;
use futures::future;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use actix_web_lab::sse;
use tokio::time;


const CLEANUP_INTERVAL: Duration = Duration::from_secs(15);

#[derive(Serialize, Deserialize)]
pub struct Media {
  pub id: u16,
  pub name: String,
  pub downloaded: HashSet<u16>,
}

impl Media {
  pub fn new(id: u16, name: String) -> Self {
    Self {
      id,
      name,
      downloaded: HashSet::new(),
    }
  }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "activity", content = "timestamp")]
pub enum Activity {
  Online,
  Offline(u64),
}

impl Activity {
  pub fn offline() -> Self {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    Self::Offline(now)
  }
}

#[derive(Serialize, Deserialize)]
pub struct Client {
  pub id: u16,
  pub ip: String,
  pub hostname: String,
  pub activity: Activity,
}

impl Client {
  pub fn new(id: u16, ip: String, hostname: String) -> Self {
    Self {
      id,
      ip,
      hostname,
      activity: Activity::Online,
    }
  }
}

fn path() -> &'static str {
  static PATH: OnceLock<String> = OnceLock::new();
  PATH.get_or_init(|| {
    let is_production = env::var("PRODUCTION").map_or(false, |prod| prod == "true");
    if is_production {
      return String::from("/root/");
    }

    if env::consts::OS != "linux" {
      panic!("Unsupported OS");
    }

    let path = env::var("HOME").unwrap() + "/.config/mediaplayer";
    if !fs::metadata(&path).is_ok() {
      fs::create_dir_all(&path).unwrap();
    }

    path
  })
}

pub trait CleanupLoop {
  fn start_cleanup_loop(&self);
}

impl CleanupLoop for AppState {
  fn start_cleanup_loop(&self) {
    let state = self.clone();
    tokio::spawn(async move {
      let mut interval = time::interval(CLEANUP_INTERVAL);
      
      loop {
        interval.tick().await;
      
        let mut state = state.write().await;
        let mut clients = HashSet::new();

        state.streams.retain(|(tx, id)| {
          let is_closed = tx.is_closed();
          if is_closed {
            clients.insert(*id);
          }

          !is_closed
        });

        clients.retain(|id| {
          if state.streams.iter().any(|(_, client_id)| client_id == id) {
            return false;
          }

          match state.clients.iter_mut().find(|client| client.id == *id) {
            Some(client) => {
              client.activity = Activity::offline();
              true
            },
            None => false,
          }
        });

        let payload = DashboardPayload::ClientDisconnected(&clients);
        state.broadcast_to_dashboard(payload).await;
      }
    });
  }
}

#[derive(Serialize, Deserialize, Default)]
pub struct State {
  pub library: Vec<Media>,
  pub clients: Vec<Client>,
  next_id: u16,

  #[serde(skip)]
  pub streams: Vec<(mpsc::Sender<Bytes>, u16)>,
  #[serde(skip)]
  pub dashboard_streams: Vec<mpsc::Sender<sse::Event>>,
}

impl State {
  pub fn new() -> Self {
    let path = format!("{}/state.json", path());
    match fs::read_to_string(&path) {
      Ok(data) => match serde_json::from_str(&data) {
        Ok(state) => state,
        Err(err) => panic!("Failed to parse state: {}\nMaybe the file is corrupted?", err),
      },
      Err(_) => Self::default(),
    }
  }

  // This method does not require saving the state to disk
  // because it is used only when a new resource is created
  // so the state will be saved after the resource is created anyway
  pub fn next_id(&mut self) -> u16 {
    self.next_id += 1;
    self.next_id
  }

  pub fn write(&self) {
    let path = format!("{}/state.json", path());
    let data = serde_json::to_string(self).unwrap();

    fs::write(path, data).unwrap();
    log::debug!("State written to disk");
  }

  pub fn new_client(&mut self, hostname: String, ip: String) -> u16 {
    let id = self.next_id();
    let client = Client::new(id, ip, hostname);
    self.clients.push(client);
    self.write();
    id
  }

  pub fn get_audio_writer(&mut self, name: String) -> (u16, fs::File) {
    let id = self.next_id();
    let media = Media::new(id, name);

    self.library.push(media);
    self.write();

    let dir = format!("{}/media", path());
    if !fs::metadata(&dir).is_ok() {
      fs::create_dir_all(&dir).unwrap();
    }

    let writer = fs::File::create(format!("{}/{}", dir, id)).unwrap();
    (id, writer)
  }

  pub fn get_audio_reader(&self, id: u16) -> Option<fs::File> {
    let path = format!("{}/media/{}", path(), id);
    match fs::File::open(&path) {
      Ok(file) => Some(file),
      Err(_) => None,
    }
  }

  pub async fn broadcast(&self, payload: Payload) {
    let payload = payload.into_bytes();
    let futures = self.streams.iter().map(|(tx, _)| tx.send(payload.clone()));
    future::join_all(futures).await;
  }

  pub async fn broadcast_to(&self, id: u16, payload: Payload) {
    let payload = payload.into_bytes();
    let futures = self.streams.iter().filter_map(|(tx, client_id)| {
      if client_id == &id { Some(tx.send(payload.clone())) } else { None }
    });

    future::join_all(futures).await;
  }

  pub async fn broadcast_to_dashboard<'a>(&self, payload: DashboardPayload<'a>) {
    let payload = payload.into_event();
    let futures = self.dashboard_streams.iter().map(|tx| tx.send(payload.clone()));
    
    future::join_all(futures).await;
  }
}
