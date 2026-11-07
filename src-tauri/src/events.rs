use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PowerEvent {
    pub id: u64,
    pub timestamp: f64,
    pub time_str: String,
    pub severity: String,   // "INFO", "WARNING", "CRITICAL"
    pub event_type: String, // "GRID_LOSS", "GRID_RESTORE", "AVR_BOOST", "AVR_TRIM", "SELF_TEST", "COUNTDOWN", "PROTECTION", "SYSTEM"
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventManager {
    pub events: Vec<PowerEvent>,
    pub next_id: u64,
}

impl Default for EventManager {
    fn default() -> Self {
        Self::load()
    }
}

pub fn get_events_path() -> PathBuf {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_app_data)
            .join("ApcPowerConsole")
            .join("event_history.json")
    } else if let Ok(user_profile) = std::env::var("USERPROFILE") {
        PathBuf::from(user_profile)
            .join("AppData")
            .join("Local")
            .join("ApcPowerConsole")
            .join("event_history.json")
    } else {
        PathBuf::from("event_history.json")
    }
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            next_id: 1,
        }
    }

    /// Load event history from disk (%LOCALAPPDATA%\ApcPowerConsole\event_history.json)
    pub fn load() -> Self {
        let path = get_events_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(mgr) = serde_json::from_str::<EventManager>(&content) {
                    return mgr;
                }
            }
        }
        Self::new()
    }

    /// Save event history to disk with atomic write pattern (.tmp -> rename)
    pub fn save(&self) -> Result<(), String> {
        let path = get_events_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize events: {}", e))?;

        let tmp_path = path.with_extension("tmp");
        std::fs::write(&tmp_path, json)
            .map_err(|e| format!("Failed to write tmp events file: {}", e))?;

        std::fs::rename(&tmp_path, &path)
            .map_err(|e| format!("Failed to replace events file: {}", e))?;

        Ok(())
    }

    /// Add a new power event and automatically persist to disk.
    /// Retains maximum 500 events (FIFO circular buffer).
    pub fn record(
        &mut self,
        severity: &str,
        event_type: &str,
        message: &str,
        details: Option<&str>,
    ) -> PowerEvent {
        let now = Local::now();
        let time_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let timestamp = now.timestamp() as f64;

        let ev = PowerEvent {
            id: self.next_id,
            timestamp,
            time_str,
            severity: severity.to_uppercase(),
            event_type: event_type.to_uppercase(),
            message: message.to_string(),
            details: details.map(|s| s.to_string()),
        };

        self.next_id += 1;
        self.events.insert(0, ev.clone()); // newest first

        if self.events.len() > 500 {
            self.events.truncate(500);
        }

        let _ = self.save();
        ev
    }

    /// Query events with optional filter and search keyword
    pub fn query(
        &self,
        limit: Option<usize>,
        severity_filter: Option<String>,
        search_query: Option<String>,
    ) -> Vec<PowerEvent> {
        let sev_clean = severity_filter.map(|s| s.trim().to_uppercase());
        let q_clean = search_query.map(|s| s.trim().to_lowercase());

        self.events
            .iter()
            .filter(|ev| {
                if let Some(ref sev) = sev_clean {
                    if sev != "ALL" && !sev.is_empty() && ev.severity != *sev {
                        return false;
                    }
                }
                if let Some(ref q) = q_clean {
                    if !q.is_empty() {
                        let matches_msg = ev.message.to_lowercase().contains(q);
                        let matches_type = ev.event_type.to_lowercase().contains(q);
                        let matches_time = ev.time_str.to_lowercase().contains(q);
                        let matches_det = ev
                            .details
                            .as_ref()
                            .map(|d| d.to_lowercase().contains(q))
                            .unwrap_or(false);
                        return matches_msg || matches_type || matches_time || matches_det;
                    }
                }
                true
            })
            .take(limit.unwrap_or(500))
            .cloned()
            .collect()
    }

    /// Clear all events and persist empty list to disk
    pub fn clear(&mut self) -> Result<(), String> {
        self.events.clear();
        self.save()
    }

    /// Generate RFC 4180 compliant CSV export
    pub fn export_csv(&self) -> String {
        let mut csv = String::from("\"ID\",\"Timestamp\",\"Severity\",\"Event Type\",\"Description\",\"Technical Details\"\r\n");
        for ev in &self.events {
            let desc_escaped = ev.message.replace('\"', "\"\"");
            let details_escaped = ev
                .details
                .as_deref()
                .unwrap_or("")
                .replace('\"', "\"\"");

            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\r\n",
                ev.id, ev.time_str, ev.severity, ev.event_type, desc_escaped, details_escaped
            ));
        }
        csv
    }
}
