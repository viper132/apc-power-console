# APC Universal Power Telemetry & Management Console

> A lightweight, modern, hardware-native power management console and graceful automation daemon for **APC / Schneider Electric Uninterruptible Power Supplies (UPS)**.  
> **Universal USB Compatibility:** Works across all USB-enabled APC models (Back-UPS Pro Gaming, Back-UPS Pro, Smart-UPS, Back-UPS BX/CS/RS, and Easy-UPS series).  
> **Ultra-Lightweight & Native:** Built on **Tauri v2 + Rust**, consuming **< 30 MB RAM** (compared to official PowerChute's ~300 MB), **0.0% CPU Idle**, zero Electron/Java bloat, zero Python dependency, Windows 11 & tiling window manager (GlazeWM) friendly.  
> **100% Real Hardware Data:** Direct USB-HID driver communication in Rust via `hidapi` (zero simulations, zero mock metrics).

---

## ⚡ Architectural Comparison

| Dimension | Schneider PowerChute Serial Shutdown (PCSS) | APC Universal Console |
| :--- | :--- | :--- |
| **Runtime Stack** | Bundled OpenJDK JRE 17 + Jetty Web Server + JSP | **Native Rust Core (Tauri v2) + Windows WebView2** |
| **RAM Footprint** | **250 MB – 350 MB RAM** | **~20 MB – 30 MB RAM** (91% reduction) |
| **CPU Overhead** | Background Java GC spikes | **0.0% Idle** (deterministic 1.2s polling) |
| **Distribution** | 250 MB+ installer | **~9.7 MB Standalone `.exe`** (3.3 MB compressed) |
| **IPC Bridge** | Local HTTP Port (vulnerable to port collisions) | **In-Memory Binary IPC (`invoke`)** |
| **System Tray** | Basic browser launcher | **Native Win32 System Tray (Live Tooltip & Context Menu)** |
| **OS Protection** | Full Shutdown only | **Hibernate (`/h`)** (saves RAM/games to SSD) & Full Shutdown |
| **Telemetry Scaling** | Model-specific hardcoded tables | **Dynamic Auto-Discovery** (firmware Report 82 active scaling) |
| **Cloud Dependency** | Nag screens for SmartConnect cloud registration | **100% Local, Private, and Autonomous** |

---

## 🚀 Key Features

### 1. Dual Perspective Views (100% Distinct Architectural Layouts)
* **Gaming Cockpit:**
  * **Morphing 3D Quantum Core:** Hardware-accelerated WebGL 3D (Three.js) geometry that physically morphs according to real-time wattage:
    * *Idle / Low Load (< 250W):* Crystalline Geodesic Dodecahedron (smooth, solid, serene rotation).
    * *Active Gaming (250W – 650W):* Radial Quantum Gimbal Cage (dual-axis counter-rotation with radial bracket expansion).
    * *Peak Stress (> 650W):* Hyper-Stellated Plasma Star (12 sharp energy prongs expanding with reactive micro-jitter).
  * **Wattage-Scaled Kinetic Swarm:** Gyro rings spin up to $6\times$ faster and inward energy particle vortex expands from 50 to 300 particles as PC load increases.
  * **3D In-Scene Holographic HUD:** Floating, high-contrast tabular typography rendered directly inside the WebGL scene with real-time parallax tilt.
* **Standard Enterprise Console:**
  * Clean, tabbed IT management console styled after industrial Schneider EcoStruxure systems (System Overview, Hardware Diagnostics, Shutdown Policies, and Audit Trail).
* **WCAG AAA Color Engine:**
  * Fully supports `🌙 DARK` (Matte Obsidian) and `☀️ LIGHT` (Brushed Silver Alloy with high-contrast cobalt accents).

### 2. Universal Hardware Auto-Discovery
* Automatically scans for any USB power device with Vendor ID `0x051D` (American Power Conversion).
* Dynamically extracts nominal load rating (`max_watts`) directly from firmware **HID Report 82**, automatically scaling gauge parameters whether connected to a 360W, 900W, 1320W, or 2700W unit.
* Dynamic model and serial number retrieval straight from USB device descriptors.

### 3. Graceful Hibernation & Automated Protection Daemon
* **Hibernate vs Shutdown:** Solves the critical gamer/developer outage dilemma by executing `shutdown.exe /h`. Active game sessions, virtual machines, and IDE workspaces are preserved to disk before the battery depletes.
* **Autonomous Evaluator:** Continuously watches real battery percentage ($\le 15\%$), estimated runtime remaining ($\le 5\text{ mins}$), and outage duration ($\ge 10\text{ mins}$).
* **Windows Policy Guard:** Ensures Windows 11 `BATACTIONCRIT` power scheme settings do not trigger premature sleep during momentary USB communication checks.

### 4. Native System Tray, Auto-Start & Configurable Background
* **Native System Tray:** Lives silently in the notification area with live battery, wattage, and status tooltip readouts.
* **Configurable Close-to-Tray:** Toggle whether closing the window minimizes to tray or exits cleanly.
* **Windows Auto-Start:** Integrated with Windows `Run` registry with `--minimized` boot support.

### 5. Real-Time Electricity Cost Engine
* Computes real-time hourly and daily operational energy costs derived from live load wattage against configurable national utility rates (default: PLN Indonesia R-1 Tariff: Rp 1,444.70 / kWh).

---

## 🛠️ USB-HID Protocol Reference

The daemon communicates strictly via read-only feature report polling to prevent USB bridge resets:

| HID Report ID | Parameter | Precision / Scaling | Description |
| :---: | :--- | :--- | :--- |
| **12** | `BatteryLifePercent` | Percentage (0–100%) | State of charge of battery pack |
| **15** | `RemainingRunTime` | 16-bit integer (seconds) | Estimated remaining operational time |
| **22** | `UPSStatusBitmask` | Bitmask flags | Byte 1: AC Present (Bit 0), Battery Healthy (Bit 4) |
| **32** | `InternalTemp` | Celsius (°C) | Chassis internal temperature sensor |
| **38** | `BatteryVoltage` | Scaled $(V \times 100)$ VDC | Float charging voltage |
| **48** | `NominalOutputVoltage` | RMS Volts AC | Inverter nominal target output (230 VAC) |
| **49** | `InputVoltage` | RMS Volts AC | Utility grid line voltage (PLN) |
| **80** | `PercentLoad` | Percentage (0–100%) | Instantaneous capacity utilization |
| **82** | `ConfigActivePower` | 16-bit integer (Watts) | Nominal hardware capacity rating |

---

## 🏁 Quick Start

### Prerequisites
* Windows 10 / 11 (64-bit)
* USB communication cable connected to your APC UPS (RJ50-to-USB or standard USB Type-B depending on model)

### Running the Application
Simply double-click:
```cmd
APC-Power-Console.exe
```
Or build from source with Rust:
```bash
cargo build --release --manifest-path src-tauri/Cargo.toml
```

---

## 📄 License
MIT License. Open source and free for personal and commercial infrastructure management.
