// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use serde::{Serialize, Deserialize};
use tauri::{CustomMenuItem, Menu, Submenu, State};
use std::fmt::Formatter;
use std::{thread, option, fmt::Debug};
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

fn thread_80211(rx: Receiver<ChannelData>) {
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
                                                        eprintln!("error while tryint to open capture: {:?}", err);
                                                    }
                                                }
                                            } else {
                                                eprintln!("cant create capture obj");
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
                    println!("sending packet");
                    if let Err(err) = capture.sendpacket(data) {
                        //add err support
                        eprintln!("error while sending packet: {:?}", err);
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
    println!("set_device");
    state.send(ChannelData::UseDevice(device));
}

fn main() {

    let (tx, rx) = unbounded();
    
    thread::spawn(move || {
        thread_80211(rx);
    });

    let settings = CustomMenuItem::new("settings".to_string(), "Settings");
    let file_submenu = Submenu::new("File", Menu::new().add_item(settings));
    let menu = Menu::new()
        .add_submenu(file_submenu);

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![set_controlls, get_devices, set_device])
        .menu(menu)
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
