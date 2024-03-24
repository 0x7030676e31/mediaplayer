use super::stream::{DashboardPayload, HintedSender, Payload};
use crate::AppState;

use std::collections::HashSet;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::OnceLock;
use std::{env, fs};

use actix_web::web::Bytes;
use futures::future;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::time;
use actix_web_lab::sse;
use lofty::{Probe, AudioFile};

const CLEANUP_INTERVAL: Duration = Duration::from_secs(15);

#[derive(Serialize, Deserialize)]
pub struct Media {
  pub id: u16,
  pub name: String,
  pub downloaded: HashSet<u16>,
  pub length: u64,
}

impl Media {
  pub fn new(id: u16, name: String) -> Self {
    Self {
      id,
      name,
      downloaded: HashSet::new(),
      length: 0,
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
  pub username: String,
  pub activity: Activity,
  #[serde(skip)]
  pub playing: Option<(u16, tokio::task::JoinHandle<()>)>,
}

#[derive(Deserialize)]
pub struct ClientInfo {
  hostname: String,
  username: String,
}

impl Client {
  pub fn new(id: u16, ip: String, hostname: String, username: String) -> Self {
    Self {
      id,
      ip,
      hostname,
      username,
      activity: Activity::Online,
      playing: None,
    }
  }
}

impl ClientInfo {
  pub fn into_client(self, id: u16, ip: String) -> Client {
    Client::new(id, ip, self.hostname, self.username)
  }
}

pub fn path() -> &'static str {
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
      log::info!("Cleanup loop started");

      loop {
        interval.tick().await;
      
        let mut state = state.write().await;
        let mut clients = HashSet::new();

        let len = state.streams.len();

        let payload = Payload::Ping.into_bytes();
        let futures = state.streams.iter().map(|(tx, _)| {
          tx.send_hinted(payload.clone())
        });
        
        let mut futures = future::join_all(futures).await.into_iter();
        state.streams.retain(|(tx, id)| {
          let is_closed = tx.is_closed() || futures.next().unwrap().is_err();
          if is_closed {
            log::info!("Client {} disconnected", id);
            clients.insert(*id);
          }

          !is_closed
        });

        let diff = len - state.streams.len();
        if diff > 0 {
          log::debug!("{} clients disconnected", diff);
        }

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

        if !clients.is_empty() {
          let payload = DashboardPayload::ClientDisconnected(&clients);
          state.broadcast_to_dashboard(payload).await;
          state.write();
        }
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
  #[serde(skip)]
  ack: u64,
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

  pub async fn new_client(&mut self, client_info: ClientInfo, ip: String) -> u16 {
    let id = self.next_id();
    let client = client_info.into_client(id, ip);
    let payload = DashboardPayload::ClientCreated(&client);
    self.broadcast_to_dashboard(payload).await;
    
    self.clients.push(client);
    self.write();
    id
  }

  // This method won't save the state to disk
  // Because the `set_audio_length` method will be called
  // right after this method is called
  pub fn get_audio_writer(&mut self, name: String) -> (u16, fs::File) {
    let id = self.next_id();
    let media = Media::new(id, name);

    self.library.push(media);

    let dir = format!("{}/media", path());
    if !fs::metadata(&dir).is_ok() {
      fs::create_dir_all(&dir).unwrap();
    }

    let writer = fs::File::create(format!("{}/{}", dir, id)).unwrap();
    (id, writer)
  }

  pub fn set_audio_length(&mut self, id: u16) -> u64 {
    let path = format!("{}/media/{}", path(), id);
    let file = Probe::open(&path)
      .unwrap()
      .guess_file_type()
      .unwrap()
      .read()
      .unwrap();

    let length = file.properties().duration().as_millis() as u64;
    let media = self.library.iter_mut().find(|media| media.id == id).unwrap();
    media.length = length;

    self.write();
    length
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
    let futures = self.streams.iter().map(|(tx, _)| tx.send_hinted(payload.clone()));
    future::join_all(futures).await;
  }

  pub async fn broadcast_to(&self, id: u16, payload: Payload) {
    let payload = payload.into_bytes();
    let futures = self.streams.iter().filter_map(|(tx, client_id)| {
      if client_id == &id { Some(tx.send_hinted(payload.clone())) } else { None }
    });

    future::join_all(futures).await;
  }

  pub fn next_ack(&mut self) -> u64 {
    self.ack += 1;
    self.ack
  }

  pub async fn broadcast_to_dashboard<'a>(&mut self, payload: DashboardPayload<'a>) {
    let payload = payload.into_event(self.next_ack(), None);
    let futures = self.dashboard_streams.iter().map(|tx| tx.send(payload.clone()));
    
    future::join_all(futures).await;
  }


  pub async fn broadcast_to_dashboard_with_nonce<'a>(&mut self, payload: DashboardPayload<'a>, nonce: u64) {
    let payload = payload.into_event(self.next_ack(), Some(nonce));
    let futures = self.dashboard_streams.iter().map(|tx| tx.send(payload.clone()));
    
    future::join_all(futures).await;
  }
}
