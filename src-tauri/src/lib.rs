pub mod energy;
pub mod events;
pub mod hardware;
pub mod shutdown;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, State,
};

pub use hardware::{GuardStatus, SelfTestExecutionResult, TelemetryData};
pub use shutdown::ShutdownPolicy;

pub struct AppState {
    pub telemetry: Mutex<TelemetryData>,
    pub shutdown_mgr: Mutex<shutdown::ShutdownManager>,
    pub events_mgr: Mutex<events::EventManager>,
    pub energy_tracker: Mutex<energy::EnergyTracker>,
    pub last_status: Mutex<String>,
    pub beeper_enabled: AtomicBool,
    pub close_to_tray: AtomicBool,
}

pub fn is_autostart_enabled() -> bool {
    #[cfg(windows)]
    {
        let mut cmd = std::process::Command::new("reg");
        cmd.args([
            "query",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            "/v",
            "ApcPowerConsole",
        ]);
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        match cmd.output() {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }
    #[cfg(not(windows))]
    {
        false
    }
}

pub fn set_autostart_enabled(enable: bool) -> Result<bool, String> {
    #[cfg(windows)]
    {
        if enable {
            let exe = std::env::current_exe()
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .to_string();
            let val = format!("\"{}\" --minimized", exe);
            let mut cmd = std::process::Command::new("reg");
            cmd.args([
                "add",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "ApcPowerConsole",
                "/t",
                "REG_SZ",
                "/d",
                &val,
                "/f",
            ]);
            cmd.creation_flags(0x08000000);
            let status = cmd.status().map_err(|e| e.to_string())?;
            if status.success() {
                Ok(true)
            } else {
                Err("Failed to set Windows autostart registry".to_string())
            }
        } else {
            let mut cmd = std::process::Command::new("reg");
            cmd.args([
                "delete",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "ApcPowerConsole",
                "/f",
            ]);
            cmd.creation_flags(0x08000000);
            let _ = cmd.status();
            Ok(false)
        }
    }
    #[cfg(not(windows))]
    {
        Ok(false)
    }
}

#[tauri::command]
fn get_autostart() -> bool {
    is_autostart_enabled()
}

#[tauri::command]
fn set_autostart(enabled: bool) -> Result<bool, String> {
    set_autostart_enabled(enabled)
}

#[tauri::command]
fn get_telemetry(state: State<AppState>) -> TelemetryData {
    state.telemetry.lock().unwrap().clone()
}

#[tauri::command]
fn get_policy(state: State<AppState>) -> ShutdownPolicy {
    state.shutdown_mgr.lock().unwrap().policy.clone()
}

#[tauri::command]
fn save_policy(
    new_policy: ShutdownPolicy,
    state: State<AppState>,
) -> Result<ShutdownPolicy, String> {
    let mut policy = new_policy;
    policy.validate()?;
    policy.save_policy()?;

    // Sync atomic close_to_tray immediately for lock-free CloseRequested checks
    state
        .close_to_tray
        .store(policy.close_to_tray, Ordering::SeqCst);

    let mut mgr = state.shutdown_mgr.lock().unwrap();
    mgr.policy = policy.clone();
    Ok(policy)
}

#[tauri::command]
fn reset_policy(state: State<AppState>) -> ShutdownPolicy {
    let def = ShutdownPolicy::default();
    let _ = def.save_policy();
    state
        .close_to_tray
        .store(def.close_to_tray, Ordering::SeqCst);

    let mut mgr = state.shutdown_mgr.lock().unwrap();
    mgr.policy = def.clone();
    def
}

#[tauri::command]
fn abort_shutdown(state: State<AppState>) -> Result<String, String> {
    let mut mgr = state.shutdown_mgr.lock().unwrap();
    let aborted = mgr.abort_shutdown();
    if aborted {
        state.events_mgr.lock().unwrap().record(
            "INFO",
            "ABORT",
            "Automated protection shutdown sequence was cancelled by user.",
            None,
        );
        Ok("Automated shutdown/hibernation procedure was safely cancelled.".to_string())
    } else {
        Ok("No scheduled shutdown is currently active.".to_string())
    }
}

#[tauri::command]
fn set_beeper(enabled: bool, state: State<AppState>) -> Result<bool, String> {
    state.beeper_enabled.store(enabled, Ordering::SeqCst);
    let mut t = state.telemetry.lock().unwrap();
    t.beeper_enabled = enabled;
    Ok(enabled)
}

#[tauri::command]
fn run_selftest(state: State<AppState>) -> Result<hardware::SelfTestExecutionResult, String> {
    // Flag shutdown manager so emergency blackout logic does not interfere during diagnostics
    {
        let mut mgr = state.shutdown_mgr.lock().unwrap();
        mgr.selftest_active = true;
    }

    state.events_mgr.lock().unwrap().record(
        "INFO",
        "SELF_TEST_START",
        "Initiating hardware diagnostic self-test cycle via USB Feature Report 33.",
        None,
    );

    let res = hardware::run_hardware_selftest();

    {
        let mut mgr = state.shutdown_mgr.lock().unwrap();
        mgr.selftest_active = false;
    }

    match &res {
        Ok(r) => {
            let sev = if r.success { "INFO" } else { "CRITICAL" };
            state.events_mgr.lock().unwrap().record(
                sev,
                "SELF_TEST_RESULT",
                &format!("Diagnostic Self-Test {}: {}", r.result_str, r.message),
                Some(&format!("Duration: {:.1}s", r.duration_secs)),
            );
            let win_type = if r.success { "Information" } else { "Error" };
            shutdown::log_windows_event(
                win_type,
                1005,
                &format!("Diagnostic Self-Test {}: {}", r.result_str, r.message),
            );
        }
        Err(e) => {
            state.events_mgr.lock().unwrap().record(
                "WARNING",
                "SELF_TEST_ERROR",
                &format!("Self-test could not be executed: {}", e),
                None,
            );
        }
    }

    res
}

#[tauri::command]
fn get_events(
    limit: Option<usize>,
    severity: Option<String>,
    search: Option<String>,
    state: State<AppState>,
) -> Vec<events::PowerEvent> {
    state.events_mgr.lock().unwrap().query(limit, severity, search)
}

#[tauri::command]
fn clear_events(state: State<AppState>) -> Result<(), String> {
    state.events_mgr.lock().unwrap().clear()
}

#[tauri::command]
fn export_events_csv(state: State<AppState>) -> Result<String, String> {
    Ok(state.events_mgr.lock().unwrap().export_csv())
}

#[tauri::command]
fn get_energy_analytics(state: State<AppState>) -> energy::EnergyAnalytics {
    let rate = state.shutdown_mgr.lock().unwrap().policy.kwh_rate_idr;
    state.energy_tracker.lock().unwrap().get_analytics(rate)
}

#[tauri::command]
fn reset_session_peaks(state: State<AppState>) -> Result<(), String> {
    let current_linev = state.telemetry.lock().unwrap().linev;
    state.energy_tracker.lock().unwrap().reset_session_peaks(current_linev);
    Ok(())
}

pub fn create_guard_status(mgr: &shutdown::ShutdownManager) -> GuardStatus {
    let remaining = if mgr.in_countdown {
        let elapsed = mgr
            .countdown_start
            .map(|s| s.elapsed().as_secs())
            .unwrap_or(0);
        mgr.policy.shutdown_grace_secs.saturating_sub(elapsed)
    } else {
        0
    };
    let action_upper = mgr.policy.action.to_uppercase();
    let status = if mgr.in_countdown {
        format!("COUNTDOWN ({}s)", remaining)
    } else if mgr.policy.enabled {
        format!("ARMED ({})", action_upper)
    } else {
        "DISABLED".to_string()
    };
    let outage_secs = mgr
        .blackout_start
        .map(|s| s.elapsed().as_secs())
        .unwrap_or(0);

    GuardStatus {
        enabled: mgr.policy.enabled,
        dry_run: mgr.policy.dry_run,
        action: mgr.policy.action.clone(),
        status,
        countdown_active: mgr.in_countdown,
        countdown_remaining: remaining,
        countdown_reason: mgr.trigger_reason.clone(),
        min_battery: mgr.policy.min_battery_pct,
        min_runtime: mgr.policy.min_runtime_mins,
        outage_secs,
        last_reason: mgr.policy.last_trigger_reason.clone(),
    }
}

pub fn run() {
    let mgr = shutdown::ShutdownManager::new();
    let initial_close_to_tray = mgr.policy.close_to_tray;
    let events_mgr = events::EventManager::load();
    let energy_tracker = energy::EnergyTracker::load();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            telemetry: Mutex::new(hardware::TelemetryData::default()),
            shutdown_mgr: Mutex::new(mgr),
            events_mgr: Mutex::new(events_mgr),
            energy_tracker: Mutex::new(energy_tracker),
            last_status: Mutex::new("ONLINE".to_string()),
            beeper_enabled: AtomicBool::new(true),
            close_to_tray: AtomicBool::new(initial_close_to_tray),
        })
        .invoke_handler(tauri::generate_handler![
            get_telemetry,
            get_policy,
            save_policy,
            reset_policy,
            abort_shutdown,
            set_beeper,
            get_autostart,
            set_autostart,
            run_selftest,
            get_events,
            clear_events,
            export_events_csv,
            get_energy_analytics,
            reset_session_peaks,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<AppState>();
                let should_minimize = state.close_to_tray.load(Ordering::SeqCst);
                if should_minimize {
                    api.prevent_close();
                    let _ = window.hide();
                } else {
                    let _ = state.energy_tracker.lock().unwrap().save();
                    let _ = state.events_mgr.lock().unwrap().save();
                    window.app_handle().exit(0);
                }
            }
        })
        .setup(|app| {
            let is_minimized = std::env::args()
                .any(|a| a == "--minimized" || a == "--silent" || a == "--background");
            if is_minimized {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            let state = app.state::<AppState>();
            let close_to_tray_active = state.close_to_tray.load(Ordering::SeqCst);

            let autostart_txt = if is_autostart_enabled() {
                "[✓] Start on Windows Boot"
            } else {
                "[ ] Start on Windows Boot"
            };

            let close_tray_txt = if close_to_tray_active {
                "[✓] Close to Tray (Background)"
            } else {
                "[ ] Close to Tray (Background)"
            };

            let open_i = MenuItem::with_id(app, "open", "⚡ Open Dashboard", true, None::<&str>)?;
            let status_i = MenuItem::with_id(
                app,
                "status",
                "Status: Online (Monitoring)",
                false,
                None::<&str>,
            )?;
            let close_tray_i =
                MenuItem::with_id(app, "toggle_close_tray", close_tray_txt, true, None::<&str>)?;
            let autostart_i =
                MenuItem::with_id(app, "autostart", autostart_txt, true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "❌ Exit", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[&open_i, &status_i, &close_tray_i, &autostart_i, &quit_i],
            )?;

            let autostart_item = autostart_i.clone();
            let close_tray_item = close_tray_i.clone();

            let tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("APC Universal Power Console")
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "toggle_close_tray" => {
                        let state = app.state::<AppState>();
                        let current = state.close_to_tray.load(Ordering::SeqCst);
                        let next = !current;
                        state.close_to_tray.store(next, Ordering::SeqCst);
                        {
                            let mut mgr = state.shutdown_mgr.lock().unwrap();
                            mgr.policy.close_to_tray = next;
                            let _ = mgr.policy.save_policy();
                        }
                        let new_txt = if next {
                            "[✓] Close to Tray (Background)"
                        } else {
                            "[ ] Close to Tray (Background)"
                        };
                        let _ = close_tray_item.set_text(new_txt);
                    }
                    "autostart" => {
                        let next = !is_autostart_enabled();
                        let _ = set_autostart_enabled(next);
                        let new_txt = if next {
                            "[✓] Start on Windows Boot"
                        } else {
                            "[ ] Start on Windows Boot"
                        };
                        let _ = autostart_item.set_text(new_txt);
                    }
                    "quit" => {
                        let state = app.state::<AppState>();
                        let _ = state.energy_tracker.lock().unwrap().save();
                        let _ = state.events_mgr.lock().unwrap().save();
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    }
                    | TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    } => {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;

            // Background Polling Thread (1.2s cadence)
            let handle = app.handle().clone();
            let status_item = status_i.clone();
            let tray_handle = tray.clone();

            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(1200));
                    let state = handle.state::<AppState>();
                    let beeper = state.beeper_enabled.load(Ordering::SeqCst);
                    let mut data = hardware::read_telemetry(beeper);

                    // Scope mutex for evaluate & energy integration
                    let (guard_status, rate) = {
                        let mut mgr = state.shutdown_mgr.lock().unwrap();
                        let trigger_notice = mgr.evaluate(&data);
                        let status = create_guard_status(&mgr);
                        let rate = mgr.policy.kwh_rate_idr;

                        if let Some(ref notice) = trigger_notice {
                            if notice.contains("Countdown") {
                                state.events_mgr.lock().unwrap().record(
                                    "CRITICAL",
                                    "COUNTDOWN",
                                    notice,
                                    None,
                                );
                            } else if notice.contains("pulih") || notice.contains("restored") {
                                state.events_mgr.lock().unwrap().record(
                                    "INFO",
                                    "GRID_RESTORE",
                                    notice,
                                    None,
                                );
                            }
                        }

                        (status, rate)
                    };
                    data.guard = guard_status;

                    // Real-time energy integration
                    if data.connected {
                        let mut et = state.energy_tracker.lock().unwrap();
                        et.integrate(data.loadw, data.linev, rate, 1.2);
                    }

                    // Track power transitions (ONLINE <-> ON BATTERY)
                    {
                        let mut last = state.last_status.lock().unwrap();
                        if *last != data.status {
                            if data.status == "ON BATTERY" {
                                state.events_mgr.lock().unwrap().record(
                                    "WARNING",
                                    "BLACKOUT",
                                    &format!(
                                        "Utility grid failure. Operating on battery reserve ({:.0}%).",
                                        data.bcharge
                                    ),
                                    Some(&data.last_transfer_reason),
                                );
                                shutdown::log_windows_event(
                                    "Warning",
                                    1001,
                                    &format!(
                                        "APC UPS: Utility grid failure. Operating on battery ({:.0}%). Cause: {}",
                                        data.bcharge, data.last_transfer_reason
                                    ),
                                );
                            } else if data.status == "ONLINE" && *last == "ON BATTERY" {
                                state.events_mgr.lock().unwrap().record(
                                    "INFO",
                                    "GRID_RESTORE",
                                    &format!(
                                        "Utility power restored ({:.1} VAC). Normal operation resumed.",
                                        data.linev
                                    ),
                                    None,
                                );
                                shutdown::log_windows_event(
                                    "Information",
                                    1002,
                                    &format!(
                                        "APC UPS: Utility power restored ({:.1} VAC). Normal operation resumed.",
                                        data.linev
                                    ),
                                );
                            }
                            *last = data.status.clone();
                        }
                    }

                    // Automated Scheduled Self-Test Cron (e.g. 30-day interval with 4 safety interlocks)
                    let should_run_cron = {
                        let mgr = state.shutdown_mgr.lock().unwrap();
                        let interval = mgr.policy.selftest_interval_days;
                        if interval > 0
                            && mgr.policy.enabled
                            && !mgr.selftest_active
                            && !mgr.in_countdown
                        {
                            let now = chrono::Local::now().timestamp() as u64;
                            let last = mgr.policy.last_selftest_timestamp;
                            let due = now.saturating_sub(last) >= (interval as u64 * 86400);

                            // 4 Safety Interlocks:
                            // 1. Grid online & stable
                            // 2. Battery >= 85%
                            // 3. Load <= 75%
                            // 4. Blackout not active
                            let safe = data.status == "ONLINE"
                                && data.linev >= 200.0
                                && data.bcharge >= 85.0
                                && data.loadpct <= 75.0
                                && mgr.blackout_start.is_none();

                            due && safe
                        } else {
                            false
                        }
                    };

                    if should_run_cron {
                        {
                            let mut mgr = state.shutdown_mgr.lock().unwrap();
                            mgr.policy.last_selftest_timestamp =
                                chrono::Local::now().timestamp() as u64;
                            let _ = mgr.policy.save_policy();
                            mgr.selftest_active = true;
                        }

                        state.events_mgr.lock().unwrap().record(
                            "INFO",
                            "CRON_SELFTEST",
                            "Executing scheduled automated battery self-test (30-day preventive maintenance).",
                            None,
                        );

                        let res = hardware::run_hardware_selftest();

                        {
                            let mut mgr = state.shutdown_mgr.lock().unwrap();
                            mgr.selftest_active = false;
                        }

                        if let Ok(r) = res {
                            let sev = if r.success { "INFO" } else { "CRITICAL" };
                            state.events_mgr.lock().unwrap().record(
                                sev,
                                "CRON_SELFTEST_RESULT",
                                &format!("Automated self-test {}: {}", r.result_str, r.message),
                                Some(&format!("Duration: {:.1}s", r.duration_secs)),
                            );
                            let win_type = if r.success { "Information" } else { "Error" };
                            shutdown::log_windows_event(
                                win_type,
                                1005,
                                &format!("Automated Self-Test: {}", r.result_str),
                            );
                        }
                    }

                    // Populate events count from persistent manager
                    data.events = state.events_mgr.lock().unwrap().events.len() as u32;

                    // Update System Tray readout text and tooltip (No locks held here!)
                    let readout = format!(
                        "Status: {} ({:.0}% | {:.0}W)",
                        data.status, data.bcharge, data.loadw
                    );
                    let _ = status_item.set_text(&readout);
                    let _ = tray_handle.set_tooltip(Some(format!(
                        "APC Console: {} - {:.0}%",
                        data.status, data.bcharge
                    )));

                    *state.telemetry.lock().unwrap() = data;
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
