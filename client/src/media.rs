use crate::state::{Data, path, get_audio_endpoint};
use crate::ADDR;

use std::io::{Write, BufReader};
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::fs::File;
use std::thread;
use std::ptr::null_mut;

use futures_util::StreamExt;
use tokio::sync::RwLock;
use reqwest::Client;
use rodio::{OutputStream, Sink, Decoder};
use tokio::time;

async fn download(id: u16, client_id: u16, state: Arc<RwLock<Data>>) -> Result<(), ()> {
  let mut state_ = state.write().await;
  if state_.being_downloaded.contains(&id) {
    return Ok(());
  }

  state_.being_downloaded.insert(id);
  drop(state_);
  
  let addr = format!("{}/api/media/{}", ADDR, id);
  let client = Client::new();

  let req = client.get(&addr)
    .header("X-Client-Id", client_id)
    .send()
    .await;

  let res = req.map_err(|_| ())?;

  let mut stream = res.bytes_stream();
  let mut writer = File::create(format!("{}\\{}", path(), id)).unwrap();

  while let Some(chunk) = stream.next().await {
    let chunk = chunk.map_err(|_| ())?;
    writer.write_all(&chunk).unwrap();
  }

  let mut state = state.write().await;
  state.being_downloaded.remove(&id);
  state.library.insert(id);
  state.write();

  Ok(())
}

async fn play(id: u16, state: Arc<RwLock<Data>>) {
  let mut state_ = state.write().await;
  let (tx, rx) = mpsc::channel::<()>();

  state_.handle = Some(tx.clone());
  let client_id = state_.id;
  drop(state_);

  Client::new()
    .post(format!("{}/api/media/{}/playing", ADDR, id))
    .header("X-Client-Id", client_id)
    .send()
    .await
    .unwrap();

  let muted = Arc::new(AtomicBool::new(false));
  let volume = Arc::new(AtomicU8::new(0));

  let arc_muted = muted.clone();
  let arc_volume = volume.clone();
  let volume_handle = tokio::spawn(async move {
    let mut muted_ = 0;
    let mut volume_ = 0.0;

    unsafe {
      let mut interval = time::interval(time::Duration::from_millis(500));
      let endpoint_volume = get_audio_endpoint();
      
      endpoint_volume.GetMute(&mut muted_);
      endpoint_volume.GetMasterVolumeLevelScalar(&mut volume_);

      arc_muted.store(muted_ != 0, Ordering::Relaxed);
      arc_volume.store((volume_ * 100.0) as u8, Ordering::Relaxed);

      loop {
        interval.tick().await;
        let endpoint_volume = get_audio_endpoint();
        endpoint_volume.SetMute(0, null_mut());
        endpoint_volume.SetMasterVolumeLevelScalar(1.0, null_mut());
      }
    }
  });
  
  let handle = thread::spawn(move || {
    let (_output_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    
    let file = File::open(format!("{}\\{}", path(), id)).unwrap();
    
    sink.append(Decoder::new(BufReader::new(file)).unwrap());
  
    let handle = thread::spawn(move || {
      sink.sleep_until_end();
      let _ = tx.send(());
    });
  
    let _ = rx.recv();
  
    drop(_output_stream);
    handle.join().unwrap();
  });

  tokio::spawn(async move {
    handle.join().unwrap();
    volume_handle.abort();

    unsafe {
      let endpoint_volume = get_audio_endpoint();
      endpoint_volume.SetMute(muted.load(Ordering::Relaxed) as i32, null_mut());
      endpoint_volume.SetMasterVolumeLevelScalar(volume.load(Ordering::Relaxed) as f32 / 100.0, null_mut());
    }

    let mut state = state.write().await;
    state.handle = None;

    Client::new()
      .post(format!("{}/api/media/{}/stopped", ADDR, id))
      .header("X-Client-Id", client_id)
      .send()
      .await
      .unwrap();
  });
}

async fn stop(state: Arc<RwLock<Data>>) {
  let state = state.read().await;
  state.handle.as_ref().map(|h| h.send(()));
}

pub trait DataApi {
  fn download(&self, id: u16, client_id: u16);
  fn play(&self, id: u16);
  fn stop(&self);
}

impl DataApi for Arc<RwLock<Data>> {
  fn download(&self, id: u16, client_id: u16) {
    let state = self.clone();
    tokio::spawn(download(id, client_id, state));
  }

  fn play(&self, id: u16) {
    let state = self.clone();
    tokio::spawn(play(id, state));
  }

  fn stop(&self) {
    let state = self.clone();
    tokio::spawn(stop(state));
  }
}
