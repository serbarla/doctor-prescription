use crate::api;

pub fn template_status_label(status: &str) -> &'static str {
    match status {
        "draft" => "Borrador",
        "published" => "Publicada",
        "archived" => "Archivada",
        _ => "Desconocido",
    }
}

pub fn version_status_label(status: &str) -> &'static str {
    match status {
        "draft" => "Borrador",
        "published" => "Publicada",
        "archived" => "Archivada",
        _ => "Desconocido",
    }
}

pub fn template_owner_label(owner_type: &str) -> &'static str {
    match owner_type {
        "doctor" => "Plantilla del médico",
        "clinic" => "Maestra de la clínica",
        _ => "Plantilla",
    }
}

pub fn optional_date(value: Option<&str>) -> String {
    match value {
        Some(date) => format_date(date),
        None => "—".to_string(),
    }
}

pub fn sex_label(sex: Option<&str>) -> &'static str {
    match sex {
        Some("female") => "Femenino",
        Some("male") => "Masculino",
        Some("other") => "Otro",
        Some("unspecified") => "Prefiere no decir",
        _ => "Sin especificar",
    }
}

pub fn role_label(role: Option<&str>) -> &'static str {
    match role {
        Some("administrator") => "Administrador",
        Some("doctor") => "Doctor",
        Some("nursing") => "Enfermería",
        Some("secretary") => "Secretaría",
        _ => "Sin rol",
    }
}

pub fn platform_label(platform: &str) -> &'static str {
    match platform {
        "macos" => "macOS",
        "windows" => "Windows",
        "web" => "Web",
        _ => "Desconocido",
    }
}

pub fn device_name_label(device: &api::DeviceRow) -> String {
    device
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| platform_label(&device.platform).to_owned())
}

pub fn last_seen_label(value: Option<&str>) -> String {
    match value {
        Some(value) => format_timestamp(value),
        None => "Nunca".to_owned(),
    }
}

pub fn format_timestamp(value: &str) -> String {
    match chrono::DateTime::parse_from_rfc3339(value) {
        Ok(parsed) => parsed.format("%d-%m-%Y %H:%M").to_string(),
        Err(_) => value.to_owned(),
    }
}

pub fn format_date(value: &str) -> String {
    let date = value.get(..10).unwrap_or(value);
    match chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(parsed) => parsed.format("%d-%m-%Y").to_string(),
        Err(_) => value.to_owned(),
    }
}

pub fn prescription_status_label(status: &str) -> &'static str {
    match status {
        "draft" => "Borrador",
        "signed" => "Firmada",
        "superseded" => "Reemplazada",
        "voided" => "Anulada",
        "pending_sync" => "Provisional",
        _ => "Desconocida",
    }
}
