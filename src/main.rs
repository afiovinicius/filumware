mod core;
mod utils;

use anyhow::Result;
use core::pipeline::{pipe_hardware, pipe_storage};
use ratatui::style::Color;
use utils::ui::{TuiApp, render_error, render_success};

use crate::utils::ui::render_alert;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = TuiApp::new();

    match pipe_hardware() {
        Ok(info) => {
            app.push_log(render_success("Privilégios de root validados com sucesso."));
            app.push_log(render_success(&format!(
                "Interface Wi-Fi ativa detectada: {}",
                info.network_interface
            )));
            app.push_log(format!(
                "\n • OS: {}\n • CPU: {}\n • GPU: {}\n • RAM: {}",
                info.os, info.cpu, info.gpu, info.ram
            ));
        }
        Err(e) => {
            app.push_log(render_error(&format!(
                "Falha na pipeline de hardware: {}",
                e
            )));
            app.set_border_color(Color::Red);
            app.run()?;
            return Ok(());
        }
    }

    match pipe_storage().await {
        Ok(Some(target)) => {
            app.push_log(render_success(
                "Alvo selecionado e inicializado com sucesso!",
            ));
            app.push_log(format!(" • SSID       : {}", target.ssid));
            app.push_log(format!(" • BSSID      : {}", target.bssid));
            app.push_log(format!(" • MAC Address: {}", target.mac_address));
            app.push_log(format!(
                " • Canal      : {} ({} MHz)",
                target.channel, target.frequency
            ));
            app.push_log(format!(" • Segurança  : {}", target.security));
            app.push_log(format!(" • RSN        : {}", target.rsn));
            app.push_log(format!(
                " • WPS Ativo  : {}",
                if target.wps_active {
                    "Sim [✔]"
                } else {
                    "Não [✖]"
                }
            ));
            app.push_log(format!(" • Diretório  : {}", target.target_dir));
            app.set_border_color(Color::LightCyan);
        }
        Ok(None) => {
            app.push_log(render_alert(
                "Nenhuma rede foi selecionada (operação encerrada pelo usuário).",
            ));
            app.set_border_color(Color::Yellow);
        }
        Err(e) => {
            app.push_log(render_error(&format!(
                "Falha na execução da pipeline de storage: {}",
                e
            )));
            app.set_border_color(Color::Red);
        }
    }

    app.run()?;
    Ok(())
}
