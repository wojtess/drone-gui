// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use deku::{DekuContainerRead, DekuContainerWrite, DekuRead, DekuUpdate, DekuWrite};
use serde::{Serialize, Deserialize};
use tauri::{CustomMenuItem, Manager, Menu, State, Submenu};
use std::fmt::Formatter;
use std::sync::Mutex;
use std::{fmt::Debug, sync::Arc};
use std::time::Duration;
use crossbeam_channel::{select, unbounded, tick, Sender, Receiver};


#[derive(Default)]
struct ControllsData {
    throttle: f64,
    pitch: f64,
    roll: f64,
    yaw: f64,
    rx_capture: Option<pcap::Capture<pcap::Active>>,
    tx_capture: Option<pcap::Capture<pcap::Active>>,
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

trait PacketOut {
    fn encode(&self) -> Vec<u8>;

    fn id(&self) -> u8;
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "little")]
struct PacketIn0x01 {
    time: i32,
    throttle: u64,
    pitch: u64,
    roll: u64,
    yaw: u64,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "little")]
struct PacketOut0x01 {
    throttle: u64,
    pitch: u64,
    roll: u64,
    yaw: u64,
}

impl PacketOut for PacketOut0x01 {
    fn encode(&self) -> Vec<u8> {
        self.to_bytes().unwrap()
    }

    fn id(&self) -> u8 {
        0x01
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "little")]
struct PacketOut0x02 {
    frequency: u32,
    packet_3: bool,
}

impl PacketOut for PacketOut0x02 {
    fn encode(&self) -> Vec<u8> {
        self.to_bytes().unwrap()
    }

    fn id(&self) -> u8 {
        0x02
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "little")]
struct PacketOut0x03 {
    throttle: u32,
    pitch: u32,
    roll: u32,
    yaw: u32,
}

impl PacketOut for PacketOut0x03 {
    fn encode(&self) -> Vec<u8> {
        self.to_bytes().unwrap()
    }

    fn id(&self) -> u8 {
        0x03
    }
}

fn add_ieee_header(packet: Box<dyn PacketOut>) -> Vec<u8> {
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
        0x3C, 0x4A,//header
        //finaly some data
    ];
    data.extend_from_slice(&[packet.id()]);//header
    data.extend_from_slice(&*packet.encode());
    println!("len: {}", (*packet.encode()).len());
    data
}

fn thread_recieve(state: Arc<Mutex<ControllsData>>, app_handle: tauri::AppHandle) {
    let timer = tick(Duration::from_millis(500));
    loop {
        select! {
            recv(timer) -> _ => {
                // println!("[RECV]BEFORE LOCK");
                {
                    app_handle.emit_all("error", "example_error").unwrap();
                    let mut maybe_capture = {
                        let mut state = state.lock().unwrap();
                        state.rx_capture.take()
                    };
                    if let Some(ref mut capture) = maybe_capture {
                        while let Ok(packet) = capture.next_packet() {
                            let mut len = packet.header.caplen;
                            let mut buf = packet.data.to_vec();
                            if buf[0] != 0x48 {
                                continue;
                            }
                            len -= 24;
                            buf.drain(0..24);
                            if len < 2/* maginc number */ + 1 {
                                continue;
                            }
                            if buf[1] == 0x3C && buf[2] == 0x4A {
                                buf.drain(0..2);
                                len -= 2;//TODO: check if there shuld be 3
                                match buf[3] {
                                    1 => {
                                        let _ = PacketIn0x01::from_bytes((&buf[..], len as usize));
                                    },
                                    _ => {/* wrong id */},
                                }
                            }
                        }
                    }
                    if let Some(capture) = maybe_capture {
                        let mut state = state.lock().unwrap();
                        state.tx_capture = Some(capture);
                    }
                }
                // println!("[RECV]AFTER LOCK");
            }
        }
    }

}

fn thread_80211(rx: Receiver<ChannelData>, _: tauri::AppHandle, state: Arc<Mutex<ControllsData>>, packets: Receiver<Box<dyn PacketOut + Send>>) {
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
                                            if let Ok(mut capture_rx) = pcap::Capture::from_device(d.clone()) {
                                                capture_rx = capture_rx.timeout(100);

                                                match capture_rx.open() {
                                                    Ok(opened_capture) => {
                                                        println!("setting rx capture");
                                                        let mut state = state.lock().unwrap();
                                                        state.rx_capture = Some(opened_capture);
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
                                            if let Ok(mut capture_tx) = pcap::Capture::from_device(d.clone()) {
                                                capture_tx = capture_tx.immediate_mode(true);
                                                if let Ok(opened_capture) = capture_tx.open() {
                                                    let mut state = state.lock().unwrap();
                                                    state.tx_capture = Some(opened_capture);
                                                }
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
            recv(packets) -> input => {
                if let Ok(packet) = input {
                    // println!("[SEND] TRYING TO LOCK STATE");
                    {
                        let mut state0 = state.lock().unwrap();
                        // println!("sending packet id {:?}", packet.id());
                        if let Some(tx) = &mut state0.tx_capture {
                            let data = add_ieee_header(packet);
                            if let Err(err) = tx.sendpacket(data) {
                                //TODO: add error
                                println!("err while sending packet {:?}", err)
                            }
                        }
                    }
                    // println!("[SEND] LOCK AFTER");
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
    let _ = state.send(ChannelData::UseDevice(device));
}

#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
}

fn main() {

    let (tx, rx) = unbounded();
    let (tx_packets, rx_packets) = unbounded::<Box<(dyn PacketOut + Send + 'static)>>();


    let state = Arc::new(Mutex::new(ControllsData::default()));
    let state_80211_thread = Arc::clone(&state);
    let state_reciver = Arc::clone(&state);
    let state_tramsiter = Arc::clone(&state);


    let settings = CustomMenuItem::new("settings".to_string(), "Settings");
    let file_submenu = Submenu::new("File", Menu::new().add_item(settings));
    let menu = Menu::new()
        .add_submenu(file_submenu);


    let _app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![set_controlls, get_devices, set_device])
        .menu(menu)
        .setup(|app| {


            {
                let app_hanlder=app.handle();
                tauri::async_runtime::spawn(async move {
                    thread_recieve(state_reciver, app_hanlder);
                });
            }

            {
                let app_hanlder=app.handle();
                tauri::async_runtime::spawn(async move {
                    let timer = tick(Duration::from_millis(50));
                    loop {
                        select! {
                            recv(timer) -> _ => {
                                let state = state_tramsiter.lock().unwrap();
                                let packet = PacketOut0x01 {
                                    throttle: state.throttle.to_bits().to_be(),
                                    pitch: state.pitch.to_bits().to_be(),
                                    roll: state.roll.to_bits().to_be(),
                                    yaw: state.yaw.to_bits().to_be(),
                                };
                                let _ = tx_packets.send(Box::new(packet));
                            }
                        }
                    }
                });
            }
            {
                let app_hanlder=app.handle();
                tauri::async_runtime::spawn(async move {
                    thread_80211(rx, app_hanlder, state_80211_thread, rx_packets);
                });
            }

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
