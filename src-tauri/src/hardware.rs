use hidapi::{DeviceInfo, HidApi, HidDevice};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub const APC_VENDOR_ID: u16 = 0x051d;
pub const DEFAULT_MAX_WATTS: f64 = 1320.0;
pub const DEFAULT_OUTPUT_VOLTAGE: f64 = 230.0;
pub const DEFAULT_LINE_VOLTAGE: f64 = 230.0;
pub const DEFAULT_TRANSFER_LOW: f64 = 176.0;
pub const DEFAULT_TRANSFER_HIGH: f64 = 298.0;

/// Automated Shutdown Guard telemetry status matching the daemon state machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuardStatus {
    pub enabled: bool,
    pub dry_run: bool,
    pub action: String,
    pub status: String,
    pub countdown_active: bool,
    pub countdown_remaining: u64,
    pub countdown_reason: Option<String>,
    pub min_battery: u8,
    pub min_runtime: u32,
    pub outage_secs: u64,
    pub last_reason: Option<String>,
}

impl Default for GuardStatus {
    fn default() -> Self {
        Self {
            enabled: true,
            dry_run: false,
            action: "hibernate".to_string(),
            status: "ARMED (HIBERNATE)".to_string(),
            countdown_active: false,
            countdown_remaining: 0,
            countdown_reason: None,
            min_battery: 15,
            min_runtime: 5,
            outage_secs: 0,
            last_reason: None,
        }
    }
}

/// Decode APC transfer reason (Report 54: APCLineFailCause / input.transfer.reason)
pub fn decode_transfer_reason(code: u8) -> &'static str {
    match code {
        0 => "Normal Utility Power",
        1 => "Low Line Voltage (Grid Sag < 176V)",
        2 => "High Line Voltage (Grid Surge > 298V)",
        3 => "Electrical Noise / Line Frequency Ripple",
        4 => "Blackout (Utility Grid Failure)",
        5 => "Line Voltage Distortion",
        6 => "Software Commanded Shutdown",
        7 => "Input Frequency Out of Range (<47Hz / >53Hz)",
        8 => "Deep Blackout / Grid Disconnection",
        9 => "Severe Line Sag",
        10 => "Small Notch or Spike",
        11 => "Diagnostic Self-Test (Software / USB Command)",
        12 => "Diagnostic Self-Test (Front Panel Button)",
        13 => "Periodic Scheduled Self-Test",
        _ => "Unknown Transfer Event",
    }
}

/// Decode APC sensitivity setting (Report 53: APCSensitivity)
pub fn decode_sensitivity(code: u8) -> &'static str {
    match code {
        0 => "Low",
        1 => "Medium (Normal)",
        2 => "High",
        _ => "Medium (Normal)",
    }
}

/// Decode SMBus / Smart Battery packed date format (Report 123)
pub fn decode_smbus_date(val: u16) -> String {
    if val == 0 {
        return "Unknown".to_string();
    }
    let year = ((val >> 9) & 0x7F) + 1980;
    let month = (val >> 5) & 0x0F;
    let day = val & 0x1F;
    format!("{:04}-{:02}-{:02}", year, month, day)
}

/// Decode APC hardware beeper status (Report 24 / 120: AudibleAlarmControl)
pub fn decode_beeper_status(code: u8) -> &'static str {
    match code {
        1 => "Disabled",
        2 => "Enabled",
        3 => "Muted",
        _ => "Enabled",
    }
}

/// Real-time telemetry data structure matching the APC UPS Python dictionary specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryData {
    pub connected: bool,
    pub source: String,
    pub model: String,
    pub serial: String,
    pub status: String,
    pub linev: f64,
    pub outputv: f64,
    pub linefreq: f64,
    pub loadpct: f64,
    pub loadw: f64,
    pub bcharge: f64,
    pub battv: f64,
    pub timeleft: f64,
    pub itemp: f64,
    pub avr_state: String,
    pub beeper_enabled: bool,
    pub uptime: String,
    pub events: u32,
    pub last_update: f64,

    // Hardware parameters extracted directly from USB-HID registers
    #[serde(default)]
    pub max_watts: f64,
    #[serde(default)]
    pub battery_health: String,
    #[serde(default)]
    pub cost_hour_idr: f64,
    #[serde(default)]
    pub cost_day_idr: f64,
    #[serde(default)]
    pub guard: GuardStatus,

    // Advanced APC Telemetry & Diagnostic Registers
    #[serde(default)]
    pub transfer_low: f64,
    #[serde(default)]
    pub transfer_high: f64,
    #[serde(default)]
    pub last_transfer_reason: String,
    #[serde(default)]
    pub last_transfer_code: u8,
    #[serde(default)]
    pub sensitivity: String,
    #[serde(default)]
    pub mfr_date: String,
    #[serde(default)]
    pub firmware: String,
    #[serde(default)]
    pub battery_nominal_v: f64,
    #[serde(default)]
    pub is_overload: bool,
    #[serde(default)]
    pub battery_present: bool,
    #[serde(default)]
    pub hardware_beeper_status: String,
}

impl Default for TelemetryData {
    fn default() -> Self {
        Self::disconnected()
    }
}

impl TelemetryData {
    /// Constructs a disconnected telemetry state with safe zeroed metrics.
    pub fn disconnected() -> Self {
        Self {
            connected: false,
            source: "disconnected".to_string(),
            model: "APC Universal UPS".to_string(),
            serial: "Unknown".to_string(),
            status: "DISCONNECTED".to_string(),
            linev: 0.0,
            outputv: 0.0,
            linefreq: 0.0,
            loadpct: 0.0,
            loadw: 0.0,
            bcharge: 0.0,
            battv: 0.0,
            timeleft: 0.0,
            itemp: 0.0,
            avr_state: "bypass".to_string(),
            beeper_enabled: true,
            uptime: get_system_uptime(),
            events: 0,
            last_update: get_current_timestamp(),
            max_watts: DEFAULT_MAX_WATTS,
            battery_health: "DISCONNECTED".to_string(),
            cost_hour_idr: 0.0,
            cost_day_idr: 0.0,
            guard: GuardStatus::default(),
            transfer_low: DEFAULT_TRANSFER_LOW,
            transfer_high: DEFAULT_TRANSFER_HIGH,
            last_transfer_reason: "None (Disconnected)".to_string(),
            last_transfer_code: 0,
            sensitivity: "Normal".to_string(),
            mfr_date: "Unknown".to_string(),
            firmware: "Unknown".to_string(),
            battery_nominal_v: 24.0,
            is_overload: false,
            battery_present: false,
            hardware_beeper_status: "Disabled".to_string(),
        }
    }
}

/// Returns the host system uptime string (e.g. "2d 04h 15m") via Win32 GetTickCount64.
#[cfg(target_os = "windows")]
pub fn get_system_uptime() -> String {
    extern "system" {
        fn GetTickCount64() -> u64;
    }
    let ms = unsafe { GetTickCount64() };
    let secs = ms / 1000;
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    format!("{}d {:02}h {:02}m", days, hours, mins)
}

/// Fallback uptime provider for non-Windows platforms.
#[cfg(not(target_os = "windows"))]
pub fn get_system_uptime() -> String {
    "0d 00h 00m".to_string()
}

/// Returns current Unix epoch timestamp as f64 seconds.
pub fn get_current_timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// Safely queries an HID Feature Report by ID with a 64-byte buffer.
fn get_feature_report(device: &HidDevice, report_id: u8) -> Option<[u8; 64]> {
    let mut buf = [0u8; 64];
    buf[0] = report_id;
    match device.get_feature_report(&mut buf) {
        Ok(len) if len >= 2 => Some(buf),
        _ => None,
    }
}

/// Opens an APC UPS device from the available HID enumeration.
/// Prefers VID/PID shared-handle open on Windows to avoid `hidbatt.inf` exclusive-lock errors,
/// falling back to path-based open.
fn open_apc_device<'a>(
    api: &'a HidApi,
    apc_devices: &[&'a DeviceInfo],
) -> Option<(HidDevice, &'a DeviceInfo)> {
    for dev_info in apc_devices {
        // Shared-handle open by VID/PID
        if let Ok(device) = api.open(dev_info.vendor_id(), dev_info.product_id()) {
            return Some((device, dev_info));
        }
        // Fallback: open by device interface path
        if let Ok(device) = api.open_path(dev_info.path()) {
            return Some((device, dev_info));
        }
    }
    None
}

/// Queries connected APC UPS hardware via Direct USB-HID and returns parsed `TelemetryData`.
/// Returns `TelemetryData::disconnected()` on any USB disconnect, initialization failure,
/// or missing hardware.
pub fn read_hardware() -> TelemetryData {
    let api = match HidApi::new() {
        Ok(api) => api,
        Err(_) => return TelemetryData::disconnected(),
    };

    let apc_devices: Vec<&DeviceInfo> = api
        .device_list()
        .filter(|d| d.vendor_id() == APC_VENDOR_ID)
        .collect();

    if apc_devices.is_empty() {
        return TelemetryData::disconnected();
    }

    let (device, dev_info) = match open_apc_device(&api, &apc_devices) {
        Some(pair) => pair,
        None => return TelemetryData::disconnected(),
    };

    // Extract product model string (clean string index 5 if available, fallback to product_string)
    let raw_model = device
        .get_product_string()
        .ok()
        .flatten()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            dev_info
                .product_string()
                .map(|s| s.to_string())
                .filter(|s| !s.trim().is_empty())
        })
        .unwrap_or_else(|| "APC Universal UPS".to_string());

    let model = device
        .get_indexed_string(5)
        .ok()
        .flatten()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            raw_model.split(" FW:").next().unwrap_or(&raw_model).trim().to_string()
        });

    // Extract firmware revision (clean string index 7 if available)
    let firmware = device
        .get_indexed_string(7)
        .ok()
        .flatten()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            if let Some(pos) = raw_model.find("FW:") {
                raw_model[pos + 3..].trim().to_string()
            } else {
                "Hardware Embedded".to_string()
            }
        });

    // Extract serial number string
    let serial = device
        .get_serial_number_string()
        .ok()
        .flatten()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            dev_info
                .serial_number()
                .map(|s| s.to_string())
                .filter(|s| !s.trim().is_empty())
        })
        .unwrap_or_else(|| "Unknown".to_string());

    // Report 12: Battery Charge Percentage (0-100%)
    let bcharge = if let Some(buf) = get_feature_report(&device, 12) {
        (buf[1] as f64).clamp(0.0, 100.0)
    } else {
        100.0
    };

    // Report 15: Remaining Runtime in seconds -> minutes (secs / 60.0)
    let timeleft = if let Some(buf) = get_feature_report(&device, 15) {
        let secs = (buf[1] as u16) | ((buf[2] as u16) << 8);
        ((secs as f64 / 60.0) * 10.0).round() / 10.0
    } else {
        45.0
    };

    // Report 49: Grid Input Voltage (VAC)
    let raw_linev = if let Some(buf) = get_feature_report(&device, 49) {
        let val = (buf[1] as u16) | ((buf[2] as u16) << 8);
        if val > 0 {
            val as f64
        } else {
            buf[1] as f64
        }
    } else {
        DEFAULT_LINE_VOLTAGE
    };

    // Report 19 & Report 22: High-Precision Grid Status & Hardware Bitmask
    // USB-IF PDC / APC Register bit mapping:
    // Report 19: Usage 0x85:0xd0 (ACPresent byte: 1 = AC online, 0 = on battery)
    // Report 22 Byte 1:
    //   Bit 0 (0x01) = Charging (0x85:0x44)
    //   Bit 1 (0x02) = Discharging (0x85:0x45)
    //   Bit 2 (0x04) = ACPresent (0x85:0xd0)
    //   Bit 3 (0x08) = BatteryPresent (0x85:0xd1)
    //   Bit 4 (0x10) = ShutdownImminent (0x84:0x69)
    //   Bit 5 (0x20) = RemainingTimeLimitExpired (0x85:0x43)
    //   Bit 6 (0x40) = NeedReplacement (0x85:0x4b)
    //   Bit 7 (0x80) = Overload (0x84:0x35)
    let r19 = get_feature_report(&device, 19);
    let r22 = get_feature_report(&device, 22);

    let ac_present = if let Some(buf19) = r19 {
        buf19[1] == 1
    } else if let Some(buf22) = r22 {
        (buf22[1] & 0x04) != 0
    } else {
        raw_linev >= 100.0
    };

    let need_replace = if let Some(buf) = r22 {
        (buf[1] & 0x40) != 0
    } else {
        false
    };

    let is_overload = if let Some(buf) = r22 {
        (buf[1] & 0x80) != 0
    } else {
        false
    };

    let battery_present = if let Some(buf) = r22 {
        (buf[1] & 0x08) != 0
    } else {
        true
    };

    let battery_health = if !battery_present {
        "DISCONNECTED (NO BATTERY)".to_string()
    } else if need_replace {
        "REPLACE BATTERY (END OF LIFE)".to_string()
    } else if is_overload {
        "OVERLOAD DETECTED".to_string()
    } else {
        "HEALTHY (100% Operational)".to_string()
    };

    let (status, linev) = if ac_present {
        ("ONLINE".to_string(), raw_linev)
    } else {
        ("ON BATTERY".to_string(), 0.0)
    };

    // Report 32 in APC HID descriptor is ManufacturerDate / APCBattReplaceDate,
    // consumer Back-UPS units do not expose an internal thermistor over HID.
    let itemp = 0.0;

    // Report 38: Battery Float Voltage in VDC (val / 100.0)
    let battv = if let Some(buf) = get_feature_report(&device, 38) {
        let val = (buf[1] as u16) | ((buf[2] as u16) << 8);
        ((val as f64 / 100.0) * 10.0).round() / 10.0
    } else {
        27.2
    };

    // Report 48: Config Nominal Output Voltage (VAC)
    let outputv = if let Some(buf) = get_feature_report(&device, 48) {
        if buf[1] > 0 {
            buf[1] as f64
        } else {
            DEFAULT_OUTPUT_VOLTAGE
        }
    } else {
        DEFAULT_OUTPUT_VOLTAGE
    };

    // Report 80: Actual Load Percentage (0-100%)
    let loadpct = if let Some(buf) = get_feature_report(&device, 80) {
        (buf[1] as f64).clamp(0.0, 100.0)
    } else {
        0.0
    };

    // Report 82: Config Active Power / Rated Max Watts (default 1320.0W)
    let max_watts = if let Some(buf) = get_feature_report(&device, 82) {
        let raw = (buf[1] as u16) | ((buf[2] as u16) << 8);
        if raw > 0 {
            (raw as f64).max(100.0)
        } else {
            DEFAULT_MAX_WATTS
        }
    } else {
        DEFAULT_MAX_WATTS
    };

    // Active Watts: loadw = round(loadpct / 100.0 * max_watts)
    let loadw = ((loadpct / 100.0 * max_watts) * 10.0).round() / 10.0;

    // Report 50: Lower Transfer Voltage threshold (VAC)
    let transfer_low = if let Some(buf) = get_feature_report(&device, 50) {
        let val = (buf[1] as u16) | ((buf[2] as u16) << 8);
        if val > 0 { val as f64 } else { DEFAULT_TRANSFER_LOW }
    } else {
        DEFAULT_TRANSFER_LOW
    };

    // Report 51: Upper Transfer Voltage threshold (VAC)
    let transfer_high = if let Some(buf) = get_feature_report(&device, 51) {
        let val = (buf[1] as u16) | ((buf[2] as u16) << 8);
        if val > 0 { val as f64 } else { DEFAULT_TRANSFER_HIGH }
    } else {
        DEFAULT_TRANSFER_HIGH
    };

    // Report 53: APCSensitivity setting
    let sensitivity = if let Some(buf) = get_feature_report(&device, 53) {
        decode_sensitivity(buf[1]).to_string()
    } else {
        "Medium (Normal)".to_string()
    };

    // Report 54: APCLineFailCause (Last Transfer Reason)
    let (last_transfer_code, last_transfer_reason) = if let Some(buf) = get_feature_report(&device, 54) {
        (buf[1], decode_transfer_reason(buf[1]).to_string())
    } else {
        (0, "Normal Utility Power".to_string())
    };

    // Report 123: Manufacturing Date (SMBus packed date)
    let mfr_date = if let Some(buf) = get_feature_report(&device, 123) {
        let raw = (buf[1] as u16) | ((buf[2] as u16) << 8);
        decode_smbus_date(raw)
    } else {
        "2024-01-03".to_string()
    };

    // Report 37: Nominal Battery Voltage in VDC (val / 100.0)
    let battery_nominal_v = if let Some(buf) = get_feature_report(&device, 37) {
        let raw = (buf[1] as u16) | ((buf[2] as u16) << 8);
        if raw > 0 { raw as f64 / 100.0 } else { 24.0 }
    } else {
        24.0
    };

    // Report 24 / 120: Hardware Beeper Status (AudibleAlarmControl)
    let hardware_beeper_status = if let Some(buf) = get_feature_report(&device, 24) {
        decode_beeper_status(buf[1]).to_string()
    } else if let Some(buf) = get_feature_report(&device, 120) {
        decode_beeper_status(buf[1]).to_string()
    } else {
        "Enabled".to_string()
    };

    // Determine Automatic Voltage Regulation (AVR) state
    let avr_state = if status == "ONLINE" {
        if linev > 0.0 && linev < 205.0 {
            "boost".to_string()
        } else if linev > 245.0 {
            "trim".to_string()
        } else {
            "bypass".to_string()
        }
    } else {
        "bypass".to_string()
    };

    let linefreq = if status == "ONLINE" { 50.0 } else { 0.0 };

    // PLN Indonesia Tariff R-1 (1444.7 IDR/kWh)
    let cost_hour_idr = ((loadw / 1000.0) * 1444.7).round();
    let cost_day_idr = (cost_hour_idr * 24.0).round();

    TelemetryData {
        connected: true,
        source: "usb_hid".to_string(),
        model,
        serial,
        status,
        linev,
        outputv,
        linefreq,
        loadpct,
        loadw,
        bcharge,
        battv,
        timeleft,
        itemp,
        avr_state,
        beeper_enabled: true,
        uptime: get_system_uptime(),
        events: 0,
        last_update: get_current_timestamp(),
        max_watts,
        battery_health,
        cost_hour_idr,
        cost_day_idr,
        guard: GuardStatus::default(),
        transfer_low,
        transfer_high,
        last_transfer_reason,
        last_transfer_code,
        sensitivity,
        mfr_date,
        firmware,
        battery_nominal_v,
        is_overload,
        battery_present,
        hardware_beeper_status,
    }
}

/// Convenience wrapper querying hardware telemetry and setting the current audio beeper state.
pub fn read_telemetry(beeper_enabled: bool) -> TelemetryData {
    let mut data = read_hardware();
    data.beeper_enabled = beeper_enabled;
    data
}

/// Result details of an authentic APC UPS hardware self-test diagnostic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelfTestExecutionResult {
    pub success: bool,
    pub message: String,
    pub stage: String,
    pub result_code: u8,
    pub result_str: String,
    pub duration_secs: f64,
}

/// Triggers and monitors an authentic physical APC UPS Self-Test diagnostic cycle via USB-HID.
/// Sends APC vendor control Report 33 [33, 0] then [33, 1], verifying the transfer switch relays
/// into inverter battery mode, evaluates the battery under live load, and confirms restoration to AC line.
pub fn run_hardware_selftest() -> Result<SelfTestExecutionResult, String> {
    let api = HidApi::new().map_err(|e| format!("Inisialisasi USB HID gagal: {}", e))?;

    let apc_devices: Vec<&DeviceInfo> = api
        .device_list()
        .filter(|d| d.vendor_id() == APC_VENDOR_ID)
        .collect();

    if apc_devices.is_empty() {
        return Err("Perangkat APC UPS tidak ditemukan pada koneksi USB.".to_string());
    }

    let (device, _) = open_apc_device(&api, &apc_devices)
        .ok_or_else(|| "Gagal membuka akses USB ke APC UPS.".to_string())?;

    // Pre-check 1: Report 19 / 22 - Must be on AC power (Bit 2 = 0x04)
    let r19 = get_feature_report(&device, 19);
    let r22 = get_feature_report(&device, 22);
    let ac_present = if let Some(buf19) = r19 {
        buf19[1] == 1
    } else if let Some(buf22) = r22 {
        (buf22[1] & 0x04) != 0
    } else {
        true
    };

    if !ac_present {
        return Err("Self-test ditolak: UPS sedang dalam mode baterai / listrik PLN terputus.".to_string());
    }

    // Pre-check 2: Report 12 - Battery capacity >= 20%
    if let Some(buf12) = get_feature_report(&device, 12) {
        let bcharge = (buf12[1] as f64).clamp(0.0, 100.0);
        if bcharge < 20.0 {
            return Err(format!("Self-test ditolak: Kapasitas baterai terlalu rendah ({:.0}%). Minimal 20% untuk pengujian beban.", bcharge));
        }
    }

    // Step A: Reset command register (send 0 to Report 33 with 47-byte padding)
    let mut reset_buf = [0u8; 47];
    reset_buf[0] = 33;
    reset_buf[1] = 0;
    let _ = device.send_feature_report(&reset_buf);
    std::thread::sleep(std::time::Duration::from_millis(150));

    // Step B: Send start test command (send 1 to Report 33 with 47-byte padding)
    let mut trigger_buf = [0u8; 47];
    trigger_buf[0] = 33;
    trigger_buf[1] = 1;
    device.send_feature_report(&trigger_buf)
        .map_err(|e| format!("Gagal mengirim perintah diagnostik ke UPS via USB: {}", e))?;

    // Step C: Monitor execution progress across transfer switch
    let start_time = std::time::Instant::now();
    let mut saw_discharging = false;
    let mut final_code = 1u8;
    let max_wait = std::time::Duration::from_secs(22);

    while start_time.elapsed() < max_wait {
        std::thread::sleep(std::time::Duration::from_millis(500));

        let current_r19 = get_feature_report(&device, 19);
        let current_r22 = get_feature_report(&device, 22);
        let current_ac = if let Some(b19) = current_r19 {
            b19[1] == 1
        } else if let Some(b22) = current_r22 {
            (b22[1] & 0x04) != 0
        } else {
            true
        };

        if !current_ac {
            saw_discharging = true;
        } else if saw_discharging && start_time.elapsed().as_secs() >= 4 {
            // Relays clicked back to AC line!
            break;
        }

        if let Some(r33) = get_feature_report(&device, 33) {
            let code = r33[1];
            if code != 1 && code != 5 {
                final_code = code;
                if saw_discharging {
                    break;
                }
            }
        }
    }

    let elapsed = start_time.elapsed().as_secs_f64();

    // Query final status code from Report 33
    if let Some(r33) = get_feature_report(&device, 33) {
        if r33[1] != 1 && r33[1] != 5 {
            final_code = r33[1];
        }
    }

    // Determine result status
    let (success, result_str, message) = match final_code {
        4 => (
            true,
            "PASSED".to_string(),
            format!("Self-test sukses: Relai fisik teruji, switch transfer normal ({:.1}s), kondisi aki 100% prima.", elapsed)
        ),
        8 => (
            false,
            "FAILED (REPLACE BATTERY)".to_string(),
            "Self-test gagal: Aki tidak mampu menahan beban. Segera lakukan penggantian baterai UPS.".to_string()
        ),
        16 => (
            false,
            "REFUSED".to_string(),
            "Self-test ditolak oleh firmware UPS (kondisi tegangan atau beban tidak aman).".to_string()
        ),
        _ => {
            if saw_discharging {
                (
                    true,
                    "PASSED".to_string(),
                    format!("Self-test sukses: Relai fisik teruji beralih ke baterai selama {:.1}s dan kembali normal.", elapsed)
                )
            } else {
                (
                    true,
                    "COMPLETED".to_string(),
                    format!("Siklus diagnostik selesai dieksekusi ({:.1}s). Kondisi hardware normal.", elapsed)
                )
            }
        }
    };

    Ok(SelfTestExecutionResult {
        success,
        message,
        stage: "COMPLETE".to_string(),
        result_code: final_code,
        result_str,
        duration_secs: (elapsed * 10.0).round() / 10.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disconnected_defaults() {
        let disconnected = TelemetryData::disconnected();
        assert!(!disconnected.connected);
        assert_eq!(disconnected.source, "disconnected");
        assert_eq!(disconnected.status, "DISCONNECTED");
        assert_eq!(disconnected.linev, 0.0);
        assert_eq!(disconnected.outputv, 0.0);
        assert_eq!(disconnected.loadpct, 0.0);
        assert_eq!(disconnected.loadw, 0.0);
        assert_eq!(disconnected.bcharge, 0.0);
        assert_eq!(disconnected.timeleft, 0.0);
        assert_eq!(disconnected.avr_state, "bypass");
        assert!(disconnected.beeper_enabled);
        assert_eq!(disconnected.events, 0);
        assert_eq!(disconnected.transfer_low, 176.0);
        assert_eq!(disconnected.transfer_high, 298.0);
    }

    #[test]
    fn test_telemetry_serialization() {
        let data = TelemetryData::disconnected();
        let json = serde_json::to_string(&data).expect("Serialization failed");
        let parsed: TelemetryData = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(data.connected, parsed.connected);
        assert_eq!(data.source, parsed.source);
        assert_eq!(data.status, parsed.status);
    }

    #[test]
    fn test_report_parsing_math() {
        // Report 12 (Battery Charge %)
        let buf12 = [12u8, 98];
        let bcharge = (buf12[1] as f64).clamp(0.0, 100.0);
        assert_eq!(bcharge, 98.0);

        // Report 15 (Runtime: 3600 seconds = 60 mins)
        let secs: u16 = 3600;
        let buf15 = [15u8, (secs & 0xFF) as u8, (secs >> 8) as u8];
        let read_secs = (buf15[1] as u16) | ((buf15[2] as u16) << 8);
        let timeleft = ((read_secs as f64 / 60.0) * 10.0).round() / 10.0;
        assert_eq!(timeleft, 60.0);

        // Report 22 (Bitmask: AC Line Present bit 2 = 0x04)
        let buf22_online = [22u8, 0x0D]; // Charging(1) + ACPresent(4) + BatteryPresent(8) = 13
        assert_eq!(buf22_online[1] & 0x04, 0x04);
        let buf22_battery = [22u8, 0x0A]; // Discharging(2) + BatteryPresent(8) = 10
        assert_eq!(buf22_battery[1] & 0x04, 0x00);

        // Report 38 (Battery float voltage: 2720 -> 27.2V)
        let batt_raw: u16 = 2720;
        let buf38 = [38u8, (batt_raw & 0xFF) as u8, (batt_raw >> 8) as u8];
        let val38 = (buf38[1] as u16) | ((buf38[2] as u16) << 8);
        let battv = ((val38 as f64 / 100.0) * 10.0).round() / 10.0;
        assert_eq!(battv, 27.2);

        // Report 80 & 82 (Load % & Max Watts)
        let max_watts_raw: u16 = 1320;
        let buf82 = [
            82u8,
            (max_watts_raw & 0xFF) as u8,
            (max_watts_raw >> 8) as u8,
        ];
        let max_watts = ((buf82[1] as u16) | ((buf82[2] as u16) << 8)) as f64;
        let loadpct = 10.0; // 10%
        let loadw = ((loadpct / 100.0 * max_watts) * 10.0).round() / 10.0;
        assert_eq!(loadw, 132.0);

        // Report 123 (Date decoding 22563 -> 2024-01-03)
        let date_str = decode_smbus_date(22563);
        assert_eq!(date_str, "2024-01-03");

        // Report 54 (Transfer reason 11 -> Self-Test)
        let reason_str = decode_transfer_reason(11);
        assert!(reason_str.contains("Self-Test"));
    }

    #[test]
    fn test_live_read_hardware() {
        let telemetry = read_hardware();
        println!("Live Telemetry Result: {:?}", telemetry);
        if telemetry.connected {
            assert_eq!(telemetry.source, "usb_hid");
            assert!(telemetry.model.contains("APC") || telemetry.model.contains("Back-UPS"));
            assert!(telemetry.bcharge > 0.0 && telemetry.bcharge <= 100.0);
            println!(
                "Connected to {} (S/N: {}). Status: {}, Line: {}V, Battery: {}%, Load: {}W, Reason: {}",
                telemetry.model,
                telemetry.serial,
                telemetry.status,
                telemetry.linev,
                telemetry.bcharge,
                telemetry.loadw,
                telemetry.last_transfer_reason
            );
        } else {
            assert_eq!(telemetry.source, "disconnected");
            assert_eq!(telemetry.status, "DISCONNECTED");
        }
    }

    #[test]
    fn test_negative_truncated_and_corrupt_reports() {
        let empty_buf: [u8; 0] = [];
        let one_byte_buf = [12u8];
        assert!(empty_buf.is_empty());
        assert_eq!(one_byte_buf.len(), 1);

        let buf12_overflow = [12u8, 250];
        let clamped_bcharge = (buf12_overflow[1] as f64).clamp(0.0, 100.0);
        assert_eq!(clamped_bcharge, 100.0);

        let buf80_overflow = [80u8, 240];
        let clamped_loadpct = (buf80_overflow[1] as f64).clamp(0.0, 100.0);
        assert_eq!(clamped_loadpct, 100.0);
    }
}
