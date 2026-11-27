use dioxus::prelude::*;

use super::{
    Badge, BadgeTone, Button, ButtonVariant, EmptyState, ErrorText, FileField, Icon, IconName,
};
use crate::api;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SyncStatus {
    pub online: bool,
    pub pending: i64,
    pub failed: i64,
}

impl SyncStatus {
    pub fn label(&self) -> &'static str {
        if !self.online {
            return "Sin conexión";
        }
        if self.failed > 0 {
            return "Error de sincronización";
        }
        if self.pending > 0 {
            return "Sincronizando";
        }
        "En línea"
    }

    pub fn tone(&self) -> &'static str {
        if !self.online {
            return "offline";
        }
        if self.failed > 0 {
            return "failed";
        }
        if self.pending > 0 {
            return "pending";
        }
        "online"
    }
}

#[component]
pub fn SyncStatusIndicator(status: SyncStatus) -> Element {
    let tone = status.tone();
    rsx! {
        div { class: "ui-sync ui-sync--{tone}", role: "status", aria_live: "polite",
            span { class: "ui-sync-dot", aria_hidden: "true" }
            span { "{status.label()}" }
        }
    }
}

#[component]
pub fn PrescriptionStatusBadge(status: String) -> Element {
    let (tone, label) = match status.as_str() {
        "draft" => (BadgeTone::Neutral, "Borrador"),
        "signed" => (BadgeTone::Mint, "Firmada"),
        "superseded" => (BadgeTone::Cream, "Reemplazada"),
        "voided" => (BadgeTone::Blossom, "Anulada"),
        "pending_sync" => (BadgeTone::Cream, "Provisional"),
        "failed" => (BadgeTone::Blossom, "Error de sincronización"),
        _ => (BadgeTone::Neutral, "Desconocido"),
    };
    rsx! {
        Badge { tone, "{label}" }
    }
}

#[component]
pub fn TemplateStatusBadge(status: String) -> Element {
    let (tone, label) = match status.as_str() {
        "draft" => (BadgeTone::Neutral, "Borrador"),
        "published" => (BadgeTone::Mint, "Publicada"),
        "archived" => (BadgeTone::Cream, "Archivada"),
        _ => (BadgeTone::Neutral, "Desconocido"),
    };
    rsx! {
        Badge { tone, "{label}" }
    }
}

#[component]
pub fn AccountStatusBadge(status: String) -> Element {
    let (tone, label) = match status.as_str() {
        "active" => (BadgeTone::Mint, "Activo"),
        "disabled" => (BadgeTone::Neutral, "Deshabilitado"),
        _ => (BadgeTone::Neutral, "Desconocido"),
    };
    rsx! {
        Badge { tone, "{label}" }
    }
}

#[component]
pub fn DeviceStatusBadge(revoked: bool) -> Element {
    if revoked {
        rsx! {
            Badge { tone: BadgeTone::Blossom, "Revocado" }
        }
    } else {
        rsx! {
            Badge { tone: BadgeTone::Mint, "Activo" }
        }
    }
}

#[component]
pub fn RxBlockIndicator(
    status: Option<api::RxBlockStatus>,
    #[props(default)] allocating: bool,
    on_allocate: EventHandler<MouseEvent>,
) -> Element {
    let Some(status) = status else {
        return rsx! {};
    };
    let can_reserve = !status.active || status.exhausted;
    rsx! {
        div { class: "ui-rx-block",
            {match status.active {
                false => rsx! {
                    span { "Sin bloque de números activo." }
                },
                true => rsx! {
                    span { "Números disponibles: {status.remaining} de {status.limit}" }
                },
            }}
            if status.active && status.exhausted {
                span { "Bloque agotado. Reserva otro antes de firmar." }
            }
            if can_reserve {
                Button {
                    variant: ButtonVariant::Secondary,
                    disabled: allocating,
                    onclick: move |event| on_allocate.call(event),
                    "Reservar bloque"
                }
            }
        }
    }
}

#[component]
pub fn LlmExplainLink(href: String) -> Element {
    rsx! {
        div { class: "ui-llm",
            a {
                href: "{href}",
                target: "_blank",
                rel: "noopener noreferrer",
                "Explicar con LLM"
            }
            p { class: "ui-hint",
                "Se abre un sitio externo con el nombre del medicamento. No se envían datos del paciente."
            }
        }
    }
}

fn verification_slug(status: &str) -> &'static str {
    match status {
        "valid" => "valid",
        "voided" => "voided",
        "superseded" => "superseded",
        "not found" => "not-found",
        _ => "unknown",
    }
}

#[component]
pub fn VerificationSeal(status: String) -> Element {
    let slug = verification_slug(&status);
    let icon = match slug {
        "valid" => IconName::Check,
        "voided" => IconName::Close,
        "superseded" => IconName::ArrowRight,
        "not-found" => IconName::Qr,
        _ => IconName::Info,
    };
    rsx! {
        div {
            class: "ui-verify-seal ui-verify-seal--{slug}",
            aria_hidden: "true",
            if slug == "valid" {
                div { class: "ui-verify-ring",
                    Icon { name: icon, size: 24 }
                }
            } else {
                Icon { name: icon, size: 24 }
            }
        }
    }
}

#[component]
pub fn VerificationResultCard(verification: api::VerificationResult) -> Element {
    let slug = verification_slug(&verification.status);
    let (title, copy) = match slug {
        "valid" => (
            "Receta auténtica",
            "Esta receta fue emitida y firmada por un profesional autorizado en PersonaSanus.",
        ),
        "voided" => (
            "Receta anulada",
            "El médico emisor anuló esta receta. El documento ya no es válido.",
        ),
        "superseded" => (
            "Receta reemplazada",
            "Esta receta fue reemplazada por una corrección posterior. Consulta la versión vigente.",
        ),
        "not-found" => (
            "Receta no encontrada",
            "El código no corresponde a una receta emitida. Revisa el código impreso junto al QR.",
        ),
        _ => (
            "Receta no verificable",
            "No pudimos determinar el estado de este documento.",
        ),
    };
    let show_details = matches!(slug, "valid" | "voided" | "superseded");
    rsx! {
        div { class: "ui-verify-card ui-verify-card--{slug}",
            p { class: "ui-verify-eyebrow", "VERIFICACIÓN PÚBLICA" }
            VerificationSeal { status: verification.status.clone() }
            h2 { class: "ui-verify-title", "{title}" }
            p { class: "ui-verify-copy", "{copy}" }
            if show_details {
                dl { class: "ui-verify-details",
                    div {
                        dt { "Médico" }
                        dd {
                            "{verification.doctor_name.as_deref().unwrap_or(\"—\")}"
                        }
                    }
                    div {
                        dt { "Fecha de receta" }
                        dd {
                            "{verification.issued_at.as_deref().map(crate::ui::format_date).unwrap_or_else(|| \"—\".to_owned())}"
                        }
                    }
                    div {
                        dt { "Número de receta" }
                        dd {
                            if let Some(number) = verification.rx_number {
                                "{number:06}"
                            } else {
                                "—"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn VerificationCodeForm(
    #[props(default)] button_variant: ButtonVariant,
    error_id: &'static str,
    on_submit: EventHandler<String>,
) -> Element {
    let mut value = use_signal(String::new);
    let mut invalid = use_signal(|| false);
    rsx! {
        form {
            class: "ui-code-form",
            onsubmit: move |event| {
                event.prevent_default();
                let code = value().trim().to_uppercase();
                if code.is_empty() {
                    invalid.set(true);
                    return;
                }
                invalid.set(false);
                on_submit.call(code);
            },
            label { class: "ui-field",
                span { class: "ui-field-label", "Código de verificación" }
                input {
                    class: "ui-input",
                    r#type: "text",
                    placeholder: "PS-7K2F-9Q",
                    value: value(),
                    autocomplete: "off",
                    spellcheck: false,
                    aria_invalid: "{invalid()}",
                    aria_describedby: invalid().then_some(error_id),
                    oninput: move |event| value.set(event.value()),
                }
            }
            Button { variant: button_variant, button_type: "submit", "Verificar" }
            if invalid() {
                div { class: "ui-code-form-error", id: error_id,
                    ErrorText { text: "Ingresa el código de verificación.".to_owned() }
                }
            }
        }
    }
}

#[component]
pub fn QuotaMeter(used: i64, limit: i64) -> Element {
    let max = if limit <= 0 { 1 } else { limit };
    let percent = if limit <= 0 {
        0
    } else {
        used.clamp(0, limit).saturating_mul(100) / limit
    };
    rsx! {
        div {
            class: "ui-quota",
            role: "progressbar",
            aria_valuemin: "0",
            aria_valuemax: "{max}",
            aria_valuenow: "{used}",
            p { "Uso de la biblioteca: {used} de {limit}." }
            div { class: "ui-quota-track",
                div { class: "ui-quota-fill", style: "width: {percent}%;" }
            }
        }
    }
}

#[component]
pub fn ImageLibrary(
    assets: Vec<api::AssetRow>,
    used: i64,
    limit: i64,
    uploading: bool,
    deleting: Option<String>,
    online: bool,
    on_delete: EventHandler<String>,
    on_upload: EventHandler<FormEvent>,
) -> Element {
    let images: Vec<api::AssetRow> = assets
        .into_iter()
        .filter(|asset| asset.kind == "image")
        .collect();
    let at_limit = used >= limit;
    let hint = if !online {
        "Subir imágenes requiere conexión."
    } else if at_limit {
        "La biblioteca está llena."
    } else {
        "JPEG o PNG, máximo 10 MB."
    };
    let deletable = deleting.is_some() || !online;
    let upload_disabled = !online || uploading || at_limit;
    rsx! {
        div { class: "ui-library",
            QuotaMeter { used, limit }
            if images.is_empty() {
                EmptyState {
                    title: "Sin imágenes",
                    description: "Sube una imagen JPEG o PNG de hasta 10 MB.",
                }
            } else {
                for asset in images {
                    div { class: "ui-library-row",
                        if asset.status == "pending" {
                            Badge { tone: BadgeTone::Cream, "Procesando" }
                        } else if let Some(url) = asset.url.clone() {
                            img {
                                class: "ui-library-thumb",
                                src: "{url}",
                                alt: "Imagen de la biblioteca",
                            }
                        } else {
                            div { class: "ui-library-thumb ui-library-thumb--placeholder",
                                Icon { name: IconName::Image, size: 20 }
                            }
                        }
                        div { class: "ui-library-info",
                            p {
                                if let Some(name) = asset.original_name.clone().filter(|name| !name.trim().is_empty()) {
                                    "{name}"
                                } else {
                                    "Imagen {asset.id.chars().take(8).collect::<String>()}"
                                }
                            }
                            p { class: "ui-hint",
                                "{asset.width.map(|value| value.to_string()).unwrap_or_else(|| \"—\".to_owned())} × {asset.height.map(|value| value.to_string()).unwrap_or_else(|| \"—\".to_owned())} · {format_bytes(asset.size_bytes)}"
                            }
                        }
                        Button {
                            variant: ButtonVariant::Danger,
                            disabled: deletable,
                            onclick: {
                                let id = asset.id.clone();
                                move |_| on_delete.call(id.clone())
                            },
                            if deleting.as_deref() == Some(asset.id.as_str()) {
                                "Eliminando..."
                            } else {
                                "Eliminar"
                            }
                        }
                    }
                }
            }
            FileField {
                label: "Subir imagen de la biblioteca (JPEG o PNG, máximo 10 MB)",
                accept: "image/jpeg,image/png",
                disabled: upload_disabled,
                hint,
                wide: true,
                on_change: move |event| on_upload.call(event),
            }
        }
    }
}

fn format_bytes(bytes: i64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
