use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalStates {
    pub pwd: String,
    pub folder_save: String,
    pub target_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserHardwareInfo {
    pub cpu: String,
    pub gpu: String,
    pub ram: String,
    pub os: String,
    pub network_interface: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkTarget {
    pub ssid: String,
    pub bssid: String,
    pub mac_address: String,
    pub rssi: i32,
    pub rsn: String,
    pub frequency: u32,
    pub channel: u32,
    pub security: String,
    pub vendor_oui: String,
    pub wps_active: bool,
    pub target_dir: String,
    pub pmkid_captured: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TableNetworkRow {
    pub ssid: String,
    pub bssid: String,
    pub rssi: String,
    pub channel: String,
    pub band: String,
    pub security: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllowedExtension {
    Pcap,
    Pcapng,
    Txt,
    Rule,
    Json,
    Bpf,
}

impl AllowedExtension {
    pub fn as_str(&self) -> &'static str {
        match self {
            AllowedExtension::Pcap => ".pcap",
            AllowedExtension::Pcapng => ".pcapng",
            AllowedExtension::Txt => ".txt",
            AllowedExtension::Rule => ".rule",
            AllowedExtension::Json => ".json",
            AllowedExtension::Bpf => ".bpf",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pcap" | ".pcap" => Some(Self::Pcap),
            "pcapng" | ".pcapng" => Some(Self::Pcapng),
            "txt" | ".txt" => Some(Self::Txt),
            "rule" | ".rule" => Some(Self::Rule),
            "json" | ".json" => Some(Self::Json),
            "bpf" | ".bpf" => Some(Self::Bpf),
            _ => None,
        }
    }
}
