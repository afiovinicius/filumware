use anyhow::{Result, anyhow};
use std::process::Command;

use crate::core::variables::TableNetworkRow;

pub fn run_command(cmd: &str) -> Result<String> {
    let output = Command::new("sh").arg("-c").arg(cmd).output()?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(result)
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(anyhow!("Falha ao executar comando [{}]: {}", cmd, err))
    }
}

pub fn check_root_privileges() -> bool {
    unsafe { libc::geteuid() == 0 }
}

pub fn format_bssid(raw: &str) -> String {
    let cleaned: String = raw.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if cleaned.len() == 12 {
        let mut formatted = String::new();
        for (i, c) in cleaned.chars().enumerate() {
            if i > 0 && i % 2 == 0 {
                formatted.push(':');
            }
            formatted.push(c.to_ascii_uppercase());
        }
        formatted
    } else {
        raw.trim().to_uppercase()
    }
}

pub fn calculate_rssi(signal_pct: i32) -> i32 {
    (signal_pct / 2) - 100
}

pub fn format_security(sec: &str) -> String {
    let s = sec.trim();
    if s.is_empty() || s == "--" {
        "Aberta / None".to_string()
    } else {
        s.to_string()
    }
}

pub fn format_rsn(sec: &str) -> String {
    let upper = sec.to_uppercase();
    if upper.contains("WPA2") || upper.contains("WPA3") || upper.contains("RSN") {
        "RSN IE 802.11i (Ativo)".to_string()
    } else if upper.contains("WPA") {
        "WPA IE (Legado)".to_string()
    } else {
        "Nenhum / Aberto".to_string()
    }
}

pub fn parse_nmcli_output(output: &str) -> Vec<TableNetworkRow> {
    let mut rows = Vec::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.split(':').collect();

        if parts.len() >= 11 {
            let raw_bssid = parts[0..6].join(":");
            let bssid = format_bssid(&raw_bssid);

            let ssid = parts[6].to_string();
            let ssid_formatted = if ssid.is_empty() {
                "<Rede Oculta>".to_string()
            } else {
                ssid
            };

            let signal_pct: i32 = parts[7].parse().unwrap_or(0);
            let rssi_val = calculate_rssi(signal_pct);
            let rssi_str = format!("{} dBm", rssi_val);

            let band = parts[8].to_string();
            let channel = parts[9].to_string();

            let security_raw = parts[10..].join(":");
            let security = format_security(&security_raw);

            rows.push(TableNetworkRow {
                ssid: ssid_formatted,
                bssid,
                rssi: rssi_str,
                channel,
                band,
                security,
            });
        }
    }

    rows
}

pub fn valid_interface(interface: &str) -> anyhow::Result<()> {
    let cmd = format!(
        "nmcli -g GENERAL.STATE device show {} | awk '{{print $2}}' | tr -d '()'",
        interface
    );

    let state = run_command(&cmd)?;

    if state == "unavailable" || state.is_empty() {
        return Err(anyhow::anyhow!(
            "Aviso: A interface ({}) não está pronta. Por favor, ative a interface.",
            interface
        ));
    }
    Ok(())
}
