use std::collections::BTreeMap;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::api;
use crate::ui::{
    ActionRow, Button, ButtonVariant, Card, ErrorText, FormColumns, FormGrid, HintText,
    SectionCard, SelectField, TextAreaField, TextField,
};

pub const SCHEMA_VERSION: u32 = 1;

pub const TEMPLATE_VARIABLES: [&str; 14] = [
    "patient.name",
    "patient.national_id",
    "patient.age",
    "doctor.name",
    "doctor.specialty",
    "doctor.national_id",
    "doctor.contact",
    "clinic.name",
    "clinic.address",
    "clinic.phone",
    "prescription.number",
    "prescription.date",
    "prescription.diagnosis",
    "medications[]",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateDocument {
    pub schema_version: u32,
    pub letterhead: Letterhead,
    pub body: String,
    pub footer: String,
    #[serde(default)]
    pub diagnosis: String,
    #[serde(default)]
    pub images: Vec<ImageSlot>,
    #[serde(default)]
    pub medications: Vec<MedicationBlock>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Letterhead {
    #[serde(default)]
    pub logo_asset_id: Option<String>,
    pub doctor_details: String,
    pub clinic_details: String,
    pub colors: DocumentColors,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentColors {
    pub primary: String,
    pub accent: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageSlot {
    #[serde(default)]
    pub asset_id: Option<String>,
    pub caption: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MedicationBlock {
    pub name: String,
    pub dose: String,
    pub route: String,
    pub frequency: String,
    pub duration: String,
    pub quantity: String,
    pub notes: String,
}

impl Default for TemplateDocument {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            letterhead: Letterhead {
                logo_asset_id: None,
                doctor_details: "Dr. Nombre Apellido\nEspecialidad\nRegistro profesional"
                    .to_owned(),
                clinic_details: "Clínica\nDirección · Teléfono".to_owned(),
                colors: DocumentColors {
                    primary: "#0f6e64".to_owned(),
                    accent: "#e8b04b".to_owned(),
                },
            },
            body: "Paciente: {{patient.name}}\nEdad: {{patient.age}}\n\nIndicaciones generales."
                .to_owned(),
            footer: "Firma y sello del médico\n{{doctor.name}} · {{doctor.contact}}".to_owned(),
            diagnosis: String::new(),
            images: Vec::new(),
            medications: vec![MedicationBlock::default()],
        }
    }
}

pub fn default_document_json() -> String {
    serde_json::to_string(&TemplateDocument::default()).unwrap_or_default()
}

pub fn parse_document(json: &str) -> Result<TemplateDocument, String> {
    let document: TemplateDocument = serde_json::from_str(json)
        .map_err(|_| "El documento no tiene el formato estructurado esperado.".to_owned())?;
    validate_document(&document)?;
    Ok(document)
}

fn valid_color(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 7 && bytes[0] == b'#' && bytes[1..].iter().all(|byte| byte.is_ascii_hexdigit())
}

fn check_text(value: &str, label: &str, max: usize) -> Result<(), String> {
    if value.chars().count() > max {
        return Err(format!("El campo {label} supera el tamaño permitido."));
    }
    Ok(())
}

fn check_asset_id(value: Option<&str>, label: &str) -> Result<(), String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    if value.chars().count() > 200 || value.chars().any(char::is_whitespace) {
        return Err(format!(
            "El identificador de imagen de {label} no es válido."
        ));
    }
    Ok(())
}

pub fn validate_document(document: &TemplateDocument) -> Result<(), String> {
    if document.schema_version != SCHEMA_VERSION {
        return Err("La versión del esquema del documento no es compatible.".to_owned());
    }
    if !valid_color(&document.letterhead.colors.primary)
        || !valid_color(&document.letterhead.colors.accent)
    {
        return Err("Los colores deben usar el formato #RRGGBB.".to_owned());
    }
    check_asset_id(document.letterhead.logo_asset_id.as_deref(), "la cabecera")?;
    check_text(
        &document.letterhead.doctor_details,
        "datos del médico",
        4000,
    )?;
    check_text(
        &document.letterhead.clinic_details,
        "datos de la clínica",
        4000,
    )?;
    check_text(&document.body, "cuerpo", 40000)?;
    check_text(&document.footer, "pie de página", 4000)?;
    check_text(&document.diagnosis, "diagnóstico de la plantilla", 2000)?;
    if document.images.len() > 10 {
        return Err("El documento admite como máximo 10 imágenes.".to_owned());
    }
    for image in &document.images {
        check_asset_id(image.asset_id.as_deref(), "la imagen")?;
        check_text(&image.caption, "leyenda de imagen", 500)?;
    }
    if document.medications.len() > 50 {
        return Err("El documento admite como máximo 50 medicamentos.".to_owned());
    }
    for medication in &document.medications {
        check_text(&medication.name, "medicamento", 500)?;
        check_text(&medication.dose, "dosis", 500)?;
        check_text(&medication.route, "vía", 500)?;
        check_text(&medication.frequency, "frecuencia", 500)?;
        check_text(&medication.duration, "duración", 500)?;
        check_text(&medication.quantity, "cantidad", 500)?;
        check_text(&medication.notes, "notas del medicamento", 1000)?;
    }
    Ok(())
}

pub fn variable_label(key: &str) -> &'static str {
    match key {
        "patient.name" => "Nombre del paciente",
        "patient.national_id" => "Cédula del paciente",
        "patient.age" => "Edad del paciente",
        "doctor.name" => "Nombre del médico",
        "doctor.specialty" => "Especialidad del médico",
        "doctor.national_id" => "Cédula del médico",
        "doctor.contact" => "Contacto del médico",
        "clinic.name" => "Nombre de la clínica",
        "clinic.address" => "Dirección de la clínica",
        "clinic.phone" => "Teléfono de la clínica",
        "prescription.number" => "Número de receta",
        "prescription.date" => "Fecha de la receta",
        "prescription.diagnosis" => "Diagnóstico",
        "medications[]" => "Medicamentos",
        _ => "Variable",
    }
}

pub fn sample_values() -> Vec<(String, String)> {
    vec![
        ("patient.name".to_owned(), "María Fernanda Ruiz".to_owned()),
        ("patient.national_id".to_owned(), "0102030405".to_owned()),
        ("patient.age".to_owned(), "35 años".to_owned()),
        ("doctor.name".to_owned(), "Dr. Carlos Pérez".to_owned()),
        ("doctor.specialty".to_owned(), "Medicina General".to_owned()),
        ("doctor.national_id".to_owned(), "0912345678".to_owned()),
        (
            "doctor.contact".to_owned(),
            "099 999 9999 · dr.perez@example.com".to_owned(),
        ),
        ("clinic.name".to_owned(), "Clínica Sanus".to_owned()),
        (
            "clinic.address".to_owned(),
            "Av. Amazonas 123, Quito".to_owned(),
        ),
        ("clinic.phone".to_owned(), "+593 2 555 1234".to_owned()),
        ("prescription.number".to_owned(), "000123".to_owned()),
        ("prescription.date".to_owned(), "12-11-2026".to_owned()),
        (
            "prescription.diagnosis".to_owned(),
            "Diagnóstico de ejemplo".to_owned(),
        ),
        (
            "medications[]".to_owned(),
            "Ver tabla de medicamentos".to_owned(),
        ),
    ]
}

fn value_map(values: &[(String, String)]) -> BTreeMap<String, String> {
    values.iter().cloned().collect()
}

pub fn resolve_text(template: &str, values: &[(String, String)]) -> (String, Vec<String>) {
    let map = value_map(values);
    let mut output = String::new();
    let mut warnings = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        output.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find("}}") {
            Some(end) => {
                let key = after[..end].trim();
                if TEMPLATE_VARIABLES.contains(&key) {
                    match map.get(key) {
                        Some(value) => output.push_str(value),
                        None => output.push_str("(dato no disponible)"),
                    }
                } else {
                    output.push_str("(variable no válida)");
                    warnings.push(key.to_owned());
                }
                rest = &after[end + 2..];
            }
            None => {
                output.push_str("(variable no válida)");
                rest = "";
            }
        }
    }
    output.push_str(rest);
    (output, warnings)
}

pub fn document_warnings(document: &TemplateDocument) -> Vec<String> {
    let values = sample_values();
    let mut warnings = Vec::new();
    for text in [
        &document.letterhead.doctor_details,
        &document.letterhead.clinic_details,
        &document.body,
        &document.footer,
    ] {
        warnings.extend(resolve_text(text, &values).1);
    }
    warnings.sort();
    warnings.dedup();
    warnings
}

pub fn medication_line(medication: &MedicationBlock) -> Option<String> {
    if medication.name.trim().is_empty() && medication.dose.trim().is_empty() {
        return None;
    }
    let mut parts = Vec::new();
    for field in [
        &medication.name,
        &medication.dose,
        &medication.route,
        &medication.frequency,
        &medication.duration,
        &medication.quantity,
        &medication.notes,
    ] {
        if !field.trim().is_empty() {
            parts.push(field.trim());
        }
    }
    Some(format!("• {}", parts.join(" · ")))
}

pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let width = width.max(8);
    let mut lines = Vec::new();
    for raw in text.split('\n') {
        if raw.trim().is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut line = String::new();
        for word in raw.split_whitespace() {
            let mut word = word.to_owned();
            while word.chars().count() > width {
                if !line.is_empty() {
                    lines.push(std::mem::take(&mut line));
                }
                let split = word
                    .char_indices()
                    .nth(width)
                    .map(|(index, _)| index)
                    .unwrap_or(word.len());
                lines.push(word[..split].to_owned());
                word = word[split..].to_owned();
            }
            if line.is_empty() {
                line.push_str(&word);
            } else if line.chars().count() + 1 + word.chars().count() <= width {
                line.push(' ');
                line.push_str(&word);
            } else {
                lines.push(std::mem::take(&mut line));
                line = word;
            }
        }
        lines.push(line);
    }
    lines
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DocumentMeta {
    pub verification_code: Option<String>,
    pub rx_number: Option<i32>,
    pub issued_at: Option<String>,
    pub provisional: bool,
    pub qr_data_url: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DocumentPage {
    pub lines: Vec<String>,
    pub images: Vec<usize>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DocumentLayout {
    pub primary: String,
    pub accent: String,
    pub header_doctor: Vec<String>,
    pub header_clinic: Vec<String>,
    pub footer: Vec<String>,
    pub pages: Vec<DocumentPage>,
    pub images: Vec<ImageSlot>,
    pub logo_asset_id: Option<String>,
    pub margin_mm: u32,
}

pub fn paginate_document(
    document: &TemplateDocument,
    values: &[(String, String)],
    margin_mm: u32,
    medications: &[MedicationBlock],
    meta: &DocumentMeta,
) -> DocumentLayout {
    let margin = margin_mm.clamp(5, 40);
    let content_width = (210 - 2 * margin) as f32;
    let width = (content_width / 1.95).floor().max(24.0) as usize;
    let footer_width = (((content_width - 32.0).max(60.0)) / 1.8).floor().max(20.0) as usize;
    let (doctor, _) = resolve_text(&document.letterhead.doctor_details, values);
    let (clinic, _) = resolve_text(&document.letterhead.clinic_details, values);
    let (body, _) = resolve_text(&document.body, values);
    let (footer, _) = resolve_text(&document.footer, values);
    let header_doctor = wrap_text(&doctor, width);
    let header_clinic = wrap_text(&clinic, width);
    let mut footer_lines = wrap_text(&footer, footer_width);
    let mut meta_lines = Vec::new();
    if let (Some(number), Some(date)) = (meta.rx_number, meta.issued_at.as_deref()) {
        meta_lines.push(format!("Receta No. {number:06} · {date}"));
    } else if let Some(date) = meta.issued_at.as_deref() {
        meta_lines.push(format!("Fecha de emisión: {date}"));
    }
    if meta.provisional {
        meta_lines.push("PENDIENTE DE SINCRONIZACIÓN".to_owned());
    } else if let Some(code) = meta.verification_code.as_deref() {
        meta_lines.push(format!("Código de verificación: {code}"));
    }
    meta_lines.extend(wrap_text(
        "Firma: ______________________________",
        footer_width,
    ));
    footer_lines.extend(meta_lines);

    let mut body_lines = wrap_text(&body, width);
    for medication in medications {
        if let Some(line) = medication_line(medication) {
            body_lines.extend(wrap_text(&line, width));
        }
    }
    if body_lines.iter().all(|line| line.trim().is_empty()) {
        body_lines = vec![String::new()];
    }
    let header_height = header_doctor.len() as f32 * 6.0 + header_clinic.len() as f32 * 4.6 + 9.0;
    let footer_height = footer_lines.len() as f32 * 4.6 + 32.0;
    let images_height = document.images.len() as f32 * 33.0;
    let usable = (297.0 - 2.0 * margin as f32 - header_height - footer_height).max(45.0);
    let per_page = (usable / 5.2).floor().max(8.0) as usize;
    let first_capacity = (((usable - images_height).max(20.0)) / 5.2)
        .floor()
        .max(4.0) as usize;
    let mut pages = Vec::new();
    let mut rest = body_lines.as_slice();
    let mut first = true;
    while !rest.is_empty() {
        let capacity = if first { first_capacity } else { per_page };
        let take = capacity.min(rest.len());
        let (chunk, tail) = rest.split_at(take);
        let mut lines = chunk.to_vec();
        while lines.last().is_some_and(|line| line.trim().is_empty()) && lines.len() > 1 {
            lines.pop();
        }
        pages.push(DocumentPage {
            lines,
            images: if first {
                (0..document.images.len()).collect()
            } else {
                Vec::new()
            },
        });
        rest = tail;
        first = false;
    }
    if pages.is_empty() {
        pages.push(DocumentPage {
            lines: Vec::new(),
            images: (0..document.images.len()).collect(),
        });
    }
    DocumentLayout {
        primary: document.letterhead.colors.primary.clone(),
        accent: document.letterhead.colors.accent.clone(),
        header_doctor,
        header_clinic,
        footer: footer_lines,
        pages,
        images: document.images.clone(),
        logo_asset_id: document.letterhead.logo_asset_id.clone(),
        margin_mm: margin,
    }
}

#[component]
pub fn A4Preview(
    doc_json: String,
    values: Vec<(String, String)>,
    margin_mm: u32,
    #[props(default)] medications: Option<Vec<MedicationBlock>>,
    #[props(default)] meta: Option<DocumentMeta>,
    #[props(default)] image_sources: Vec<(String, String)>,
) -> Element {
    let document = match parse_document(&doc_json) {
        Ok(document) => document,
        Err(error) => {
            return rsx! { ErrorText { text: error } };
        }
    };
    let values = if values.is_empty() {
        sample_values()
    } else {
        values
    };
    let meta = meta.unwrap_or_default();
    let medications = medications.unwrap_or_else(|| document.medications.clone());
    let mut header_warnings = Vec::new();
    for text in [
        &document.letterhead.doctor_details,
        &document.letterhead.clinic_details,
        &document.footer,
    ] {
        header_warnings.extend(resolve_text(text, &values).1);
    }
    let layout = paginate_document(&document, &values, margin_mm, &medications, &meta);
    let total = layout.pages.len();
    let primary = layout.primary.clone();
    let accent = layout.accent.clone();
    let sources: BTreeMap<String, String> = image_sources.into_iter().collect();
    let qr = meta.qr_data_url.clone();
    let provisional = meta.provisional;

    rsx! {
        div {
            class: "a4-pages",
            for (index, page) in layout.pages.iter().enumerate() {
                div {
                    key: "{index}",
                    class: "a4-page",
                    style: "padding: {layout.margin_mm}mm;",
                    div {
                        class: "a4-header",
                        style: "border-color: {primary};",
                        if let Some(source) = layout.logo_asset_id.as_ref().and_then(|id| sources.get(id)) {
                            img { class: "a4-logo-image", src: "{source}", alt: "Logo de la clínica" }
                        }
                        for (line_index, line) in layout.header_doctor.iter().enumerate() {
                            p {
                                key: "doctor-{line_index}",
                                class: "a4-doctor",
                                style: "color: {primary};",
                                "{line}"
                            }
                        }
                        for (line_index, line) in layout.header_clinic.iter().enumerate() {
                            p {
                                key: "clinic-{line_index}",
                                class: "a4-clinic",
                                "{line}"
                            }
                        }
                    }
                    if !page.images.is_empty() {
                        div {
                            class: "a4-images",
                            for image_index in page.images.iter().copied() {
                                if let Some(image) = layout.images.get(image_index) {
                                    if let Some(source) = image.asset_id.as_ref().and_then(|id| sources.get(id)) {
                                        img {
                                            key: "image-{image_index}",
                                            class: "a4-image",
                                            src: "{source}",
                                            alt: "{image.caption}",
                                        }
                                    } else {
                                        p {
                                            key: "image-{image_index}",
                                            "Imagen: {image.caption}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div {
                        class: "a4-body",
                        for (line_index, line) in page.lines.iter().enumerate() {
                            p { key: "{line_index}", "{line}" }
                        }
                    }
                    div {
                        class: "a4-footer",
                        style: "border-color: {accent};",
                        for (line_index, line) in layout.footer.iter().enumerate() {
                            p { key: "{line_index}", "{line}" }
                        }
                        div {
                            class: "a4-qr",
                            if provisional {
                                div { class: "a4-qr-placeholder", "QR provisional" }
                            } else if let Some(source) = qr.as_ref() {
                                img { src: "{source}", alt: "Código QR de verificación" }
                            } else {
                                div { class: "a4-qr-placeholder", "QR de verificación" }
                            }
                        }
                        p { class: "a4-page-no", "Página {index + 1} de {total}" }
                    }
                }
            }
            if !header_warnings.is_empty() {
                div {
                    class: "a4-warnings",
                    p { "Variables no válidas en la cabecera o el pie: {header_warnings.join(\", \")}" }
                }
            }
        }
    }
}

fn save_document(
    mut saving: Signal<bool>,
    mut message: Signal<String>,
    mut saved_version: Signal<Option<String>>,
    template_id: String,
    margin: u32,
    document: TemplateDocument,
    on_saved: EventHandler<(String, String, u32)>,
) {
    if saving() {
        return;
    }
    if let Err(error) = validate_document(&document) {
        message.set(error);
        return;
    }
    let doc_json = match serde_json::to_string(&document) {
        Ok(json) => json,
        Err(_) => {
            message.set("No se pudo preparar el documento.".to_owned());
            return;
        }
    };
    let theme = json!({ "margin_mm": margin }).to_string();
    let existing = saved_version();
    saving.set(true);
    message.set(String::new());
    spawn(async move {
        let result = match existing {
            Some(version_id) => {
                api::update_template_version(&template_id, &version_id, &doc_json, &theme).await
            }
            None => api::create_template_version(&template_id, &doc_json, &theme).await,
        };
        saving.set(false);
        match result {
            Ok(version) => {
                saved_version.set(Some(version.id.clone()));
                message.set("Borrador guardado.".to_owned());
                on_saved.call((version.id, doc_json, margin));
            }
            Err(error) => message.set(
                if error.starts_with("No se pudo contactar")
                    || error.starts_with("No se pudo leer la respuesta")
                    || error.starts_with("La solicitud falló")
                {
                    "No se pudo guardar el borrador.".to_owned()
                } else {
                    error
                },
            ),
        }
    });
}

#[component]
pub fn TemplateDocumentForm(
    template_id: String,
    version_id: Option<String>,
    initial_doc: String,
    initial_margin_mm: u32,
    on_saved: EventHandler<(String, String, u32)>,
) -> Element {
    let mut document =
        use_signal(|| parse_document(&initial_doc).unwrap_or_else(|_| TemplateDocument::default()));
    let saved_version = use_signal(|| version_id.clone());
    let mut margin = use_signal(|| initial_margin_mm.clamp(5, 40));
    let mut picker = use_signal(|| TEMPLATE_VARIABLES[0].to_owned());
    let message = use_signal(String::new);
    let saving = use_signal(|| false);
    let library = use_resource(|| async move { api::fetch_assets().await });
    let library_rows = match library() {
        Some(Ok(library)) => library.assets,
        _ => Vec::new(),
    };

    let insert_variable = move |_| {
        let key = picker();
        let mut draft = document.write();
        draft.body.push_str(&format!("{{{{{key}}}}}"));
    };

    rsx! {
        SectionCard { title: "Membrete".to_owned(), description: None,
            FormGrid { columns: FormColumns::Two,
                SelectField {
                    label: "Logo de la clínica".to_owned(),
                    value: document().letterhead.logo_asset_id.clone().unwrap_or_default(),
                    onchange: move |value: String| {
                        document.write().letterhead.logo_asset_id =
                            if value.is_empty() { None } else { Some(value) };
                    },
                    wide: false,
                    hint: None,
                    error: None,
                    disabled: false,
                    hide_label: false,
                    option { value: "", "Sin logo" }
                    for asset in library_rows.iter() {
                        option { value: "{asset.id}", "{asset_label(asset)}" }
                    }
                }
                TextField {
                    label: "Color principal".to_owned(),
                    value: document().letterhead.colors.primary.clone(),
                    oninput: move |value| {
                        document.write().letterhead.colors.primary = value;
                    },
                    input_type: "color".to_owned(),
                    placeholder: String::new(),
                    hint: None,
                    error: None,
                    autocomplete: "off".to_owned(),
                    spellcheck: false,
                    required: false,
                    disabled: false,
                    readonly: false,
                    min: None,
                    max: None,
                    step: None,
                    wide: false,
                    hide_label: false,
                }
                TextField {
                    label: "Color de acento".to_owned(),
                    value: document().letterhead.colors.accent.clone(),
                    oninput: move |value| {
                        document.write().letterhead.colors.accent = value;
                    },
                    input_type: "color".to_owned(),
                    placeholder: String::new(),
                    hint: None,
                    error: None,
                    autocomplete: "off".to_owned(),
                    spellcheck: false,
                    required: false,
                    disabled: false,
                    readonly: false,
                    min: None,
                    max: None,
                    step: None,
                    wide: false,
                    hide_label: false,
                }
                TextField {
                    label: "Márgenes (mm)".to_owned(),
                    value: margin().to_string(),
                    oninput: move |value: String| {
                        if let Ok(parsed) = value.parse::<u32>() {
                            margin.set(parsed.clamp(5, 40));
                        }
                    },
                    input_type: "number".to_owned(),
                    placeholder: String::new(),
                    hint: None,
                    error: None,
                    autocomplete: "off".to_owned(),
                    spellcheck: false,
                    required: false,
                    disabled: false,
                    readonly: false,
                    min: Some("5".to_owned()),
                    max: Some("40".to_owned()),
                    step: None,
                    wide: false,
                    hide_label: false,
                }
                TextAreaField {
                    label: "Datos del médico".to_owned(),
                    value: document().letterhead.doctor_details.clone(),
                    oninput: move |value| {
                        document.write().letterhead.doctor_details = value;
                    },
                    placeholder: String::new(),
                    wide: true,
                    hint: None,
                    error: None,
                    required: false,
                    disabled: false,
                    readonly: false,
                    hide_label: false,
                    rows: None,
                }
                TextAreaField {
                    label: "Datos de la clínica".to_owned(),
                    value: document().letterhead.clinic_details.clone(),
                    oninput: move |value| {
                        document.write().letterhead.clinic_details = value;
                    },
                    placeholder: String::new(),
                    wide: true,
                    hint: None,
                    error: None,
                    required: false,
                    disabled: false,
                    readonly: false,
                    hide_label: false,
                    rows: None,
                }
            }
        }
        SectionCard { title: "Cuerpo".to_owned(), description: None,
            div { class: "ui-variable-row",
                SelectField {
                    label: "Variable".to_owned(),
                    value: picker(),
                    onchange: move |value| picker.set(value),
                    wide: false,
                    hint: None,
                    error: None,
                    disabled: false,
                    hide_label: false,
                    for key in TEMPLATE_VARIABLES {
                        option { value: "{key}", "{variable_label(key)} — {key}" }
                    }
                }
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: insert_variable,
                    "Insertar variable"
                }
            }
            TextAreaField {
                label: "Cuerpo de la receta".to_owned(),
                value: document().body.clone(),
                oninput: move |value| document.write().body = value,
                placeholder: String::new(),
                wide: true,
                hint: None,
                error: None,
                required: false,
                disabled: false,
                readonly: false,
                hide_label: false,
                rows: None,
            }
            TextAreaField {
                label: "Pie de página".to_owned(),
                value: document().footer.clone(),
                oninput: move |value| document.write().footer = value,
                placeholder: String::new(),
                wide: true,
                hint: None,
                error: None,
                required: false,
                disabled: false,
                readonly: false,
                hide_label: false,
                rows: None,
            }
            TextAreaField {
                label: "Diagnóstico".to_owned(),
                value: document().diagnosis.clone(),
                oninput: move |value| document.write().diagnosis = value,
                placeholder: String::new(),
                wide: true,
                hint: None,
                error: None,
                required: false,
                disabled: false,
                readonly: false,
                hide_label: false,
                rows: None,
            }
        }
        SectionCard { title: "Imágenes".to_owned(), description: None,
            HintText {
                text: "Elige imágenes de la biblioteca de la clínica. La biblioteca admite una imagen en esta versión.".to_owned()
            }
            for index in 0..document().images.len() {
                Card {
                    key: "image-{index}",
                    FormGrid { columns: FormColumns::Two,
                        SelectField {
                            label: "Imagen".to_owned(),
                            value: document().images[index].asset_id.clone().unwrap_or_default(),
                            onchange: move |value: String| {
                                document.write().images[index].asset_id =
                                    if value.is_empty() { None } else { Some(value) };
                            },
                            wide: false,
                            hint: None,
                            error: None,
                            disabled: false,
                            hide_label: false,
                            option { value: "", "Sin imagen" }
                            for asset in library_rows.iter().filter(|asset| asset.kind == "image") {
                                option { value: "{asset.id}", "{asset_label(asset)}" }
                            }
                        }
                        TextField {
                            label: "Leyenda".to_owned(),
                            value: document().images[index].caption.clone(),
                            oninput: move |value| {
                                document.write().images[index].caption = value;
                            },
                            placeholder: String::new(),
                            hint: None,
                            error: None,
                            autocomplete: "off".to_owned(),
                            spellcheck: true,
                            required: false,
                            disabled: false,
                            readonly: false,
                            min: None,
                            max: None,
                            step: None,
                            wide: false,
                            hide_label: false,
                        }
                    }
                    ActionRow {
                        Button {
                            variant: ButtonVariant::Danger,
                            onclick: move |_| {
                                document.write().images.remove(index);
                            },
                            "Quitar imagen"
                        }
                    }
                }
            }
            Button {
                variant: ButtonVariant::Secondary,
                onclick: move |_| {
                    let first_image = match library() {
                        Some(Ok(library)) => library
                            .assets
                            .iter()
                            .find(|asset| asset.kind == "image")
                            .map(|asset| asset.id.clone()),
                        _ => None,
                    };
                    document.write().images.push(ImageSlot {
                        asset_id: first_image,
                        caption: String::new(),
                    });
                },
                "Agregar imagen"
            }
        }
        SectionCard { title: "Medicamentos".to_owned(), description: None,
            for index in 0..document().medications.len() {
                Card {
                    key: "medication-{index}",
                    FormGrid { columns: FormColumns::Two,
                        TextField {
                            label: "Medicamento".to_owned(),
                            value: document().medications[index].name.clone(),
                            oninput: move |value| document.write().medications[index].name = value,
                            placeholder: String::new(),
                            hint: None,
                            error: None,
                            autocomplete: "off".to_owned(),
                            spellcheck: true,
                            required: false,
                            disabled: false,
                            readonly: false,
                            min: None,
                            max: None,
                            step: None,
                            wide: false,
                            hide_label: false,
                        }
                        TextField {
                            label: "Dosis".to_owned(),
                            value: document().medications[index].dose.clone(),
                            oninput: move |value| document.write().medications[index].dose = value,
                            placeholder: String::new(),
                            hint: None,
                            error: None,
                            autocomplete: "off".to_owned(),
                            spellcheck: true,
                            required: false,
                            disabled: false,
                            readonly: false,
                            min: None,
                            max: None,
                            step: None,
                            wide: false,
                            hide_label: false,
                        }
                        TextField {
                            label: "Vía".to_owned(),
                            value: document().medications[index].route.clone(),
                            oninput: move |value| document.write().medications[index].route = value,
                            placeholder: String::new(),
                            hint: None,
                            error: None,
                            autocomplete: "off".to_owned(),
                            spellcheck: true,
                            required: false,
                            disabled: false,
                            readonly: false,
                            min: None,
                            max: None,
                            step: None,
                            wide: false,
                            hide_label: false,
                        }
                        TextField {
                            label: "Frecuencia".to_owned(),
                            value: document().medications[index].frequency.clone(),
                            oninput: move |value| document.write().medications[index].frequency = value,
                            placeholder: String::new(),
                            hint: None,
                            error: None,
                            autocomplete: "off".to_owned(),
                            spellcheck: true,
                            required: false,
                            disabled: false,
                            readonly: false,
                            min: None,
                            max: None,
                            step: None,
                            wide: false,
                            hide_label: false,
                        }
                        TextField {
                            label: "Duración".to_owned(),
                            value: document().medications[index].duration.clone(),
                            oninput: move |value| document.write().medications[index].duration = value,
                            placeholder: String::new(),
                            hint: None,
                            error: None,
                            autocomplete: "off".to_owned(),
                            spellcheck: true,
                            required: false,
                            disabled: false,
                            readonly: false,
                            min: None,
                            max: None,
                            step: None,
                            wide: false,
                            hide_label: false,
                        }
                        TextField {
                            label: "Cantidad".to_owned(),
                            value: document().medications[index].quantity.clone(),
                            oninput: move |value| document.write().medications[index].quantity = value,
                            placeholder: String::new(),
                            hint: None,
                            error: None,
                            autocomplete: "off".to_owned(),
                            spellcheck: false,
                            required: false,
                            disabled: false,
                            readonly: false,
                            min: None,
                            max: None,
                            step: None,
                            wide: false,
                            hide_label: false,
                        }
                        TextField {
                            label: "Notas".to_owned(),
                            value: document().medications[index].notes.clone(),
                            oninput: move |value| document.write().medications[index].notes = value,
                            placeholder: String::new(),
                            hint: None,
                            error: None,
                            autocomplete: "off".to_owned(),
                            spellcheck: true,
                            required: false,
                            disabled: false,
                            readonly: false,
                            min: None,
                            max: None,
                            step: None,
                            wide: true,
                            hide_label: false,
                        }
                    }
                    ActionRow {
                        Button {
                            variant: ButtonVariant::Danger,
                            onclick: move |_| {
                                document.write().medications.remove(index);
                            },
                            "Quitar medicamento"
                        }
                    }
                }
            }
            Button {
                variant: ButtonVariant::Secondary,
                onclick: move |_| {
                    document.write().medications.push(MedicationBlock::default());
                },
                "Agregar medicamento"
            }
        }
        if !message().is_empty() && message() != "Borrador guardado." {
            ErrorText { text: message() }
        }
        ActionRow {
            Button {
                variant: ButtonVariant::Secondary,
                disabled: saving(),
                loading: saving(),
                onclick: {
                    let template_id = template_id.clone();
                    move |_| {
                        save_document(
                            saving,
                            message,
                            saved_version,
                            template_id.clone(),
                            margin(),
                            document(),
                            on_saved,
                        );
                    }
                },
                if saving() { "Guardando..." } else { "Guardar borrador" }
            }
        }
    }
}

pub fn template_theme_margin(theme: Option<&str>) -> u32 {
    theme
        .and_then(|theme| serde_json::from_str::<Value>(theme).ok())
        .and_then(|value| value.get("margin_mm").and_then(Value::as_u64))
        .map(|value| value.clamp(5, 40) as u32)
        .unwrap_or(15)
}

fn asset_label(asset: &api::AssetRow) -> String {
    asset.original_name.clone().unwrap_or_else(|| {
        let id = asset.id.as_str();
        format!("Imagen {}", &id[..id.len().min(8)])
    })
}
