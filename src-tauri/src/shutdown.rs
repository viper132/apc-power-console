use crate::hardware::TelemetryData;
use chrono::{Datelike, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

/// Shutdown and automated power protection configuration policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ShutdownPolicy {
    /// Live automated shutdown guard active
    pub enabled: bool,
    /// Dry run mode: logs planned actions without executing system shutdown commands
    pub dry_run: bool,
    /// Action mode: "hibernate" or "shutdown"
    pub action: String,
    /// Minimum battery percentage threshold (5..=90)
    pub min_battery_pct: u8,
    /// Minimum estimated runtime minutes threshold (1..=60)
    pub min_runtime_mins: u32,
    /// Outage duration timeout in seconds (30..=3600)
    pub outage_timeout_secs: u64,
    /// Grace period countdown delay in seconds (5..=300)
    pub shutdown_grace_secs: u64,
    /// Electricity tariff in IDR per kWh (e.g. PLN R-1 1444.7)
    pub kwh_rate_idr: f64,
    /// Reason recorded from the last triggered protection event
    pub last_trigger_reason: Option<String>,
    /// Whether closing the main window minimizes to tray instead of quitting
    pub close_to_tray: bool,
    /// Optional command or script file (.bat/.cmd/.ps1) to execute before system hibernation/shutdown
    pub pre_shutdown_script: Option<String>,
    /// Maximum seconds allocated for pre-shutdown script execution (5..=120)
    pub pre_shutdown_timeout_secs: u32,
    /// Optional script file to execute when utility grid power is restored
    pub post_restore_script: Option<String>,
    /// Continuous grid stabilization delay in seconds before executing post-restore hook (5..=60)
    pub post_restore_delay_secs: u32,
    /// Battery installation date string in YYYY-MM-DD format for chemical aging tracking
    pub battery_install_date: Option<String>,
    /// Interval in days for scheduled automated battery self-test (e.g. 14 or 30 days, set 0 to disable)
    pub selftest_interval_days: u32,
    /// Unix timestamp of last executed self-test
    pub last_selftest_timestamp: u64,
}

impl Default for ShutdownPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            dry_run: false,
            action: "hibernate".to_string(),
            min_battery_pct: 15,
            min_runtime_mins: 5,
            outage_timeout_secs: 600,
            shutdown_grace_secs: 60,
            kwh_rate_idr: 1444.7,
            last_trigger_reason: None,
            close_to_tray: true,
            pre_shutdown_script: None,
            pre_shutdown_timeout_secs: 30,
            post_restore_script: None,
            post_restore_delay_secs: 10,
            battery_install_date: Some("2024-01-03".to_string()),
            selftest_interval_days: 30,
            last_selftest_timestamp: 0,
        }
    }
}

impl ShutdownPolicy {
    /// Validate and normalize policy fields against safe operating limits.
    pub fn validate(&mut self) -> Result<(), String> {
        self.action = self.action.trim().to_lowercase();
        if self.action != "hibernate" && self.action != "shutdown" {
            return Err("Action must be either 'hibernate' or 'shutdown'".to_string());
        }

        if !(5..=90).contains(&self.min_battery_pct) {
            return Err(format!(
                "min_battery_pct ({}) must be between 5 and 90",
                self.min_battery_pct
            ));
        }

        if !(1..=60).contains(&self.min_runtime_mins) {
            return Err(format!(
                "min_runtime_mins ({}) must be between 1 and 60",
                self.min_runtime_mins
            ));
        }

        if !(30..=3600).contains(&self.outage_timeout_secs) {
            return Err(format!(
                "outage_timeout_secs ({}) must be between 30 and 3600",
                self.outage_timeout_secs
            ));
        }

        if !(5..=300).contains(&self.shutdown_grace_secs) {
            return Err(format!(
                "shutdown_grace_secs ({}) must be between 5 and 300",
                self.shutdown_grace_secs
            ));
        }

        if !self.kwh_rate_idr.is_finite() || self.kwh_rate_idr < 0.0 {
            return Err(format!(
                "kwh_rate_idr ({}) must be a non-negative finite number",
                self.kwh_rate_idr
            ));
        }

        if !(5..=120).contains(&self.pre_shutdown_timeout_secs) {
            self.pre_shutdown_timeout_secs = 30;
        }

        if !(5..=60).contains(&self.post_restore_delay_secs) {
            self.post_restore_delay_secs = 10;
        }

        self.dry_run = false;

        Ok(())
    }

    /// Save current policy as formatted JSON to %LOCALAPPDATA%\ApcPowerConsole\shutdown_policy.json
    pub fn save_policy(&self) -> Result<(), String> {
        let path = get_config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e))?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize policy: {}", e))?;

        std::fs::write(&path, json)
            .map_err(|e| format!("Failed to write policy file {}: {}", path.display(), e))?;

        Ok(())
    }

    /// Computes battery operational age in months based on install date
    pub fn battery_age_months(&self) -> Option<u32> {
        if let Some(ref date_str) = self.battery_install_date {
            compute_battery_age_months(date_str)
        } else {
            None
        }
    }

    /// True if battery age is 36 months (3 years) or older
    pub fn battery_replacement_advisory(&self) -> bool {
        self.battery_age_months().map(|m| m >= 36).unwrap_or(false)
    }
}

/// Computes battery operational age in months given an installation date string "YYYY-MM-DD"
pub fn compute_battery_age_months(install_date: &str) -> Option<u32> {
    let parsed = NaiveDate::parse_from_str(install_date.trim(), "%Y-%m-%d").ok()?;
    let today = Local::now().date_naive();
    if today < parsed {
        return Some(0);
    }
    let years = today.year() - parsed.year();
    let months = (years * 12) + (today.month() as i32 - parsed.month() as i32);
    Some(months.max(0) as u32)
}

/// Returns the configuration file path pointing to `%LOCALAPPDATA%\ApcPowerConsole\shutdown_policy.json`
pub fn get_config_path() -> PathBuf {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_app_data)
            .join("ApcPowerConsole")
            .join("shutdown_policy.json")
    } else if let Ok(user_profile) = std::env::var("USERPROFILE") {
        PathBuf::from(user_profile)
            .join("AppData")
            .join("Local")
            .join("ApcPowerConsole")
            .join("shutdown_policy.json")
    } else {
        PathBuf::from("shutdown_policy.json")
    }
}

/// Loads policy from JSON file; if missing or invalid, returns default policy.
pub fn load_policy() -> ShutdownPolicy {
    let config_path = get_config_path();
    let local_path = PathBuf::from("shutdown_policy.json");

    let path_to_read = if config_path.exists() {
        Some(config_path)
    } else if local_path.exists() {
        Some(local_path)
    } else {
        None
    };

    if let Some(path) = path_to_read {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(mut policy) = serde_json::from_str::<ShutdownPolicy>(&content) {
                if policy.validate().is_ok() {
                    return policy;
                }
            }
        }
    }

    ShutdownPolicy::default()
}

/// Helper function to save a ShutdownPolicy
pub fn save_policy(policy: &ShutdownPolicy) -> Result<(), String> {
    policy.save_policy()
}

/// Helper function to save a ShutdownPolicy to disk (alias for save_policy)
pub fn save_policy_to_disk(policy: &ShutdownPolicy) -> Result<(), String> {
    policy.save_policy()
}

/// Sanitizes a string for safe inclusion in single-quoted PowerShell WinRT scripts
fn sanitize_for_powershell(input: &str, max_len: usize) -> String {
    input
        .replace('\'', "''")
        .replace('\r', " ")
        .replace('\n', " ")
        .trim()
        .chars()
        .take(max_len)
        .collect()
}

/// Asynchronously spawns a user-defined hook script without showing console windows
pub fn execute_script_nonblocking(script_path: &str) {
    let clean_path = script_path.trim();
    if clean_path.is_empty() {
        return;
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let path = std::path::Path::new(clean_path);
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext == "ps1" {
            let _ = std::process::Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                    clean_path,
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn();
        } else {
            let _ = std::process::Command::new("cmd")
                .args(["/c", clean_path])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn();
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("sh")
            .arg("-c")
            .arg(clean_path)
            .spawn();
    }
}

/// Write critical event to Windows Application Event Log (EventViewer)
pub fn log_windows_event(entry_type: &str, event_id: u32, message: &str) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let clean_msg = sanitize_for_powershell(message, 300);
        let ps_cmd = format!(
            "Write-EventLog -LogName Application -Source 'Application' -EntryType {} -EventId {} -Message '{}' -ErrorAction SilentlyContinue",
            entry_type, event_id, clean_msg
        );
        let _ = std::process::Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_cmd])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (entry_type, event_id, message);
    }
}

/// Daemon manager responsible for evaluating telemetry against policy and executing graceful shutdown
#[derive(Debug)]
pub struct ShutdownManager {
    pub policy: ShutdownPolicy,
    pub blackout_start: Option<Instant>,
    pub countdown_start: Option<Instant>,
    pub trigger_reason: Option<String>,
    pub in_countdown: bool,
    pub is_shutdown_scheduled: bool,
    pub selftest_active: bool,
    pub pre_script_executed: bool,
    pub post_restore_candidate: Option<Instant>,
    pub post_restore_executed: bool,
}

impl Default for ShutdownManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ShutdownManager {
    /// Creates a new ShutdownManager by loading policy from disk
    pub fn new() -> Self {
        Self {
            policy: load_policy(),
            blackout_start: None,
            countdown_start: None,
            trigger_reason: None,
            in_countdown: false,
            is_shutdown_scheduled: false,
            selftest_active: false,
            pre_script_executed: false,
            post_restore_candidate: None,
            post_restore_executed: true,
        }
    }

    /// Creates a new ShutdownManager with a specific policy
    pub fn with_policy(policy: ShutdownPolicy) -> Self {
        Self {
            policy,
            blackout_start: None,
            countdown_start: None,
            trigger_reason: None,
            in_countdown: false,
            is_shutdown_scheduled: false,
            selftest_active: false,
            pre_script_executed: false,
            post_restore_candidate: None,
            post_restore_executed: true,
        }
    }

    /// Display native Windows 11 Toast Notification via PowerShell WinRT API
    pub fn notify_windows(title: &str, message: &str) {
        let clean_title = sanitize_for_powershell(title, 100);
        let clean_msg = sanitize_for_powershell(message, 250);

        let ps_cmd = format!(
            "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null;\
            $template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02);\
            $textNodes = $template.GetElementsByTagName('text');\
            $textNodes.Item(0).AppendChild($template.CreateTextNode('{}')) | Out-Null;\
            $textNodes.Item(1).AppendChild($template.CreateTextNode('{}')) | Out-Null;\
            $notifier = [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('{{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}}\\\\WindowsPowerShell\\\\v1.0\\\\powershell.exe');\
            $notification = [Windows.UI.Notifications.ToastNotification]::new($template);\
            $notifier.Show($notification);\
            [System.Console]::Beep(1200, 300);",
            clean_title, clean_msg
        );

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_cmd])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn();
        }

        #[cfg(not(target_os = "windows"))]
        {
            println!("[NOTIFY TOAST] {}: {}", clean_title, clean_msg);
        }
    }

    /// Cancels any active countdown or scheduled shutdown
    pub fn abort_shutdown(&mut self) -> bool {
        let was_active = self.in_countdown || self.is_shutdown_scheduled;
        self.in_countdown = false;
        self.countdown_start = None;
        self.trigger_reason = None;
        self.is_shutdown_scheduled = false;
        self.pre_script_executed = false;

        if was_active {
            println!("[*] Aborting scheduled Windows shutdown/hibernate...");
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("shutdown.exe")
                    .arg("/a")
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn();
            }
            Self::notify_windows(
                "APC UPS — PROTECTION ABORTED",
                "Automated shutdown/hibernation procedure was safely cancelled.",
            );
            true
        } else {
            false
        }
    }

    /// Evaluates live hardware telemetry conditions against protection policy rules.
    /// Returns an Option<String> with status/trigger description when an event occurs or countdown is active.
    pub fn evaluate(&mut self, telemetry: &TelemetryData) -> Option<String> {
        // While user-initiated hardware self-test diagnostic is running, do not trigger emergency blackout procedures
        if self.selftest_active {
            return None;
        }

        if !self.policy.enabled {
            if self.in_countdown {
                self.abort_shutdown();
            }
            return None;
        }

        // Robust parsing & sanitization against NaN/Inf values
        let bcharge = if telemetry.bcharge.is_finite() {
            telemetry.bcharge.clamp(0.0, 100.0)
        } else {
            100.0
        };

        let timeleft = if telemetry.timeleft.is_finite() {
            telemetry.timeleft.max(0.0)
        } else {
            48.0
        };

        let linev = if telemetry.linev.is_finite() {
            telemetry.linev.max(0.0)
        } else {
            230.0
        };

        let status_upper = telemetry.status.trim().to_uppercase();
        let is_ac_online = status_upper == "ONLINE" && linev > 50.0;
        let is_on_battery = status_upper == "ON BATTERY"
            || status_upper == "BATTERY"
            || status_upper == "CRITICAL"
            || linev == 0.0
            || (bcharge < 100.0 && linev <= 50.0);

        // 1. Check for AC Grid Power Restoration
        if is_ac_online {
            if self.blackout_start.is_some() || self.in_countdown || self.is_shutdown_scheduled {
                println!("[!] UTILITY GRID RESTORED: Operation returned to AC mains.");
                self.abort_shutdown();
                self.blackout_start = None;
                self.post_restore_candidate = Some(Instant::now());
                self.post_restore_executed = false;

                Self::notify_windows(
                    "APC UPS — GRID POWER RESTORED",
                    "Utility power restored. Emergency protection cancelled.",
                );
                return Some("Grid power restored: protection cancelled".to_string());
            }

            // Post-restore hook evaluation with stability debounce delay
            if !self.post_restore_executed {
                if let Some(start) = self.post_restore_candidate {
                    let delay = self.policy.post_restore_delay_secs.max(5) as u64;
                    if start.elapsed().as_secs() >= delay {
                        if let Some(ref script) = self.policy.post_restore_script {
                            if !script.trim().is_empty() {
                                println!(
                                    "[*] Grid power stable for {}s. Executing post-restore script: {}",
                                    delay, script
                                );
                                execute_script_nonblocking(script);
                            }
                        }
                        self.post_restore_executed = true;
                    }
                }
            }

            return None;
        }

        // 2. Track Outage Start
        if is_on_battery && self.blackout_start.is_none() {
            self.blackout_start = Some(Instant::now());
            self.post_restore_candidate = None;
            self.post_restore_executed = true;
            println!("[!] BLACKOUT DETECTED: Tracking continuous power outage duration...");
        }

        // 3. Tick Countdown if active
        if self.in_countdown {
            // Execute pre-shutdown command script once at start of countdown
            if !self.pre_script_executed {
                if let Some(ref script) = self.policy.pre_shutdown_script {
                    if !script.trim().is_empty() {
                        println!("[*] Executing pre-shutdown command script: {}", script);
                        execute_script_nonblocking(script);
                    }
                }
                self.pre_script_executed = true;
            }

            let elapsed = self
                .countdown_start
                .map(|s| s.elapsed().as_secs())
                .unwrap_or(0);
            if elapsed >= self.policy.shutdown_grace_secs {
                self.in_countdown = false;
                self.execute_protection();
                return Some(format!(
                    "Grace period expired: executing {} ({})",
                    self.policy.action,
                    self.trigger_reason.as_deref().unwrap_or("Unknown")
                ));
            } else {
                let remaining = self.policy.shutdown_grace_secs.saturating_sub(elapsed);
                return Some(format!(
                    "Protection countdown active: {}s remaining ({})",
                    remaining,
                    self.trigger_reason.as_deref().unwrap_or("")
                ));
            }
        }

        // 4. If shutdown already scheduled/executed, do nothing further
        if self.is_shutdown_scheduled {
            return Some("Protection already scheduled/executed".to_string());
        }

        // 5. Check Triggers on Real Power Loss
        let blackout_duration = self
            .blackout_start
            .map(|s| s.elapsed().as_secs())
            .unwrap_or(0);
        let mut trigger_reason = None;

        if bcharge <= self.policy.min_battery_pct as f64 {
            trigger_reason = Some(format!(
                "Critical battery capacity: {:.1}% (Threshold: {}%)",
                bcharge, self.policy.min_battery_pct
            ));
        } else if timeleft <= self.policy.min_runtime_mins as f64 {
            trigger_reason = Some(format!(
                "Critical battery runtime: {:.1} mins (Threshold: {} mins)",
                timeleft, self.policy.min_runtime_mins
            ));
        } else if self.policy.outage_timeout_secs > 0
            && blackout_duration >= self.policy.outage_timeout_secs
        {
            trigger_reason = Some(format!(
                "Continuous outage exceeded threshold: {:.1} mins (Limit: {:.1} mins)",
                blackout_duration as f64 / 60.0,
                self.policy.outage_timeout_secs as f64 / 60.0
            ));
        }

        if let Some(reason) = trigger_reason {
            self.in_countdown = true;
            self.countdown_start = Some(Instant::now());
            self.trigger_reason = Some(reason.clone());
            self.policy.last_trigger_reason = Some(reason.clone());
            let _ = self.policy.save_policy();

            let action_label = if self.policy.action == "hibernate" {
                "HIBERNATE"
            } else {
                "SHUTDOWN"
            };
            let title = format!("APC UPS — EMERGENCY {}", action_label);
            let msg = format!(
                "{}. System will {} in {} seconds! Save your work now.",
                reason,
                action_label.to_lowercase(),
                self.policy.shutdown_grace_secs
            );

            println!("\n=======================================================");
            println!("[!] EMERGENCY POWER COUNTDOWN STARTED ({action_label}):");
            println!("[!] Reason: {reason}");
            println!(
                "[!] Grace Period: {} seconds",
                self.policy.shutdown_grace_secs
            );
            println!("=======================================================\n");

            Self::notify_windows(&title, &msg);
            log_windows_event("Warning", 1003, &format!("Emergency Protection Countdown Started: {}", reason));
            return Some(format!("Countdown started: {}", reason));
        }

        None
    }

    /// Executes the configured protection action (hibernate or shutdown)
    pub fn execute_protection(&mut self) {
        self.is_shutdown_scheduled = true;
        let action_label = if self.policy.action == "hibernate" {
            "HIBERNATE"
        } else {
            "SHUTDOWN"
        };

        println!("\n=======================================================");
        println!("[!] GRACE PERIOD EXPIRED -> EXECUTING {}", action_label);
        if let Some(reason) = &self.trigger_reason {
            println!("[!] Reason: {}", reason);
        }
        println!("=======================================================\n");

        Self::notify_windows(
            &format!("APC UPS — {} NOW", action_label),
            &format!(
                "Grace period expired. Executing system {}...",
                action_label.to_lowercase()
            ),
        );

        log_windows_event(
            "Error",
            1004,
            &format!("Grace period expired. Executing system {}!", action_label),
        );

        if self.policy.dry_run {
            println!(
                "[DRY RUN] System action bypassed (dry_run=true). Action: {}",
                self.policy.action
            );
            return;
        }

        if self.policy.action == "hibernate" {
            println!("[!] Executing: shutdown.exe /h");
            #[cfg(target_os = "windows")]
            {
                match std::process::Command::new("shutdown.exe").arg("/h").spawn() {
                    Ok(_) => println!("[*] Hibernation command spawned successfully"),
                    Err(e) => eprintln!("[!] Failed to spawn hibernation command: {}", e),
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                println!(
                    "[SIMULATED] std::process::Command::new(\"shutdown.exe\").arg(\"/h\").spawn()"
                );
            }
        } else {
            println!("[!] Executing: shutdown.exe /s /t 60");
            #[cfg(target_os = "windows")]
            {
                match std::process::Command::new("shutdown.exe")
                    .args(["/s", "/t", "60"])
                    .spawn()
                {
                    Ok(_) => println!("[*] Shutdown command spawned successfully"),
                    Err(e) => eprintln!("[!] Failed to spawn shutdown command: {}", e),
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                println!("[SIMULATED] std::process::Command::new(\"shutdown.exe\").args([\"/s\", \"/t\", \"60\"]).spawn()");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = ShutdownPolicy::default();
        assert!(policy.enabled);
        assert!(!policy.dry_run);
        assert_eq!(policy.action, "hibernate");
        assert_eq!(policy.min_battery_pct, 15);
        assert_eq!(policy.min_runtime_mins, 5);
        assert_eq!(policy.outage_timeout_secs, 600);
        assert_eq!(policy.shutdown_grace_secs, 60);
        assert_eq!(policy.kwh_rate_idr, 1444.7);
        assert!(policy.close_to_tray);
        assert_eq!(policy.pre_shutdown_timeout_secs, 30);
        assert_eq!(policy.post_restore_delay_secs, 10);
    }

    #[test]
    fn test_battery_age_computation() {
        let age = compute_battery_age_months("2024-01-03");
        assert!(age.is_some());
        assert!(age.unwrap() >= 24);
    }

    #[test]
    fn test_policy_validation() {
        let mut policy = ShutdownPolicy::default();
        assert!(policy.validate().is_ok());

        policy.action = "invalid_action".to_string();
        assert!(policy.validate().is_err());

        policy.action = "shutdown".to_string();
        policy.min_battery_pct = 2; // out of range 5..=90
        assert!(policy.validate().is_err());

        policy.min_battery_pct = 15;
        policy.min_runtime_mins = 0; // out of range 1..=60
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_powershell_sanitization() {
        let unsafe_title = "User's APC UPS; Stop-Process -Name explorer\r\n";
        let sanitized = sanitize_for_powershell(unsafe_title, 100);
        assert!(!sanitized.contains('\r'));
        assert!(!sanitized.contains('\n'));
        assert!(sanitized.contains("User''s"));
    }

    #[test]
    fn test_save_and_load_policy() {
        let mut policy = ShutdownPolicy::default();
        policy.min_battery_pct = 20;
        policy.action = "shutdown".to_string();

        let json = serde_json::to_string_pretty(&policy).expect("serialization failed");
        let loaded: ShutdownPolicy = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(loaded.min_battery_pct, 20);
        assert_eq!(loaded.action, "shutdown");
    }

    #[test]
    fn test_evaluate_online() {
        let mut mgr = ShutdownManager::new();
        let mut telemetry = TelemetryData::default();
        telemetry.status = "ONLINE".to_string();
        telemetry.linev = 230.0;
        telemetry.bcharge = 100.0;

        let res = mgr.evaluate(&telemetry);
        assert_eq!(res, None);
        assert!(!mgr.in_countdown);
    }

    #[test]
    fn test_evaluate_disabled_policy() {
        let mut policy = ShutdownPolicy::default();
        policy.enabled = false;
        let mut mgr = ShutdownManager::with_policy(policy);

        let mut telemetry = TelemetryData::default();
        telemetry.status = "ON BATTERY".to_string();
        telemetry.linev = 0.0;
        telemetry.bcharge = 5.0; // critical

        let res = mgr.evaluate(&telemetry);
        assert_eq!(res, None);
        assert!(!mgr.in_countdown);
    }

    #[test]
    fn test_evaluate_trigger_low_battery() {
        let mut mgr = ShutdownManager::new();
        mgr.policy.min_battery_pct = 20;

        let mut telemetry = TelemetryData::default();
        telemetry.status = "ON BATTERY".to_string();
        telemetry.linev = 0.0;
        telemetry.bcharge = 15.0; // trigger

        let res = mgr.evaluate(&telemetry);
        assert!(res.is_some());
        assert!(mgr.in_countdown);
        assert!(mgr.trigger_reason.as_ref().unwrap().contains("battery"));
    }

    #[test]
    fn test_evaluate_trigger_low_runtime() {
        let mut mgr = ShutdownManager::new();
        mgr.policy.min_runtime_mins = 10;

        let mut telemetry = TelemetryData::default();
        telemetry.status = "ON BATTERY".to_string();
        telemetry.linev = 0.0;
        telemetry.bcharge = 50.0;
        telemetry.timeleft = 5.0; // trigger

        let res = mgr.evaluate(&telemetry);
        assert!(res.is_some());
        assert!(mgr.in_countdown);
        assert!(mgr.trigger_reason.as_ref().unwrap().contains("runtime"));
    }

    #[test]
    fn test_evaluate_trigger_outage_timeout() {
        let mut mgr = ShutdownManager::new();
        mgr.policy.outage_timeout_secs = 30;

        let mut telemetry = TelemetryData::default();
        telemetry.status = "ON BATTERY".to_string();
        telemetry.linev = 0.0;
        telemetry.bcharge = 80.0;
        telemetry.timeleft = 30.0;

        // First evaluate marks outage start
        let _ = mgr.evaluate(&telemetry);
        assert!(!mgr.in_countdown);

        // Advance blackout start by 35 seconds
        mgr.blackout_start = Some(Instant::now() - std::time::Duration::from_secs(35));

        let res = mgr.evaluate(&telemetry);
        assert!(res.is_some());
        assert!(mgr.in_countdown);
        assert!(mgr.trigger_reason.as_ref().unwrap().contains("outage"));
    }

    #[test]
    fn test_power_restoration_aborts_countdown() {
        let mut mgr = ShutdownManager::new();
        mgr.in_countdown = true;
        mgr.countdown_start = Some(Instant::now());
        mgr.trigger_reason = Some("Testing countdown".to_string());

        let mut telemetry = TelemetryData::default();
        telemetry.status = "ONLINE".to_string();
        telemetry.linev = 228.0;
        telemetry.bcharge = 50.0;

        let res = mgr.evaluate(&telemetry);
        assert!(res.is_some());
        assert!(!mgr.in_countdown);
        assert!(mgr.countdown_start.is_none());
    }

    #[test]
    fn test_evaluate_countdown_completion() {
        let mut mgr = ShutdownManager::new();
        mgr.policy.shutdown_grace_secs = 60;
        mgr.policy.dry_run = true;
        mgr.in_countdown = true;
        mgr.countdown_start = Some(Instant::now() - std::time::Duration::from_secs(65));
        mgr.trigger_reason = Some("Testing countdown expiration".to_string());

        let mut telemetry = TelemetryData::default();
        telemetry.status = "ON BATTERY".to_string();
        telemetry.linev = 0.0;

        let res = mgr.evaluate(&telemetry);
        assert!(res.is_some());
        assert!(res.unwrap().contains("expired"));
        assert!(!mgr.in_countdown);
        assert!(mgr.is_shutdown_scheduled);
    }

    #[test]
    fn test_manual_abort() {
        let mut mgr = ShutdownManager::new();
        mgr.in_countdown = true;
        mgr.is_shutdown_scheduled = true;

        let aborted = mgr.abort_shutdown();
        assert!(aborted);
        assert!(!mgr.in_countdown);
        assert!(!mgr.is_shutdown_scheduled);

        let aborted_again = mgr.abort_shutdown();
        assert!(!aborted_again);
    }

    #[test]
    fn test_robustness_nan_inf() {
        let mut mgr = ShutdownManager::new();
        let mut telemetry = TelemetryData::default();
        telemetry.bcharge = f64::NAN;
        telemetry.timeleft = f64::INFINITY;
        telemetry.linev = f64::NEG_INFINITY;
        telemetry.status = "ONLINE".to_string();

        let res = mgr.evaluate(&telemetry);
        assert_eq!(res, None);
    }
}
