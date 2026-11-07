use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyEnergy {
    pub date: String, // "YYYY-MM-DD"
    pub kwh: f64,
    pub cost_idr: f64,
    pub peak_watts: f64,
    pub runtime_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnergyAnalytics {
    pub lifetime_kwh: f64,
    pub lifetime_cost_idr: f64,
    pub today_kwh: f64,
    pub today_cost_idr: f64,
    pub today_peak_w: f64,
    pub month_kwh: f64,
    pub month_cost_idr: f64,
    pub session_peak_w: f64,
    pub session_min_v: f64,
    pub session_max_v: f64,
    pub history_days: Vec<DailyEnergy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyTracker {
    pub lifetime_kwh: f64,
    pub current_day: String,
    pub daily_records: Vec<DailyEnergy>,
    #[serde(default)]
    pub session_peak_w: f64,
    #[serde(default)]
    pub session_min_v: f64,
    #[serde(default)]
    pub session_max_v: f64,
    #[serde(skip)]
    pub last_save_time: f64,
}

pub fn get_energy_path() -> PathBuf {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_app_data)
            .join("ApcPowerConsole")
            .join("energy_tracker.json")
    } else if let Ok(user_profile) = std::env::var("USERPROFILE") {
        PathBuf::from(user_profile)
            .join("AppData")
            .join("Local")
            .join("ApcPowerConsole")
            .join("energy_tracker.json")
    } else {
        PathBuf::from("energy_tracker.json")
    }
}

impl Default for EnergyTracker {
    fn default() -> Self {
        Self::load()
    }
}

impl EnergyTracker {
    pub fn new() -> Self {
        let today = Local::now().format("%Y-%m-%d").to_string();
        Self {
            lifetime_kwh: 0.0,
            current_day: today,
            daily_records: Vec::new(),
            session_peak_w: 0.0,
            session_min_v: 230.0,
            session_max_v: 230.0,
            last_save_time: 0.0,
        }
    }

    pub fn load() -> Self {
        let path = get_energy_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(mut tracker) = serde_json::from_str::<EnergyTracker>(&content) {
                    if tracker.session_min_v <= 0.0 {
                        tracker.session_min_v = 230.0;
                    }
                    if tracker.session_max_v <= 0.0 {
                        tracker.session_max_v = 230.0;
                    }
                    return tracker;
                }
            }
        }
        Self::new()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = get_energy_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize energy tracker: {}", e))?;

        let tmp_path = path.with_extension("tmp");
        std::fs::write(&tmp_path, json)
            .map_err(|e| format!("Failed to write tmp energy file: {}", e))?;

        std::fs::rename(&tmp_path, &path)
            .map_err(|e| format!("Failed to replace energy file: {}", e))?;

        Ok(())
    }

    /// Integrates load wattage over time interval delta_secs (typically 1.2s).
    /// Auto-persists to disk if > 60 seconds have elapsed since last save.
    pub fn integrate(&mut self, loadw: f64, linev: f64, rate_idr: f64, delta_secs: f64) {
        if !loadw.is_finite() || loadw < 0.0 || delta_secs <= 0.0 {
            return;
        }

        let delta_kwh = (loadw * delta_secs) / 3_600_000.0;
        let delta_cost = delta_kwh * rate_idr;

        self.lifetime_kwh += delta_kwh;

        // Session extreme tracking
        if loadw > self.session_peak_w {
            self.session_peak_w = loadw;
        }
        if linev > 50.0 {
            if linev < self.session_min_v || self.session_min_v == 230.0 {
                self.session_min_v = linev;
            }
            if linev > self.session_max_v || self.session_max_v == 230.0 {
                self.session_max_v = linev;
            }
        }

        let today = Local::now().format("%Y-%m-%d").to_string();

        if let Some(record) = self.daily_records.iter_mut().find(|r| r.date == today) {
            record.kwh += delta_kwh;
            record.cost_idr += delta_cost;
            record.runtime_secs += delta_secs.round() as u64;
            if loadw > record.peak_watts {
                record.peak_watts = loadw;
            }
        } else {
            self.daily_records.insert(
                0,
                DailyEnergy {
                    date: today.clone(),
                    kwh: delta_kwh,
                    cost_idr: delta_cost,
                    peak_watts: loadw,
                    runtime_secs: delta_secs.round() as u64,
                },
            );
            // Retain up to 90 days of history
            if self.daily_records.len() > 90 {
                self.daily_records.truncate(90);
            }
        }

        self.current_day = today;

        // Save periodically (every 60 seconds)
        let now_sec = chrono::Local::now().timestamp() as f64;
        if (now_sec - self.last_save_time) > 60.0 {
            self.last_save_time = now_sec;
            let _ = self.save();
        }
    }

    pub fn get_analytics(&self, rate_idr: f64) -> EnergyAnalytics {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let current_month = Local::now().format("%Y-%m").to_string();

        let today_record = self.daily_records.iter().find(|r| r.date == today);
        let today_kwh = today_record.map(|r| r.kwh).unwrap_or(0.0);
        let today_cost_idr = today_record.map(|r| r.cost_idr).unwrap_or(today_kwh * rate_idr);
        let today_peak_w = today_record.map(|r| r.peak_watts).unwrap_or(0.0);

        let mut month_kwh = 0.0;
        let mut month_cost_idr = 0.0;

        for r in &self.daily_records {
            if r.date.starts_with(&current_month) {
                month_kwh += r.kwh;
                month_cost_idr += r.cost_idr;
            }
        }

        let lifetime_cost_idr = self.lifetime_kwh * rate_idr;

        EnergyAnalytics {
            lifetime_kwh: (self.lifetime_kwh * 1000.0).round() / 1000.0,
            lifetime_cost_idr: lifetime_cost_idr.round(),
            today_kwh: (today_kwh * 1000.0).round() / 1000.0,
            today_cost_idr: today_cost_idr.round(),
            today_peak_w: (today_peak_w * 10.0).round() / 10.0,
            month_kwh: (month_kwh * 100.0).round() / 100.0,
            month_cost_idr: month_cost_idr.round(),
            session_peak_w: (self.session_peak_w * 10.0).round() / 10.0,
            session_min_v: (self.session_min_v * 10.0).round() / 10.0,
            session_max_v: (self.session_max_v * 10.0).round() / 10.0,
            history_days: self.daily_records.iter().take(14).cloned().collect(),
        }
    }

    pub fn reset_session_peaks(&mut self, current_linev: f64) {
        self.session_peak_w = 0.0;
        self.session_min_v = if current_linev > 50.0 { current_linev } else { 230.0 };
        self.session_max_v = if current_linev > 50.0 { current_linev } else { 230.0 };
        let _ = self.save();
    }
}
