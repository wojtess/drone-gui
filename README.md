This application requires elevated privileges (root/sudo) to run due to the executable binary needing the `CAP_NET_RAW=ep` capability.  This is handled automatically within the modified `tauri` script in `package.json`:

```json
"tauri": "cargo build --manifest-path=src-tauri/Cargo.toml && sudo setcap 'CAP_NET_RAW=ep' src-tauri/target/debug/drone-gui && tauri"
```

Use the following commands to run the application:

* **Development:** `npm run tauri dev`
* **Build:** `npm run tauri build`

## Network Interface Configuration

# **WARNING: Interface Selection Required** #

> **Immediately after starting the application, you _MUST_ go to the settings and select your network interface.**  The application will technically launch without this crucial step, but its core functionality will be completely disabled. While packets *will* be transmitted, they will be silently discarded if the interface remains unconfigured.  **This is not optional!**

The selected interface **must** be in monitor mode.  See this [guide](https://www.geeksforgeeks.org/how-to-put-wifi-interface-into-monitor-mode-in-linux/) for instructions on enabling monitor mode.

### Quick Commands for Monitor Mode (Linux)

Replace `<interface>` with your network interface name (e.g., wlp5s0). Replace `<channel>` with the desired Wi-Fi channel (default for ESP is 11).

```bash
sudo iw dev
sudo ip link set <interface> down
sudo iw <interface> set monitor none
sudo ip link set <interface> up
sudo iwconfig <interface> mode monitor channel <channel>
```

### RFKILL Troubleshooting

If you encounter issues, check for RFKILL blocks:

```bash
rfkill list all
```

* **Soft Block:**  Unblock with: `sudo rfkill unblock all`
* **Hard Block:** Check for a physical switch on your wireless card.


### Other Useful Commands

* Check interface status: `cat /sys/class/net/<interface>/operstate`
* Show interface details: `ip link show`