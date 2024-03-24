use crate::stream::create_client;

use std::collections::HashSet;
use std::sync::OnceLock;
use std::{env, fs};
use std::ptr::null_mut;
use std::sync::mpsc;

use serde::{Serialize, Deserialize};
use tokio::time::{sleep, Duration};
use bincode::{serialize, deserialize};

use winapi::shared::winerror::S_OK;
use winapi::shared::minwindef::LPVOID;

use winapi::um::combaseapi::CoUninitialize;
use winapi::um::objbase::CoInitialize;
use winapi::um::mmdeviceapi::{IMMDevice, eRender, eConsole, CLSID_MMDeviceEnumerator, IMMDeviceEnumerator};
use winapi::um::endpointvolume::IAudioEndpointVolume;
use winapi::um::combaseapi::{CLSCTX_ALL, CoCreateInstance};
use winapi::Interface;

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

pub unsafe fn get_audio_endpoint() -> &'static mut IAudioEndpointVolume {
  static mut ENDPOINT_VOLUME: *mut IAudioEndpointVolume = null_mut();
  if ENDPOINT_VOLUME.is_null() {
    ENDPOINT_VOLUME = init_audio_endpoint().unwrap() as *const _ as *mut _;
  }

  &mut *ENDPOINT_VOLUME
}

unsafe fn init_audio_endpoint<'a>() -> Result<&'a IAudioEndpointVolume, &'static str> {
  CoInitialize(null_mut());

  let mut device_enumerator: *mut IMMDeviceEnumerator = null_mut();
  let hr = CoCreateInstance(
    &CLSID_MMDeviceEnumerator,
    null_mut(),
    CLSCTX_ALL,
    &IMMDeviceEnumerator::uuidof(),
    &mut device_enumerator as *mut _ as *mut LPVOID,
  );

  if hr != S_OK {
    CoUninitialize();
    return Err("Failed to create device enumerator.");
  }

  let device_enumerator = &*device_enumerator;
  let mut default_device: *mut IMMDevice = null_mut();
  let hr = device_enumerator.GetDefaultAudioEndpoint(eRender, eConsole, &mut default_device);

  if hr != S_OK {
    CoUninitialize();
    return Err("Failed to get default audio endpoint.");
  }

  let default_device = &*default_device;
  let mut endpoint_volume: *mut IAudioEndpointVolume = null_mut();
  let hr = default_device.Activate(
    &IAudioEndpointVolume::uuidof(),
    CLSCTX_ALL,
    null_mut(),
    &mut endpoint_volume as *mut _ as *mut LPVOID,
  );

  if hr != S_OK {
    CoUninitialize();
    return Err("Failed to get endpoint volume.");
  }

  println!("Endpoint volume retrieved successfully.");
  Ok(&*endpoint_volume)
}

#[derive(Serialize, Deserialize)]
pub struct Data {
  pub library: HashSet<u16>,
  pub id: u16,
  #[serde(skip)]
  pub being_downloaded: HashSet<u16>,
  #[serde(skip)]
  pub handle: Option<mpsc::Sender<()>>,
}

impl Data {
  fn new(id: u16) -> Self {
    Self {
      library: HashSet::new(),
      being_downloaded: HashSet::new(),
      id,
      handle: None,
    }
  }

  pub async fn init() -> Self {
    let path = format!("{}\\data.bin", path());
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
    let buff = fs::read(format!("{}\\data.bin", path())).unwrap();
    deserialize(&buff).unwrap()
  }

  pub fn write(&self) {
    let content = serialize(&self).unwrap();
    fs::write(format!("{}\\data.bin", path()), content).unwrap();
  }
}
