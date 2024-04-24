// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use serde::{Serialize, Deserialize};
use tauri::{CustomMenuItem, Menu, Submenu, State, Manager, Window};
use std::fmt::Formatter;
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

fn thread_80211(rx: Receiver<ChannelData>, ) {
    let mut state = ControllsData::default();
    let timer = tick(Duration::from_millis(10));
    loop {
        select! {
            recv(rx) -> input => {
                if let Ok(data) = input {
                    match data {
                        ChannelData::Controlls(t, p, r, y) => {
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
                                                        state.capture = Some(opened_capture);
                                                    },
                                                    Err(err) => {
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
                if let Some(ref mut capture) = state.capture {
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
                    data.extend_from_slice(&state.throttle.to_be_bytes());
                    data.extend_from_slice(&state.pitch.to_be_bytes());
                    data.extend_from_slice(&state.roll.to_be_bytes());
                    data.extend_from_slice(&state.yaw.to_be_bytes());
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
    send_msg(state, "set_device".to_string());
    state.send(ChannelData::UseDevice(device));
}

#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
}

#[tauri::command]
fn send_msg(state: State<Sender<ChannelData>>, variable: String) {
    // event.emit("variable_sent", Some(variable))
}

fn main() {
    
    let (tx, rx) = unbounded();

    thread::spawn(move || {
        thread_80211(rx, a);
    });
    

    let settings = CustomMenuItem::new("settings".to_string(), "Settings");
    let file_submenu = Submenu::new("File", Menu::new().add_item(settings));
    let menu = Menu::new()
        .add_submenu(file_submenu);

    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![set_controlls, get_devices, set_device, send_msg])
        .menu(menu)
        .on_menu_event(|event| {
            match event.menu_item_id() {
                "settings" => {
                    let _ = event.window().emit("openSettings", ());
                }
                _ => {}
            }
        })
        .setup(|app| {
            
        })
        .manage(tx)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
