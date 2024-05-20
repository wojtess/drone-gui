// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use serde::{Serialize, Deserialize};
use tauri::{CustomMenuItem, Menu, Submenu, State, Manager, Window};
use core::time;
use std::fmt::Formatter;
use std::sync::Mutex;
use std::{thread, option, fmt::Debug, sync::Arc};
use std::time::Duration;
use crossbeam_channel::{select, unbounded, tick, Sender, Receiver};


#[derive(Default)]
struct ControllsData {
    throttle: f64,
    pitch: f64,
    roll: f64,
    yaw: f64,
    capture: Option<pcap::Capture<pcap::Active>>,
}

impl Debug for ControllsData {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("ControllsData")
         .field("throttle", &self.throttle)
         .field("pitch", &self.pitch)
         .field("roll", &self.roll)
         .field("yaw", &self.yaw)
         .finish()
    }
}
trait InPacket {
    fn decode(&self);
}

trait OutPacket {
    fn encode(&self);
}

struct PacketIn0x01 {
    time: i32,
    throttle: u64,
    pitch: u64,
    roll: u64,
    yaw: u64,
}
impl InPacket for PacketIn0x01 {
    fn decode(&self) {
        todo!()
    }
}

fn thread_recieve(state: Arc<Mutex<ControllsData>>) {
    let timer = tick(Duration::from_millis(10));
    loop {
        select! {
            recv(timer) -> _ => {
                let mut state = state.lock().unwrap();
                if let Some(ref mut capture) = state.capture {
                    while let Ok(packet) = capture.next_packet() {

                    }
                }
            }
        }
    }

}

fn thread_80211(rx: Receiver<ChannelData>, app_hanlder: tauri::AppHandle, state: Arc<Mutex<ControllsData>>) {
    let timer = tick(Duration::from_millis(10));
    loop {
        select! {
            recv(rx) -> input => {
                if let Ok(data) = input {
                    match data {
                        ChannelData::Controlls(t, p, r, y) => {
                            let mut state = state.lock().unwrap();
                            state.pitch = p;
                            state.throttle = t;
                            state.roll = r;
                            state.yaw = y;
                        },
                        ChannelData::UseDevice(device) => {
                            match pcap::Device::list() {
                                Ok(devices) => {
                                    for d in devices {
                                        if d.name == device.name {
                                            if let Ok(capture) = pcap::Capture::from_device(d.clone()) {
                                                match capture.open() {
                                                    Ok(opened_capture) => {
                                                        println!("setting capture");
                                                        let mut state = state.lock().unwrap();
                                                        state.capture = Some(opened_capture);
                                                    },
                                                    Err(err) => {
                                                        println!("err setting capture, {:?}", err)
                                                        // send_msg(("error while tryint to open capture: {:?}", err).to_string()).expect("Failed to call send_variable");
                                                    }
                                                }
                                            } else {
                                                // send_msg("cant create capture obj".to_string()).expect("Failed to call send_variable");
                                                //add error support
                                            }
                                            break;
                                        }
                                    }
                                },
                                Err(_err) => {
                                    //add err support
                                }
                            }
                        },
                    }
                }
            },
            recv(timer) -> _ => {
                let mut state0 = state.lock().unwrap();
                //WAZNE USTAWIC W USTAWIENIACH INTERFACE INACZEJ NIE ZADZIALA
                let mut data = vec![
                    //radiotap header
                    0x00, // <-- radiotap version
                    0x00, // <-- padding
                    0x0b, 0x00, // <- radiotap header length
                    0x04, 0x0c, 0x00, 0x00, // <-- bitmap
                    0x6c, // <-- rate
                    0x0c, //<-- tx power
                    0x01, //<-- antenna
                    //ieee80211 header
                    0x48, 0x00, 0x00, 0x00,
                    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                    0x13, 0x22, 0x33, 0x44, 0x55, 0x66,
                    0x13, 0x22, 0x33, 0x44, 0x55, 0x66,
                    0x00, 0x00,
                    //magic number
                    0x3C, 0x4A,
                    //finaly some data
                    0x01,//packet id
                ];
                {
                    data.extend_from_slice(&state0.throttle.to_be_bytes());
                    data.extend_from_slice(&state0.pitch.to_be_bytes());
                    data.extend_from_slice(&state0.roll.to_be_bytes());
                    data.extend_from_slice(&state0.yaw.to_be_bytes());
                }
                if let Some(ref mut capture) = state0.capture {
                    // data.extend_from_slice(&((state.throttle*1000.0)as i64).to_be_bytes());
                    // data.extend_from_slice(&((state.pitch*1000.0)as i64).to_be_bytes());
                    // data.extend_from_slice(&((state.roll*1000.0)as i64).to_be_bytes());
                    // data.extend_from_slice(&((state.yaw*1000.0)as i64).to_be_bytes());
                    // send_msg("sending packet".to_string()).expect("Failed to call send_variable");
                    if let Err(err) = capture.sendpacket(data) {
                        //add err support
                        // send_msg(("error while sending packet: {:?}", err).to_string()).expect("Failed to call send_variable");
                    }
                }
            }
        }
    }
}

enum ChannelData {
    Controlls(f64, f64, f64, f64),
    UseDevice(Device),
}

#[tauri::command]
fn set_controlls(state: State<Sender<ChannelData>>, throttle: f64, pitch: f64, roll: f64, yaw: f64) {
    let controlls = ChannelData::Controlls(
        throttle,
        pitch,
        roll,
        yaw
    );
    state.send(controlls).unwrap();
}

#[derive(Serialize, Deserialize)]
struct Device {
    name: String,
    desc: String
}

#[tauri::command]
fn get_devices() -> Vec<Device> {
    let mut out = vec![];
    if let Ok(devices) = pcap::Device::list() {
        for device in devices {
            let mut desc = "".to_string();
            if let Some(d) = device.desc {
                desc = d;
            }
            out.push(Device{
                name: device.name,
                desc: desc,
            })
        }
    }
    return out;
}

#[tauri::command]
fn set_device(state: State<Sender<ChannelData>>, device: Device) {
    //send_msg(state, "set_device".to_string());
    state.send(ChannelData::UseDevice(device));
}

#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
}

fn main() {
    
    let (tx, rx) = unbounded();

    
    let state = Arc::new(Mutex::new(ControllsData::default()));
    let state0 = Arc::clone(&state);
    let state1 = Arc::clone(&state);

    let settings = CustomMenuItem::new("settings".to_string(), "Settings");
    let file_submenu = Submenu::new("File", Menu::new().add_item(settings));
    let menu = Menu::new()
        .add_submenu(file_submenu);

    tauri::async_runtime::spawn(async move {
        thread_recieve(state1);
    });

    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![set_controlls, get_devices, set_device])
        .menu(menu)
        .setup(|app| {
            let app_hanlder=app.handle();
            tauri::async_runtime::spawn(async move {
                thread_80211(rx, app_hanlder, state0);
            });
            Ok(())
        })
        .on_menu_event(|event| {
            match event.menu_item_id() {
                "settings" => {
                    let _ = event.window().emit("openSettings", ());
                }
                _ => {}
            }
        })
        .manage(tx)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
