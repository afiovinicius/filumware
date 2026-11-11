use crate::core::variables::AllowedExtension;
use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::{self, Read};
use std::path::PathBuf;

pub fn create_folder(bssid: &str) -> io::Result<PathBuf> {
    let sanitized_bssid = bssid.replace(':', "");
    let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();

    let folder_name = format!("{}-{}", sanitized_bssid, timestamp);
    let mut path = PathBuf::from("attacks");
    path.push(folder_name);

    fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn save_file<T: serde::Serialize>(
    dir_path: &PathBuf,
    file_name: &str,
    extension: AllowedExtension,
    data: &T,
) -> io::Result<()> {
    if !dir_path.exists() || !dir_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Pasta de destino inválida ou não existe.",
        ));
    }

    let full_filename = format!("{}{}", file_name, extension.as_str());
    let file_path = dir_path.join(full_filename);

    if file_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "Erro crítico: O arquivo '{:?}' já existe! Abortando criação para evitar sobrescrita.",
                file_path
            ),
        ));
    }

    match extension {
        AllowedExtension::Bpf => {}
        AllowedExtension::Json => {
            let json_data = serde_json::to_string_pretty(data)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            fs::write(&file_path, json_data)?;
        }
        AllowedExtension::Txt | AllowedExtension::Rule => {
            let content = serde_json::to_string(data).unwrap_or_else(|_| "".to_string());
            fs::write(&file_path, content.trim_matches('"'))?;
        }
        AllowedExtension::Pcap | AllowedExtension::Pcapng => {
            let content = serde_json::to_string(data).unwrap_or_default();
            fs::write(&file_path, content)?;
        }
    }

    println!("[+] Arquivo salvo com sucesso: {:?}", file_path);
    Ok(())
}

pub fn read_file(
    dir_path: &PathBuf,
    file_name: &str,
    extension: AllowedExtension,
) -> io::Result<String> {
    let full_name = format!("{}{}", file_name, extension.as_str());
    let file_path = dir_path.join(full_name);

    let mut file = OpenOptions::new().read(true).open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

pub fn update_file(
    dir_path: &PathBuf,
    file_name: &str,
    extension: AllowedExtension,
    new_content: &str,
) -> io::Result<()> {
    let full_name = format!("{}{}", file_name, extension.as_str());
    let file_path = dir_path.join(full_name);

    if !file_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Arquivo não encontrado para atualização.",
        ));
    }

    fs::write(file_path, new_content)?;
    Ok(())
}

pub fn delete_file(
    dir_path: &PathBuf,
    file_name: &str,
    extension: AllowedExtension,
) -> io::Result<()> {
    let full_name = format!("{}{}", file_name, extension.as_str());
    let file_path = dir_path.join(full_name);

    if file_path.exists() {
        fs::remove_file(file_path)?;
    }
    Ok(())
}

pub fn find_file_in_target(
    dir_path: &PathBuf,
    file_name: &str,
    extension: AllowedExtension,
) -> Option<PathBuf> {
    let full_name = format!("{}{}", file_name, extension.as_str());
    let path = dir_path.join(full_name);
    if path.exists() { Some(path) } else { None }
}
