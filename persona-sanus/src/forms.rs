use std::collections::BTreeMap;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::api;
use crate::ui::{
    ActionRow, Button, ButtonVariant, Card, CheckboxField, ErrorText, FormColumns, FormGrid,
    HintText, SectionCard, SelectField, TextAreaField, TextField,
};

pub const SCHEMA_VERSION: u32 = 2;

pub const TEMPLATE_VARIABLES: [&str; 24] = [
    "patient.name",
    "patient.national_id",
    "patient.age",
    "patient.age_detail",
    "patient.sex",
    "patient.hcl",
    "patient.allergies",
    "doctor.name",
    "doctor.specialty",
    "doctor.national_id",
    "doctor.license_no",
    "doctor.contact",
    "clinic.name",
    "clinic.address",
    "clinic.phone",
    "prescription.number",
    "prescription.date",
    "prescription.time",
    "prescription.recetario_type",
    "prescription.validity",
    "prescription.diagnosis",
    "prescription.warning_signs",
    "prescription.instructions",
    "medications[]",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateDocument {
    pub schema_version: u32,
    pub page: PageSetup,
    pub letterhead: Letterhead,
    pub panels: Vec<Panel>,
    #[serde(default)]
    pub images: Vec<ImageSlot>,
    #[serde(default)]
    pub medications: Vec<MedicationBlock>,
    #[serde(default)]
    pub diagnosis: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageSetup {
    pub orientation: Orientation,
    pub panel_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Panel {
    pub title: String,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Section {
    Letterhead,
    PatientFields {
        fields: Vec<FieldKind>,
    },
    Validity {
        label_1_day: String,
        label_3_days: String,
    },
    Diagnosis {
        show_allergies: bool,
    },
    FreeText {
        title: String,
        text: String,
    },
    MedicationTable {
        style: MedicationTableStyle,
        #[serde(default)]
        header_captions: Vec<String>,
        show_observations: bool,
    },
    IndicationList {
        show_observations: bool,
    },
    WarningSigns,
    Recommendations,
    PrescriberBlock,
    ReceptionBlock,
    SignatureQr,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MedicationTableStyle {
    Full,
    Compact,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldKind {
    RecetarioType,
    IssueDate,
    IssueTime,
    PatientNationalId,
    PatientHcl,
    PatientName,
    PatientAge,
    PatientSex,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Letterhead {
    #[serde(default)]
    pub logo_asset_id: Option<String>,
    pub doctor_details: String,
    pub clinic_details: String,
    #[serde(default)]
    pub form_code: Option<String>,
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
    #[serde(default)]
    pub quantity_number: Option<i32>,
    pub notes: String,
}

impl Default for TemplateDocument {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            page: PageSetup {
                orientation: Orientation::Portrait,
                panel_count: 1,
            },
            letterhead: Letterhead {
                logo_asset_id: None,
                doctor_details: "Dr. Nombre Apellido\nEspecialidad\nRegistro profesional"
                    .to_owned(),
                clinic_details: "Clínica\nDirección · Teléfono".to_owned(),
                form_code: None,
                colors: DocumentColors {
                    primary: "#0f6e64".to_owned(),
                    accent: "#e8b04b".to_owned(),
                },
            },
            panels: vec![Panel {
                title: String::new(),
                sections: vec![
                    Section::Letterhead,
                    Section::FreeText {
                        title: String::new(),
                        text: "Paciente: {{patient.name}}\nEdad: {{patient.age}}\n\nIndicaciones generales.".to_owned(),
                    },
                    Section::Diagnosis { show_allergies: false },
                    Section::MedicationTable {
                        style: MedicationTableStyle::Compact,
                        header_captions: medication_table_compact_headers(),
                        show_observations: true,
                    },
                    Section::FreeText {
                        title: String::new(),
                        text: "Firma y sello del médico\n{{doctor.name}} · {{doctor.contact}}"
                            .to_owned(),
                    },
                    Section::SignatureQr,
                ],
            }],
            images: Vec::new(),
            medications: vec![MedicationBlock::default()],
            diagnosis: String::new(),
        }
    }
}

pub fn hvq_preset_document() -> TemplateDocument {
    TemplateDocument {
        schema_version: SCHEMA_VERSION,
        page: PageSetup {
            orientation: Orientation::Landscape,
            panel_count: 2,
        },
        letterhead: Letterhead {
            logo_asset_id: None,
            doctor_details: "{{doctor.name}}\n{{doctor.specialty}}".to_owned(),
            clinic_details: "{{clinic.name}}\n{{clinic.address}}\nTelf.: {{clinic.phone}}"
                .to_owned(),
            form_code: Some("5-189113".to_owned()),
            colors: DocumentColors {
                primary: "#111111".to_owned(),
                accent: "#111111".to_owned(),
            },
        },
        panels: vec![
            Panel {
                title: "MEDICACIÓN:".to_owned(),
                sections: vec![
                    Section::Letterhead,
                    Section::PatientFields {
                        fields: vec![
                            FieldKind::RecetarioType,
                            FieldKind::IssueDate,
                            FieldKind::IssueTime,
                            FieldKind::PatientNationalId,
                            FieldKind::PatientHcl,
                            FieldKind::PatientName,
                            FieldKind::PatientAge,
                        ],
                    },
                    Section::Validity {
                        label_1_day: "1 DIA".to_owned(),
                        label_3_days: "3 DIAS".to_owned(),
                    },
                    Section::Diagnosis {
                        show_allergies: true,
                    },
                    Section::MedicationTable {
                        style: MedicationTableStyle::Full,
                        header_captions: medication_table_full_headers(),
                        show_observations: false,
                    },
                    Section::ReceptionBlock,
                    Section::PrescriberBlock,
                    Section::SignatureQr,
                ],
            },
            Panel {
                title: "INDICACIONES:".to_owned(),
                sections: vec![
                    Section::Letterhead,
                    Section::PatientFields {
                        fields: vec![
                            FieldKind::RecetarioType,
                            FieldKind::IssueDate,
                            FieldKind::IssueTime,
                            FieldKind::PatientNationalId,
                            FieldKind::PatientHcl,
                            FieldKind::PatientName,
                            FieldKind::PatientAge,
                        ],
                    },
                    Section::Validity {
                        label_1_day: "1 DIA".to_owned(),
                        label_3_days: "3 DIAS".to_owned(),
                    },
                    Section::WarningSigns,
                    Section::Recommendations,
                    Section::IndicationList {
                        show_observations: true,
                    },
                    Section::PrescriberBlock,
                    Section::SignatureQr,
                ],
            },
        ],
        images: Vec::new(),
        medications: Vec::new(),
        diagnosis: String::new(),
    }
}

fn medication_table_full_headers() -> Vec<String> {
    vec![
        "DATOS DEL MEDICAMENTO (DENOMINACIÓN COMÚN INTERNACIONAL (DCI), FORMA FARMACÉUTICA Y CONCENTRACIÓN/DISPOSITIVO MÉDICO (TAMAÑO, CALIBRE, VOLUMEN...))".to_owned(),
        "CANTIDAD (EN NÚMEROS Y LETRAS)".to_owned(),
        "DOSIS / POSOLOGÍA".to_owned(),
        "VÍA DE ADMINISTRACIÓN".to_owned(),
        "FRECUENCIA DE LA ADMINISTRACIÓN".to_owned(),
        "DURACIÓN DEL TRATAMIENTO".to_owned(),
    ]
}

fn medication_table_compact_headers() -> Vec<String> {
    vec![
        "Medicamento".to_owned(),
        "Dosis / Posología".to_owned(),
        "Vía".to_owned(),
        "Frecuencia".to_owned(),
    ]
}

fn indication_list_headers() -> Vec<String> {
    vec![
        "DENOMINACIÓN COMÚN INTERNACIONAL (DCI)-MEDICAMENTO".to_owned(),
        "DOSIS / POSOLOGÍA".to_owned(),
        "VÍA DE ADMINISTRACIÓN".to_owned(),
        "FRECUENCIA DE LA ADMINISTRACIÓN".to_owned(),
    ]
}

fn medication_table_full_weights() -> Vec<f32> {
    vec![0.34, 0.12, 0.15, 0.13, 0.14, 0.12]
}

fn medication_table_compact_weights() -> Vec<f32> {
    vec![0.40, 0.20, 0.18, 0.22]
}

fn indication_list_weights() -> Vec<f32> {
    vec![0.42, 0.20, 0.18, 0.20]
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TemplateDocumentV1 {
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

impl From<TemplateDocumentV1> for TemplateDocument {
    fn from(v1: TemplateDocumentV1) -> Self {
        let mut sections: Vec<Section> = vec![
            Section::Letterhead,
            Section::FreeText {
                title: String::new(),
                text: v1.body.clone(),
            },
        ];
        if !v1.diagnosis.trim().is_empty() {
            sections.push(Section::Diagnosis {
                show_allergies: false,
            });
        }
        sections.push(Section::MedicationTable {
            style: MedicationTableStyle::Compact,
            header_captions: medication_table_compact_headers(),
            show_observations: true,
        });
        sections.push(Section::FreeText {
            title: String::new(),
            text: v1.footer.clone(),
        });
        sections.push(Section::SignatureQr);
        let letterhead = Letterhead {
            logo_asset_id: v1.letterhead.logo_asset_id,
            doctor_details: v1.letterhead.doctor_details,
            clinic_details: v1.letterhead.clinic_details,
            form_code: None,
            colors: v1.letterhead.colors,
        };
        Self {
            schema_version: SCHEMA_VERSION,
            page: PageSetup {
                orientation: Orientation::Portrait,
                panel_count: 1,
            },
            letterhead,
            panels: vec![Panel {
                title: String::new(),
                sections,
            }],
            images: v1.images,
            medications: v1.medications,
            diagnosis: v1.diagnosis,
        }
    }
}

pub fn default_document_json() -> String {
    serde_json::to_string(&TemplateDocument::default()).unwrap_or_default()
}

pub fn parse_document(json: &str) -> Result<TemplateDocument, String> {
    if let Ok(v2) = serde_json::from_str::<TemplateDocument>(json) {
        if v2.schema_version == SCHEMA_VERSION {
            validate_document(&v2)?;
            return Ok(v2);
        }
    }
    let v1: TemplateDocumentV1 = serde_json::from_str(json)
        .map_err(|_| "El documento no tiene el formato estructurado esperado.".to_owned())?;
    if v1.schema_version != 1 {
        return Err("La versión del esquema del documento no es compatible.".to_owned());
    }
    let migrated: TemplateDocument = v1.into();
    validate_document(&migrated)?;
    Ok(migrated)
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
    if let Some(form_code) = document.letterhead.form_code.as_deref() {
        check_text(form_code, "código de formulario", 120)?;
    }
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
    if document.page.panel_count < 1 || document.page.panel_count > 2 {
        return Err("El número de paneles debe ser 1 o 2.".to_owned());
    }
    if document.panels.len() != document.page.panel_count as usize {
        return Err("El número de paneles no coincide con la configuración.".to_owned());
    }
    match document.page.orientation {
        Orientation::Portrait if document.page.panel_count != 1 => {
            return Err("La orientación y el número de paneles no coinciden.".to_owned());
        }
        Orientation::Landscape if document.page.panel_count != 2 => {
            return Err("La orientación y el número de paneles no coinciden.".to_owned());
        }
        _ => {}
    }
    for panel in &document.panels {
        if panel.sections.len() > 12 {
            return Err("Cada panel admite como máximo 12 secciones.".to_owned());
        }
        for section in &panel.sections {
            match section {
                Section::FreeText { title, text } => {
                    check_text(title, "título de sección", 120)?;
                    check_text(text, "texto de sección", 40000)?;
                }
                Section::Validity {
                    label_1_day,
                    label_3_days,
                } => {
                    check_text(label_1_day, "etiqueta de vigencia", 120)?;
                    check_text(label_3_days, "etiqueta de vigencia", 120)?;
                }
                Section::MedicationTable {
                    style,
                    header_captions,
                    ..
                } => {
                    let expected = match style {
                        MedicationTableStyle::Full => 6,
                        MedicationTableStyle::Compact => 4,
                    };
                    if header_captions.len() != expected {
                        return Err("El número de columnas de la tabla no es válido.".to_owned());
                    }
                    for caption in header_captions {
                        check_text(caption, "encabezado de tabla", 500)?;
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub fn variable_label(key: &str) -> &'static str {
    match key {
        "patient.name" => "Nombre del paciente",
        "patient.national_id" => "Cédula del paciente",
        "patient.age" => "Edad del paciente",
        "patient.age_detail" => "Edad detallada",
        "patient.sex" => "Sexo del paciente",
        "patient.hcl" => "Historia clínica (HCL)",
        "patient.allergies" => "Alergias del paciente",
        "doctor.name" => "Nombre del médico",
        "doctor.specialty" => "Especialidad del médico",
        "doctor.national_id" => "Cédula del médico",
        "doctor.license_no" => "Registro profesional (ACCESS)",
        "doctor.contact" => "Contacto del médico",
        "clinic.name" => "Nombre de la clínica",
        "clinic.address" => "Dirección de la clínica",
        "clinic.phone" => "Teléfono de la clínica",
        "prescription.number" => "Número de receta",
        "prescription.date" => "Fecha de la receta",
        "prescription.time" => "Hora de emisión",
        "prescription.recetario_type" => "Tipo de recetario",
        "prescription.validity" => "Vigencia",
        "prescription.diagnosis" => "Diagnóstico",
        "prescription.warning_signs" => "Signos de alarma",
        "prescription.instructions" => "Recomendaciones no farmacológicas",
        "medications[]" => "Medicamentos",
        _ => "Variable",
    }
}

pub fn sample_values() -> Vec<(String, String)> {
    vec![
        ("patient.name".to_owned(), "María Fernanda Ruiz".to_owned()),
        ("patient.national_id".to_owned(), "0102030405".to_owned()),
        ("patient.age".to_owned(), "35 años".to_owned()),
        ("patient.age_detail".to_owned(), "35a 2m 10d".to_owned()),
        ("patient.sex".to_owned(), "Femenino".to_owned()),
        ("patient.hcl".to_owned(), "722846-774928".to_owned()),
        ("patient.allergies".to_owned(), "Penicilina".to_owned()),
        ("doctor.name".to_owned(), "Dr. Carlos Pérez".to_owned()),
        ("doctor.specialty".to_owned(), "Medicina General".to_owned()),
        ("doctor.national_id".to_owned(), "0912345678".to_owned()),
        ("doctor.license_no".to_owned(), "1803693264".to_owned()),
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
        ("prescription.time".to_owned(), "13:19".to_owned()),
        (
            "prescription.recetario_type".to_owned(),
            "AMBULATORIO".to_owned(),
        ),
        ("prescription.validity".to_owned(), "3 DIAS".to_owned()),
        (
            "prescription.diagnosis".to_owned(),
            "Diagnóstico de ejemplo".to_owned(),
        ),
        (
            "prescription.warning_signs".to_owned(),
            "Fiebre mayor a 38.5 °C".to_owned(),
        ),
        (
            "prescription.instructions".to_owned(),
            "Reposo e hidratación".to_owned(),
        ),
        (
            "medications[]".to_owned(),
            "Ver tabla de medicamentos".to_owned(),
        ),
    ]
}

pub fn sample_medications() -> Vec<MedicationBlock> {
    vec![
        MedicationBlock {
            name: "Betametasona + Ácido Fusídico (Fusimed B)".to_owned(),
            dose: "1 aplicaciones".to_owned(),
            route: "vía tópica".to_owned(),
            frequency: "cada 12 horas".to_owned(),
            duration: "30 días".to_owned(),
            quantity: "1".to_owned(),
            quantity_number: Some(1),
            notes: "Aplicar poca cantidad en el area afectada 8am y 8pm".to_owned(),
        },
        MedicationBlock {
            name: "Glicolic H Loción".to_owned(),
            dose: "1 aplicaciones".to_owned(),
            route: "vía tópica".to_owned(),
            frequency: "cada 24 horas".to_owned(),
            duration: "30 días".to_owned(),
            quantity: "1".to_owned(),
            quantity_number: Some(1),
            notes: "Aplicar en la mancha del rostro todas las noches".to_owned(),
        },
        MedicationBlock {
            name: "Prednisona 20mg / sólido oral / tabletas (Pharmab)".to_owned(),
            dose: "20 miligramos".to_owned(),
            route: "vía oral".to_owned(),
            frequency: "cada 12 horas".to_owned(),
            duration: "30 días".to_owned(),
            quantity: "40".to_owned(),
            quantity_number: Some(40),
            notes: "2 TABLETAS JUNTAS 7AM DESPUES DEL DESAYUNO 5 DIAS, LUEGO 1 1/2 TABLETAS LUEGO SOLO 1 TABLETA POR 5 DIAS LUEGO MANTENER TRES VECES".to_owned(),
        },
        MedicationBlock {
            name: "Gluconato de Zinc 50mg".to_owned(),
            dose: "1 tableta".to_owned(),
            route: "vía oral".to_owned(),
            frequency: "cada 24 horas".to_owned(),
            duration: "90 días".to_owned(),
            quantity: "90".to_owned(),
            quantity_number: Some(90),
            notes: "1 tableta despues de la merienda".to_owned(),
        },
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
    warnings.extend(resolve_text(&document.letterhead.doctor_details, &values).1);
    warnings.extend(resolve_text(&document.letterhead.clinic_details, &values).1);
    for panel in &document.panels {
        if !panel.title.trim().is_empty() {
            warnings.extend(resolve_text(&panel.title, &values).1);
        }
        for section in &panel.sections {
            match section {
                Section::FreeText { title, text } => {
                    warnings.extend(resolve_text(title, &values).1);
                    warnings.extend(resolve_text(text, &values).1);
                }
                Section::Validity {
                    label_1_day,
                    label_3_days,
                } => {
                    warnings.extend(resolve_text(label_1_day, &values).1);
                    warnings.extend(resolve_text(label_3_days, &values).1);
                }
                Section::PrescriberBlock => {
                    warnings.extend(resolve_text(PRESCRIBER_TEMPLATE, &values).1);
                }
                Section::ReceptionBlock => {}
                _ => {}
            }
        }
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

pub fn quantity_in_words(n: i32) -> String {
    if !(0..=9999).contains(&n) {
        return n.to_string();
    }
    if n == 0 {
        return "CERO".to_owned();
    }
    let _units = [
        "CERO", "UNO", "DOS", "TRES", "CUATRO", "CINCO", "SEIS", "SIETE", "OCHO", "NUEVE",
    ];
    let _teens = [
        "DIEZ",
        "ONCE",
        "DOCE",
        "TRECE",
        "CATORCE",
        "QUINCE",
        "DIECISEIS",
        "DIECISIETE",
        "DIECIOCHO",
        "DIECINUEVE",
    ];
    let _tens = [
        "",
        "",
        "VEINTE",
        "TREINTA",
        "CUARENTA",
        "CINCUENTA",
        "SESENTA",
        "SETENTA",
        "OCHENTA",
        "NOVENTA",
    ];
    let hundreds = [
        "",
        "CIENTO",
        "DOSCIENTOS",
        "TRESCIENTOS",
        "CUATROCIENTOS",
        "QUINIENTOS",
        "SEISCIENTOS",
        "SETECIENTOS",
        "OCHOCIENTOS",
        "NOVECIENTOS",
    ];

    fn below_hundred(n: i32) -> String {
        let units = [
            "CERO", "UNO", "DOS", "TRES", "CUATRO", "CINCO", "SEIS", "SIETE", "OCHO", "NUEVE",
        ];
        let teens = [
            "DIEZ",
            "ONCE",
            "DOCE",
            "TRECE",
            "CATORCE",
            "QUINCE",
            "DIECISEIS",
            "DIECISIETE",
            "DIECIOCHO",
            "DIECINUEVE",
        ];
        let tens = [
            "",
            "",
            "VEINTE",
            "TREINTA",
            "CUARENTA",
            "CINCUENTA",
            "SESENTA",
            "SETENTA",
            "OCHENTA",
            "NOVENTA",
        ];
        if n < 10 {
            units[n as usize].to_owned()
        } else if n < 20 {
            teens[(n - 10) as usize].to_owned()
        } else if n < 30 {
            if n == 20 {
                "VEINTE".to_owned()
            } else {
                format!("VEINTI{}", units[(n - 20) as usize])
            }
        } else {
            let ten = (n / 10) as usize;
            let unit = (n % 10) as usize;
            if unit == 0 {
                tens[ten].to_owned()
            } else {
                format!("{} Y {}", tens[ten], units[unit])
            }
        }
    }

    let mut result = String::new();
    let thousands = n / 1000;
    let remainder = n % 1000;
    if thousands > 0 {
        if thousands == 1 {
            result.push_str("MIL");
        } else {
            result.push_str(&below_hundred(thousands));
            result.push(' ');
            result.push_str("MIL");
        }
        if remainder > 0 {
            result.push(' ');
        }
    }
    if remainder > 0 {
        if remainder == 100 {
            result.push_str("CIEN");
        } else {
            let h = (remainder / 100) as usize;
            let rest = remainder % 100;
            if h > 0 {
                result.push_str(hundreds[h]);
                if rest > 0 {
                    result.push(' ');
                }
            }
            if rest > 0 {
                result.push_str(&below_hundred(rest));
            }
        }
    }
    result
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
pub struct DocumentLayout {
    pub primary: String,
    pub accent: String,
    pub page_width_mm: f32,
    pub page_height_mm: f32,
    pub margin_mm: u32,
    pub pages: Vec<LayoutPage>,
    pub images: Vec<ImageSlot>,
    pub logo_asset_id: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LayoutPage {
    pub panels: Vec<LayoutPanel>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LayoutPanel {
    pub x_mm: f32,
    pub width_mm: f32,
    pub has_letterhead: bool,
    pub blocks: Vec<LayoutBlock>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutBlock {
    Text {
        lines: Vec<String>,
        size_pt: f32,
        bold: bool,
        color: TextColor,
        bold_prefix: Option<String>,
    },
    Rule,
    Table {
        header: Vec<String>,
        rows: Vec<TableRow>,
        col_weights: Vec<f32>,
    },
    Image {
        slot_index: usize,
    },
    Spacer {
        height_mm: f32,
    },
    Qr,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TableRow {
    pub cells: Vec<String>,
    pub span_all: bool,
    pub bold: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextColor {
    Primary,
    Accent,
    Ink,
    Muted,
}

const PRESCRIBER_TEMPLATE: &str = "APELLIDOS Y NOMBRES: {{doctor.name}}\nNÚMERO DE REGISTRO(ACCESS): {{doctor.license_no}}\nNÚMERO DE CONTACTO-PRESCRIPTOR: {{doctor.contact}}";

fn text_width_chars(width_mm: f32, size_pt: f32) -> usize {
    (width_mm / (1.95 * size_pt / 10.0)).floor().max(8.0) as usize
}

fn text_block(lines: Vec<String>, size_pt: f32, bold: bool, color: TextColor) -> LayoutBlock {
    LayoutBlock::Text {
        lines,
        size_pt,
        bold,
        color,
        bold_prefix: None,
    }
}

fn text_block_prefix(
    lines: Vec<String>,
    size_pt: f32,
    color: TextColor,
    prefix: String,
) -> LayoutBlock {
    LayoutBlock::Text {
        lines,
        size_pt,
        bold: false,
        color,
        bold_prefix: Some(prefix),
    }
}

fn block_height(block: &LayoutBlock, width_mm: f32) -> f32 {
    match block {
        LayoutBlock::Text { lines, size_pt, .. } => lines.len() as f32 * size_pt * 0.52,
        LayoutBlock::Rule => 1.0,
        LayoutBlock::Table {
            header,
            rows,
            col_weights,
        } => {
            let cell_widths: Vec<f32> =
                col_weights.iter().map(|weight| width_mm * weight).collect();
            let header_height = header
                .iter()
                .enumerate()
                .map(|(index, caption)| {
                    let cell_width = cell_widths.get(index).copied().unwrap_or(width_mm);
                    let chars = text_width_chars(cell_width, 7.0);
                    let lines = wrap_text(caption, chars).len() as f32;
                    lines * 7.0 * 0.52 + 1.5
                })
                .fold(0.0, f32::max)
                .max(5.0);
            let body_height: f32 = rows
                .iter()
                .map(|row| {
                    if row.span_all {
                        let chars = text_width_chars(width_mm, 8.0);
                        let lines = wrap_text(&row.cells.join(" "), chars).len() as f32;
                        lines * 8.0 * 0.52 + 1.5
                    } else {
                        row.cells
                            .iter()
                            .enumerate()
                            .map(|(index, cell)| {
                                let cell_width =
                                    cell_widths.get(index).copied().unwrap_or(width_mm);
                                let chars = text_width_chars(cell_width, 8.0);
                                let lines = wrap_text(cell, chars).len() as f32;
                                lines * 8.0 * 0.52 + 1.5
                            })
                            .fold(0.0, f32::max)
                            .max(5.0)
                    }
                })
                .sum();
            header_height + body_height
        }
        LayoutBlock::Image { .. } => 31.0,
        LayoutBlock::Spacer { height_mm } => *height_mm,
        LayoutBlock::Qr => 20.0,
    }
}

fn field_label(kind: &FieldKind) -> &'static str {
    match kind {
        FieldKind::RecetarioType => "RECETARIO:",
        FieldKind::IssueDate => "FECHA:",
        FieldKind::IssueTime => "HORA:",
        FieldKind::PatientNationalId => "CI:",
        FieldKind::PatientHcl => "HCL:",
        FieldKind::PatientName => "APELLIDOS Y NOMBRES:",
        FieldKind::PatientAge => "EDAD:",
        FieldKind::PatientSex => "SEXO:",
    }
}

fn field_variable(kind: &FieldKind) -> &'static str {
    match kind {
        FieldKind::RecetarioType => "prescription.recetario_type",
        FieldKind::IssueDate => "prescription.date",
        FieldKind::IssueTime => "prescription.time",
        FieldKind::PatientNationalId => "patient.national_id",
        FieldKind::PatientHcl => "patient.hcl",
        FieldKind::PatientName => "patient.name",
        FieldKind::PatientAge => "patient.age_detail",
        FieldKind::PatientSex => "patient.sex",
    }
}

fn resolve(template: &str, values: &[(String, String)]) -> String {
    resolve_text(template, values).0
}

fn make_letterhead_blocks(
    document: &TemplateDocument,
    values: &[(String, String)],
    width_mm: f32,
) -> Vec<LayoutBlock> {
    let mut blocks = Vec::new();
    let doctor = resolve(&document.letterhead.doctor_details, values);
    let clinic = resolve(&document.letterhead.clinic_details, values);
    let doctor_lines = wrap_text(&doctor, text_width_chars(width_mm, 13.0));
    let mut clinic_lines = wrap_text(&clinic, text_width_chars(width_mm, 8.0));
    if let Some(form_code) = document.letterhead.form_code.as_deref() {
        clinic_lines.push(format!("Código: {form_code}"));
    }
    blocks.push(text_block(doctor_lines, 13.0, true, TextColor::Primary));
    blocks.push(text_block(clinic_lines, 8.0, false, TextColor::Muted));
    blocks.push(LayoutBlock::Rule);
    blocks
}

fn make_patient_fields_blocks(
    fields: &[FieldKind],
    values: &[(String, String)],
    _width_mm: f32,
) -> Vec<LayoutBlock> {
    let mut blocks = Vec::new();
    for field in fields {
        let label = field_label(field);
        let value = resolve(&format!("{{{{{}}}}}", field_variable(field)), values);
        let line = format!("{label} {value}");
        blocks.push(text_block_prefix(
            vec![line],
            8.5,
            TextColor::Ink,
            label.to_owned(),
        ));
    }
    blocks
}

fn make_validity_block(
    label_1_day: &str,
    label_3_days: &str,
    values: &[(String, String)],
) -> LayoutBlock {
    let validity = values
        .iter()
        .find(|(key, _)| key == "prescription.validity")
        .map(|(_, value)| value.as_str())
        .unwrap_or("");
    let check_1 = if validity == label_1_day {
        "[X]"
    } else {
        "[ ]"
    };
    let check_3 = if validity == label_3_days {
        "[X]"
    } else {
        "[ ]"
    };
    let line = format!("VIGENCIA: {check_1} {label_1_day}  {check_3} {label_3_days}");
    text_block_prefix(vec![line], 8.5, TextColor::Ink, "VIGENCIA: ".to_owned())
}

fn quantity_cell(medication: &MedicationBlock) -> String {
    match medication.quantity_number {
        Some(n) => format!("{n} ({})", quantity_in_words(n)),
        None if !medication.quantity.trim().is_empty() => medication.quantity.clone(),
        None => String::new(),
    }
}

fn make_medication_table_full(
    header: &[String],
    medications: &[MedicationBlock],
) -> Option<LayoutBlock> {
    if medications.is_empty() {
        return None;
    }
    let mut rows: Vec<TableRow> = Vec::new();
    for medication in medications {
        if medication.name.trim().is_empty() {
            continue;
        }
        rows.push(TableRow {
            cells: vec![
                medication.name.clone(),
                quantity_cell(medication),
                medication.dose.clone(),
                medication.route.clone(),
                medication.frequency.clone(),
                medication.duration.clone(),
            ],
            span_all: false,
            bold: false,
        });
    }
    if rows.is_empty() {
        return None;
    }
    Some(LayoutBlock::Table {
        header: header.to_vec(),
        rows,
        col_weights: medication_table_full_weights(),
    })
}

fn make_medication_table_compact(
    header: &[String],
    show_observations: bool,
    medications: &[MedicationBlock],
) -> Option<LayoutBlock> {
    if medications.is_empty() {
        return None;
    }
    let mut rows: Vec<TableRow> = Vec::new();
    for medication in medications {
        if medication.name.trim().is_empty() {
            continue;
        }
        rows.push(TableRow {
            cells: vec![
                medication.name.clone(),
                medication.dose.clone(),
                medication.route.clone(),
                medication.frequency.clone(),
            ],
            span_all: false,
            bold: false,
        });
        if show_observations && !medication.notes.trim().is_empty() {
            rows.push(TableRow {
                cells: vec![format!("Observación: {}", medication.notes)],
                span_all: true,
                bold: false,
            });
        }
    }
    if rows.is_empty() {
        return None;
    }
    Some(LayoutBlock::Table {
        header: header.to_vec(),
        rows,
        col_weights: medication_table_compact_weights(),
    })
}

fn make_indication_list(
    show_observations: bool,
    medications: &[MedicationBlock],
) -> Option<LayoutBlock> {
    if medications.is_empty() {
        return None;
    }
    let mut rows: Vec<TableRow> = Vec::new();
    for (position, medication) in medications.iter().enumerate() {
        if medication.name.trim().is_empty() {
            continue;
        }
        rows.push(TableRow {
            cells: vec![
                format!("{}) {}", position + 1, medication.name),
                medication.dose.clone(),
                medication.route.clone(),
                medication.frequency.clone(),
            ],
            span_all: false,
            bold: false,
        });
        if show_observations && !medication.notes.trim().is_empty() {
            rows.push(TableRow {
                cells: vec![format!("Observación: {}", medication.notes)],
                span_all: true,
                bold: false,
            });
        }
    }
    if rows.is_empty() {
        return None;
    }
    Some(LayoutBlock::Table {
        header: indication_list_headers(),
        rows,
        col_weights: indication_list_weights(),
    })
}

fn make_section_blocks(
    section: &Section,
    document: &TemplateDocument,
    values: &[(String, String)],
    width_mm: f32,
    medications: &[MedicationBlock],
) -> Vec<LayoutBlock> {
    match section {
        Section::Letterhead => make_letterhead_blocks(document, values, width_mm),
        Section::PatientFields { fields } => make_patient_fields_blocks(fields, values, width_mm),
        Section::Validity {
            label_1_day,
            label_3_days,
        } => vec![make_validity_block(label_1_day, label_3_days, values)],
        Section::Diagnosis { show_allergies } => {
            let mut blocks = Vec::new();
            let diagnosis = resolve("{{prescription.diagnosis}}", values);
            blocks.push(text_block(
                vec!["DIAGNÓSTICO:".to_owned()],
                8.5,
                true,
                TextColor::Ink,
            ));
            blocks.push(text_block(
                wrap_text(&diagnosis, text_width_chars(width_mm, 8.5)),
                8.5,
                false,
                TextColor::Ink,
            ));
            if *show_allergies {
                let allergies = resolve("{{patient.allergies}}", values);
                blocks.push(text_block(
                    vec!["ALERGIAS:".to_owned()],
                    8.5,
                    true,
                    TextColor::Ink,
                ));
                blocks.push(text_block(
                    wrap_text(&allergies, text_width_chars(width_mm, 8.5)),
                    8.5,
                    false,
                    TextColor::Ink,
                ));
            }
            blocks
        }
        Section::FreeText { title, text } => {
            let mut blocks = Vec::new();
            let resolved_title = resolve(title, values);
            if !resolved_title.trim().is_empty() {
                blocks.push(text_block(
                    vec![resolved_title],
                    9.0,
                    true,
                    TextColor::Primary,
                ));
            }
            let resolved_text = resolve(text, values);
            if !resolved_text.trim().is_empty() {
                blocks.push(text_block(
                    wrap_text(&resolved_text, text_width_chars(width_mm, 9.0)),
                    9.0,
                    false,
                    TextColor::Ink,
                ));
            }
            blocks
        }
        Section::MedicationTable {
            style,
            header_captions,
            show_observations,
        } => {
            let block = match style {
                MedicationTableStyle::Full => {
                    make_medication_table_full(header_captions, medications)
                }
                MedicationTableStyle::Compact => {
                    make_medication_table_compact(header_captions, *show_observations, medications)
                }
            };
            block.into_iter().collect()
        }
        Section::IndicationList { show_observations } => {
            make_indication_list(*show_observations, medications)
                .into_iter()
                .collect()
        }
        Section::WarningSigns => {
            let value = resolve("{{prescription.warning_signs}}", values);
            vec![
                text_block(
                    vec!["SIGNOS DE ALARMA:".to_owned()],
                    8.5,
                    true,
                    TextColor::Ink,
                ),
                text_block(
                    wrap_text(&value, text_width_chars(width_mm, 8.5)),
                    8.5,
                    false,
                    TextColor::Ink,
                ),
            ]
        }
        Section::Recommendations => {
            let value = resolve("{{prescription.instructions}}", values);
            vec![
                text_block(
                    vec!["RECOMENDACIONES NO FARMACOLÓGICAS:".to_owned()],
                    8.5,
                    true,
                    TextColor::Ink,
                ),
                text_block(
                    wrap_text(&value, text_width_chars(width_mm, 8.5)),
                    8.5,
                    false,
                    TextColor::Ink,
                ),
            ]
        }
        Section::PrescriberBlock => {
            let mut blocks = Vec::new();
            blocks.push(text_block(
                vec!["DATOS DEL PRESCRIPTOR:".to_owned()],
                8.5,
                true,
                TextColor::Ink,
            ));
            let text = resolve(PRESCRIBER_TEMPLATE, values);
            let lines = wrap_text(&text, text_width_chars(width_mm, 8.5));
            for line in lines {
                if line.starts_with("APELLIDOS Y NOMBRES: ") {
                    blocks.push(text_block_prefix(
                        vec![line],
                        8.5,
                        TextColor::Ink,
                        "APELLIDOS Y NOMBRES: ".to_owned(),
                    ));
                } else if line.starts_with("NÚMERO DE REGISTRO(ACCESS): ") {
                    blocks.push(text_block_prefix(
                        vec![line],
                        8.5,
                        TextColor::Ink,
                        "NÚMERO DE REGISTRO(ACCESS): ".to_owned(),
                    ));
                } else if line.starts_with("NÚMERO DE CONTACTO-PRESCRIPTOR: ") {
                    blocks.push(text_block_prefix(
                        vec![line],
                        8.5,
                        TextColor::Ink,
                        "NÚMERO DE CONTACTO-PRESCRIPTOR: ".to_owned(),
                    ));
                } else {
                    blocks.push(text_block(vec![line], 8.5, false, TextColor::Ink));
                }
            }
            blocks.push(text_block_prefix(
                vec!["FIRMA:".to_owned()],
                8.5,
                TextColor::Ink,
                "FIRMA:".to_owned(),
            ));
            blocks.push(LayoutBlock::Spacer { height_mm: 12.0 });
            blocks.push(text_block_prefix(
                vec!["SELLO MÉDICO:".to_owned()],
                8.5,
                TextColor::Ink,
                "SELLO MÉDICO:".to_owned(),
            ));
            blocks.push(LayoutBlock::Spacer { height_mm: 18.0 });
            blocks
        }
        Section::ReceptionBlock => {
            vec![
                text_block(
                    vec!["RECEPCIÓN MEDICACIÓN DEL PACIENTE:".to_owned()],
                    8.5,
                    true,
                    TextColor::Ink,
                ),
                text_block(
                    vec!["FIRMA: __________________    N° CÉDULA: __________________    PARENTESCO: __________________".to_owned()],
                    8.0,
                    false,
                    TextColor::Ink,
                ),
            ]
        }
        Section::SignatureQr => vec![LayoutBlock::Rule],
    }
}

fn footer_text(_values: &[(String, String)], meta: &DocumentMeta) -> Vec<String> {
    let mut lines = Vec::new();
    if let (Some(number), Some(date)) = (meta.rx_number, meta.issued_at.as_deref()) {
        lines.push(format!("Receta No. {number:06} · {date}"));
    } else if let Some(date) = meta.issued_at.as_deref() {
        lines.push(format!("Fecha de emisión: {date}"));
    }
    if meta.provisional {
        lines.push("PENDIENTE DE SINCRONIZACIÓN".to_owned());
    } else if let Some(code) = meta.verification_code.as_deref() {
        lines.push(format!("Código de verificación: {code}"));
    }
    lines.push("Firma: ______________________________".to_owned());
    lines
}

fn make_signature_qr_blocks(values: &[(String, String)], meta: &DocumentMeta) -> Vec<LayoutBlock> {
    let mut blocks = Vec::new();
    let footer_lines = footer_text(values, meta);
    blocks.push(text_block(footer_lines, 7.5, false, TextColor::Muted));
    blocks.push(LayoutBlock::Qr);
    blocks
}

fn is_fixed_section(section: &Section) -> bool {
    matches!(
        section,
        Section::Letterhead | Section::PatientFields { .. } | Section::Validity { .. }
    )
}

pub fn paginate_document(
    document: &TemplateDocument,
    values: &[(String, String)],
    margin_mm: u32,
    medications: &[MedicationBlock],
    meta: &DocumentMeta,
) -> DocumentLayout {
    let margin = margin_mm.clamp(5, 40);
    let (page_width, page_height, panels_geometry) = match document.page.orientation {
        Orientation::Portrait => {
            let width = 210.0;
            let height = 297.0;
            let panel_width = width - 2.0 * margin as f32;
            (width, height, vec![(margin as f32, panel_width)])
        }
        Orientation::Landscape => {
            let width = 297.0;
            let height = 210.0;
            let gutter = 6.0;
            let panel_width = (width - 2.0 * margin as f32 - gutter) / 2.0;
            (
                width,
                height,
                vec![
                    (margin as f32, panel_width),
                    (margin as f32 + panel_width + gutter, panel_width),
                ],
            )
        }
    };

    let meds = if medications.is_empty() {
        &document.medications
    } else {
        medications
    };

    let mut all_panel_blocks: Vec<Vec<LayoutBlock>> = Vec::new();
    for (panel_index, (panel, (_x, width))) in document
        .panels
        .iter()
        .zip(panels_geometry.iter())
        .enumerate()
    {
        let mut blocks: Vec<LayoutBlock> = Vec::new();
        let mut table_title_emitted = false;
        for section in &panel.sections {
            let section_blocks = make_section_blocks(section, document, values, *width, meds);
            if matches!(
                section,
                Section::MedicationTable { .. } | Section::IndicationList { .. }
            ) && !panel.title.trim().is_empty()
                && !table_title_emitted
                && !section_blocks.is_empty()
            {
                blocks.push(text_block(
                    vec![panel.title.clone()],
                    10.0,
                    true,
                    TextColor::Primary,
                ));
                table_title_emitted = true;
            }
            blocks.extend(section_blocks);
            if matches!(section, Section::Letterhead)
                && !document.images.is_empty()
                && panel_index == 0
            {
                for index in 0..document.images.len() {
                    blocks.push(LayoutBlock::Image { slot_index: index });
                }
            }
        }
        if !panel.title.trim().is_empty() && !table_title_emitted {
            blocks.push(text_block(
                vec![panel.title.clone()],
                10.0,
                true,
                TextColor::Primary,
            ));
        }
        all_panel_blocks.push(blocks);
    }

    let usable_height = page_height - 2.0 * margin as f32;

    let mut pages: Vec<LayoutPage> = Vec::new();
    let mut panel_positions: Vec<usize> = vec![0; all_panel_blocks.len()];

    loop {
        let mut current_panel_blocks: Vec<Vec<LayoutBlock>> = Vec::new();
        let mut any_remaining = false;
        for (panel_index, blocks) in all_panel_blocks.iter().enumerate() {
            let pos = panel_positions[panel_index];
            let (_x, width) = panels_geometry[panel_index];
            let mut page_blocks: Vec<LayoutBlock> = Vec::new();
            let mut height = 0.0;
            let has_letterhead = document.panels[panel_index]
                .sections
                .first()
                .is_some_and(|section| matches!(section, Section::Letterhead));

            if pos == 0 && has_letterhead {
                for section in &document.panels[panel_index].sections {
                    if is_fixed_section(section) {
                        let fixed_blocks =
                            make_section_blocks(section, document, values, width, meds);
                        for block in fixed_blocks {
                            let block_h = block_height(&block, width);
                            if height + block_h > usable_height {
                                break;
                            }
                            height += block_h;
                            page_blocks.push(block);
                        }
                    } else {
                        break;
                    }
                }
            }

            let mut index = pos;
            while index < blocks.len() {
                let block = &blocks[index];
                let block_h = block_height(block, width);
                if height + block_h > usable_height {
                    break;
                }
                height += block_h;
                page_blocks.push(block.clone());
                index += 1;
            }

            if index < blocks.len() {
                any_remaining = true;
            }
            current_panel_blocks.push(page_blocks);
            panel_positions[panel_index] = index;
        }

        if current_panel_blocks.iter().all(|blocks| blocks.is_empty()) && !any_remaining {
            break;
        }

        if pages.is_empty() || current_panel_blocks.iter().any(|blocks| !blocks.is_empty()) {
            let panels: Vec<LayoutPanel> = current_panel_blocks
                .into_iter()
                .enumerate()
                .map(|(index, blocks)| {
                    let has_letterhead = document.panels[index]
                        .sections
                        .first()
                        .is_some_and(|section| matches!(section, Section::Letterhead));
                    LayoutPanel {
                        x_mm: panels_geometry[index].0,
                        width_mm: panels_geometry[index].1,
                        has_letterhead,
                        blocks,
                    }
                })
                .collect();
            pages.push(LayoutPage { panels });
        }

        if !any_remaining {
            break;
        }
    }

    if pages.is_empty() {
        pages.push(LayoutPage {
            panels: panels_geometry
                .iter()
                .map(|(x, width)| LayoutPanel {
                    x_mm: *x,
                    width_mm: *width,
                    has_letterhead: false,
                    blocks: Vec::new(),
                })
                .collect(),
        });
    }

    let page_total = pages.len();
    for (page_index, page) in pages.iter_mut().enumerate() {
        let is_last = page_index + 1 == page_total;
        for (panel_index, panel) in page.panels.iter_mut().enumerate() {
            let has_signature_qr = document.panels[panel_index]
                .sections
                .iter()
                .any(|section| matches!(section, Section::SignatureQr));
            if is_last && has_signature_qr {
                panel.blocks.extend(make_signature_qr_blocks(values, meta));
            }
        }
    }

    DocumentLayout {
        primary: document.letterhead.colors.primary.clone(),
        accent: document.letterhead.colors.accent.clone(),
        page_width_mm: page_width,
        page_height_mm: page_height,
        margin_mm: margin,
        pages,
        images: document.images.clone(),
        logo_asset_id: document.letterhead.logo_asset_id.clone(),
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
    let effective_medications = medications.unwrap_or_else(|| document.medications.clone());
    let has_tables = document.panels.iter().any(|panel| {
        panel.sections.iter().any(|section| {
            matches!(
                section,
                Section::MedicationTable { .. } | Section::IndicationList { .. }
            )
        })
    });
    let preview_medications = if effective_medications.is_empty() && has_tables {
        sample_medications()
    } else {
        effective_medications
    };
    let mut header_warnings = Vec::new();
    header_warnings.extend(resolve_text(&document.letterhead.doctor_details, &values).1);
    header_warnings.extend(resolve_text(&document.letterhead.clinic_details, &values).1);
    for panel in &document.panels {
        if !panel.title.trim().is_empty() {
            header_warnings.extend(resolve_text(&panel.title, &values).1);
        }
        for section in &panel.sections {
            match section {
                Section::FreeText { title, text } => {
                    header_warnings.extend(resolve_text(title, &values).1);
                    header_warnings.extend(resolve_text(text, &values).1);
                }
                Section::Validity {
                    label_1_day,
                    label_3_days,
                } => {
                    header_warnings.extend(resolve_text(label_1_day, &values).1);
                    header_warnings.extend(resolve_text(label_3_days, &values).1);
                }
                _ => {}
            }
        }
    }
    let layout = paginate_document(&document, &values, margin_mm, &preview_medications, &meta);
    let total = layout.pages.len();
    let sources: BTreeMap<String, String> = image_sources.into_iter().collect();
    let qr = meta.qr_data_url.clone();
    let provisional = meta.provisional;
    let landscape_class = if layout.page_width_mm > layout.page_height_mm {
        "a4-page--landscape"
    } else {
        ""
    };
    let warning_text = header_warnings.join(", ");

    rsx! {
        div {
            class: "a4-pages",
            for (index, page) in layout.pages.iter().enumerate() {
                div {
                    key: "{index}",
                    class: "a4-page {landscape_class}",
                    style: "padding: {layout.margin_mm}mm;",
                    div {
                        class: "a4-panels",
                        for panel in page.panels.iter() {
                            div {
                                class: "a4-panel",
                                style: "width: {panel.width_mm}mm;",
                                for block in panel.blocks.iter() {
                                    { render_layout_block(block, &layout, panel, &sources, qr.as_ref(), provisional) }
                                }
                            }
                        }
                    }
                    p { class: "a4-page-no", "Página {index + 1} de {total}" }
                }
            }
            if !header_warnings.is_empty() {
                div {
                    class: "a4-warnings",
                    p { "Variables no válidas en la cabecera o el pie: {warning_text}" }
                }
            }
        }
    }
}

fn render_layout_block(
    block: &LayoutBlock,
    layout: &DocumentLayout,
    _panel: &LayoutPanel,
    sources: &BTreeMap<String, String>,
    qr: Option<&String>,
    provisional: bool,
) -> Element {
    match block {
        LayoutBlock::Text {
            lines,
            size_pt,
            bold,
            color,
            bold_prefix,
        } => {
            let color_hex = match color {
                TextColor::Primary => layout.primary.clone(),
                TextColor::Accent => layout.accent.clone(),
                TextColor::Ink => "#111111".to_owned(),
                TextColor::Muted => "#555555".to_owned(),
            };
            let bold_class = if *bold { "a4-text--bold" } else { "" };
            rsx! {
                div {
                    class: "a4-text",
                    for line in lines.iter() {
                        p {
                            class: "{bold_class}",
                            style: "font-size: {size_pt}pt; color: {color_hex};",
                            if let Some(prefix) = bold_prefix {
                                span { class: "a4-text--bold", "{prefix}" }
                                "{line.strip_prefix(prefix).unwrap_or(line)}"
                            } else {
                                "{line}"
                            }
                        }
                    }
                }
            }
        }
        LayoutBlock::Rule => {
            let color = layout.primary.clone();
            rsx! {
                div { class: "a4-rule", style: "border-color: {color};" }
            }
        }
        LayoutBlock::Table {
            header,
            rows,
            col_weights,
        } => {
            let count = header.len();
            rsx! {
                table { class: "a4-table",
                    colgroup {
                        for weight in col_weights.iter() {
                            col { style: "width: {weight * 100.0}%;" }
                        }
                    }
                    thead {
                        tr {
                            for caption in header.iter() {
                                th { "{caption}" }
                            }
                        }
                    }
                    tbody {
                        for row in rows.iter() {
                            if row.span_all {
                                tr {
                                    td { colspan: "{count}",
                                        if row.bold {
                                            strong { "{row.cells.join(\" \")}" }
                                        } else {
                                            "{row.cells.join(\" \")}"
                                        }
                                    }
                                }
                            } else {
                                tr {
                                    for cell in row.cells.iter() {
                                        td {
                                            if row.bold {
                                                strong { "{cell}" }
                                            } else {
                                                "{cell}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        LayoutBlock::Image { slot_index } => {
            if let Some(image) = layout.images.get(*slot_index) {
                if let Some(source) = image.asset_id.as_ref().and_then(|id| sources.get(id)) {
                    rsx! {
                        div { class: "a4-image-wrap",
                            img { class: "a4-image", src: "{source}", alt: "{image.caption}" }
                            if !image.caption.trim().is_empty() {
                                p { class: "a4-image-caption", "{image.caption}" }
                            }
                        }
                    }
                } else {
                    rsx! {
                        div { class: "a4-image-wrap",
                            p { "Imagen: {image.caption}" }
                        }
                    }
                }
            } else {
                rsx! {}
            }
        }
        LayoutBlock::Spacer { height_mm } => {
            rsx! {
                div { style: "height: {height_mm}mm;" }
            }
        }
        LayoutBlock::Qr => {
            let qr_size = if layout
                .pages
                .first()
                .is_some_and(|page| page.panels.len() > 1)
            {
                18.0
            } else {
                25.0
            };
            rsx! {
                div {
                    class: "a4-qr",
                    style: "width: {qr_size}mm; height: {qr_size}mm;",
                    if provisional {
                        div { class: "a4-qr-placeholder", "QR provisional" }
                    } else if let Some(source) = qr {
                        img { src: "{source}", alt: "Código QR de verificación" }
                    } else {
                        div { class: "a4-qr-placeholder", "QR de verificación" }
                    }
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

fn section_label(section: &Section) -> &'static str {
    match section {
        Section::Letterhead => "Membrete",
        Section::PatientFields { .. } => "Datos del paciente",
        Section::Validity { .. } => "Vigencia",
        Section::Diagnosis { .. } => "Diagnóstico",
        Section::FreeText { .. } => "Texto libre",
        Section::MedicationTable { .. } => "Tabla de medicamentos",
        Section::IndicationList { .. } => "Lista de indicaciones",
        Section::WarningSigns => "Signos de alarma",
        Section::Recommendations => "Recomendaciones",
        Section::PrescriberBlock => "Datos del prescriptor",
        Section::ReceptionBlock => "Recepción",
        Section::SignatureQr => "Firma y QR",
    }
}

const ALL_FIELD_KINDS: [FieldKind; 8] = [
    FieldKind::RecetarioType,
    FieldKind::IssueDate,
    FieldKind::IssueTime,
    FieldKind::PatientNationalId,
    FieldKind::PatientHcl,
    FieldKind::PatientName,
    FieldKind::PatientAge,
    FieldKind::PatientSex,
];

fn default_section(type_value: &str) -> Section {
    match type_value {
        "letterhead" => Section::Letterhead,
        "patient_fields" => Section::PatientFields {
            fields: ALL_FIELD_KINDS.to_vec(),
        },
        "validity" => Section::Validity {
            label_1_day: "1 DIA".to_owned(),
            label_3_days: "3 DIAS".to_owned(),
        },
        "diagnosis" => Section::Diagnosis {
            show_allergies: false,
        },
        "medication_table" => Section::MedicationTable {
            style: MedicationTableStyle::Compact,
            header_captions: medication_table_compact_headers(),
            show_observations: true,
        },
        "indication_list" => Section::IndicationList {
            show_observations: true,
        },
        "warning_signs" => Section::WarningSigns,
        "recommendations" => Section::Recommendations,
        "prescriber_block" => Section::PrescriberBlock,
        "reception_block" => Section::ReceptionBlock,
        "signature_qr" => Section::SignatureQr,
        _ => Section::FreeText {
            title: String::new(),
            text: String::new(),
        },
    }
}

fn panel_is_non_empty(panel: &Panel) -> bool {
    !panel.title.trim().is_empty()
        || panel
            .sections
            .iter()
            .any(|section| !matches!(section, Section::Letterhead))
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
    let mut picker = use_signal(String::new);
    let mut new_section_type = use_signal(|| "free_text".to_owned());
    let panel_two_non_empty = document()
        .panels
        .get(1)
        .map(panel_is_non_empty)
        .unwrap_or(false);
    let message = use_signal(String::new);
    let saving = use_signal(|| false);
    let library = use_resource(|| async move { api::fetch_assets().await });
    let library_rows = match library() {
        Some(Ok(library)) => library.assets,
        _ => Vec::new(),
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
                TextField {
                    label: "Código de formulario".to_owned(),
                    value: document().letterhead.form_code.clone().unwrap_or_default(),
                    oninput: move |value: String| {
                        document.write().letterhead.form_code =
                            if value.is_empty() { None } else { Some(value) };
                    },
                    input_type: "text".to_owned(),
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
        SectionCard { title: "Página".to_owned(), description: None,
            FormGrid { columns: FormColumns::Two,
                SelectField {
                    label: "Orientación".to_owned(),
                    value: match document().page.orientation {
                        Orientation::Portrait => "portrait".to_owned(),
                        Orientation::Landscape => "landscape".to_owned(),
                    },
                    onchange: move |value: String| {
                        let mut draft = document.write();
                        match value.as_str() {
                            "landscape" => {
                                draft.page.orientation = Orientation::Landscape;
                                draft.page.panel_count = 2;
                                if draft.panels.len() < 2 {
                                    draft.panels.push(Panel {
                                        title: String::new(),
                                        sections: vec![Section::Letterhead],
                                    });
                                }
                            }
                            _ => {
                                if draft.panels.get(1).map(panel_is_non_empty).unwrap_or(false) {
                                    return;
                                }
                                draft.page.orientation = Orientation::Portrait;
                                draft.page.panel_count = 1;
                                draft.panels.truncate(1);
                            }
                        }
                    },
                    wide: false,
                    hint: None,
                    error: None,
                    disabled: false,
                    hide_label: false,
                    option { value: "portrait", "Retrato" }
                    option { value: "landscape", "Paisaje" }
                }
                SelectField {
                    label: "Paneles".to_owned(),
                    value: document().page.panel_count.to_string(),
                    onchange: move |value: String| {
                        if let Ok(count) = value.parse::<u32>() {
                            let count = count.clamp(1, 2);
                            let mut draft = document.write();
                            if count == 1
                                && draft.panels.get(1).map(panel_is_non_empty).unwrap_or(false)
                            {
                                return;
                            }
                            draft.page.panel_count = count;
                            while draft.panels.len() < count as usize {
                                draft.panels.push(Panel {
                                    title: String::new(),
                                    sections: vec![Section::Letterhead],
                                });
                            }
                            if draft.panels.len() > count as usize {
                                draft.panels.truncate(count as usize);
                            }
                        }
                    },
                    wide: false,
                    hint: if panel_two_non_empty {
                        Some("Vacía el panel 2 antes de reducir.".to_owned())
                    } else {
                        None
                    },
                    error: None,
                    disabled: false,
                    hide_label: false,
                    option { value: "1", disabled: panel_two_non_empty, "1" }
                    option { value: "2", "2" }
                }
            }
        }
        for panel_index in 0..document().panels.len() {
            SectionCard {
                title: format!(
                    "Panel {} — {}",
                    panel_index + 1,
                    document().panels[panel_index].title
                ),
                description: None,
                FormGrid { columns: FormColumns::Two,
                    TextField {
                        label: "Título del panel".to_owned(),
                        value: document().panels[panel_index].title.clone(),
                        oninput: move |value| {
                            document.write().panels[panel_index].title = value;
                        },
                        input_type: "text".to_owned(),
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
                for section_index in 0..document().panels[panel_index].sections.len() {
                    Card {
                        key: "section-{panel_index}-{section_index}",
                        p { class: "ui-text-subheading", "{section_label(&document().panels[panel_index].sections[section_index])}" }
                        match document().panels[panel_index].sections[section_index].clone() {
                            Section::FreeText { title, text } => rsx! {
                                TextField {
                                    label: "Título".to_owned(),
                                    value: title,
                                    oninput: move |value| {
                                        if let Section::FreeText { title, .. } =
                                            &mut document.write().panels[panel_index].sections[section_index]
                                        {
                                            *title = value;
                                        }
                                    },
                                    input_type: "text".to_owned(),
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
                                TextAreaField {
                                    label: "Texto".to_owned(),
                                    value: text,
                                    oninput: move |value| {
                                        if let Section::FreeText { text, .. } =
                                            &mut document.write().panels[panel_index].sections[section_index]
                                        {
                                            *text = value;
                                        }
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
                                SelectField {
                                    label: "Variable".to_owned(),
                                    value: picker(),
                                    onchange: move |value: String| picker.set(value),
                                    wide: false,
                                    hint: None,
                                    error: None,
                                    disabled: false,
                                    hide_label: false,
                                    option { value: "", "Elige una variable" }
                                    for variable in TEMPLATE_VARIABLES {
                                        option { value: "{variable}", "{variable_label(variable)} — {variable}" }
                                    }
                                }
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    onclick: move |_| {
                                        if !picker().is_empty() {
                                            if let Section::FreeText { text, .. } =
                                                &mut document.write().panels[panel_index].sections[section_index]
                                            {
                                                text.push_str(&format!("{{{{{}}}}}", picker()));
                                            }
                                        }
                                    },
                                    "Insertar variable"
                                }
                            },
                            Section::PatientFields { fields } => rsx! {
                                for kind in ALL_FIELD_KINDS {
                                    CheckboxField {
                                        label: variable_label(field_variable(&kind)).to_owned(),
                                        description: None,
                                        checked: fields.contains(&kind),
                                        onchange: move |checked: bool| {
                                            let mut draft = document.write();
                                            if let Section::PatientFields { fields } =
                                                &mut draft.panels[panel_index].sections[section_index]
                                            {
                                                if checked {
                                                    if !fields.contains(&kind) {
                                                        fields.push(kind);
                                                    }
                                                } else {
                                                    fields.retain(|field| *field != kind);
                                                }
                                            }
                                        },
                                        disabled: false,
                                        wide: false,
                                    }
                                }
                            },
                            Section::Validity { label_1_day, label_3_days } => rsx! {
                                TextField {
                                    label: "Etiqueta 1 día".to_owned(),
                                    value: label_1_day,
                                    oninput: move |value| {
                                        if let Section::Validity { label_1_day, .. } =
                                            &mut document.write().panels[panel_index].sections[section_index]
                                        {
                                            *label_1_day = value;
                                        }
                                    },
                                    input_type: "text".to_owned(),
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
                                    label: "Etiqueta 3 días".to_owned(),
                                    value: label_3_days,
                                    oninput: move |value| {
                                        if let Section::Validity { label_3_days, .. } =
                                            &mut document.write().panels[panel_index].sections[section_index]
                                        {
                                            *label_3_days = value;
                                        }
                                    },
                                    input_type: "text".to_owned(),
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
                            },
                            Section::Diagnosis { show_allergies } => rsx! {
                                CheckboxField {
                                    label: "Mostrar alergias".to_owned(),
                                    description: None,
                                    checked: show_allergies,
                                    onchange: move |checked: bool| {
                                        if let Section::Diagnosis { show_allergies } =
                                            &mut document.write().panels[panel_index].sections[section_index]
                                        {
                                            *show_allergies = checked;
                                        }
                                    },
                                    disabled: false,
                                    wide: false,
                                }
                            },
                            Section::MedicationTable { style, header_captions, show_observations } => rsx! {
                                SelectField {
                                    label: "Estilo".to_owned(),
                                    value: match style {
                                        MedicationTableStyle::Full => "full".to_owned(),
                                        MedicationTableStyle::Compact => "compact".to_owned(),
                                    },
                                    onchange: move |value: String| {
                                        let mut draft = document.write();
                                        if let Section::MedicationTable { style, header_captions, .. } =
                                            &mut draft.panels[panel_index].sections[section_index]
                                        {
                                            match value.as_str() {
                                                "full" => {
                                                    *style = MedicationTableStyle::Full;
                                                    *header_captions = medication_table_full_headers();
                                                }
                                                _ => {
                                                    *style = MedicationTableStyle::Compact;
                                                    *header_captions = medication_table_compact_headers();
                                                }
                                            }
                                        }
                                    },
                                    wide: false,
                                    hint: None,
                                    error: None,
                                    disabled: false,
                                    hide_label: false,
                                    option { value: "full", "Completa (6 columnas)" }
                                    option { value: "compact", "Compacta (4 columnas)" }
                                }
                                for caption_index in 0..header_captions.len() {
                                    TextField {
                                        label: format!("Columna {}", caption_index + 1),
                                        value: header_captions[caption_index].clone(),
                                        oninput: move |value| {
                                            if let Section::MedicationTable { header_captions, .. } =
                                                &mut document.write().panels[panel_index].sections[section_index]
                                            {
                                                if caption_index < header_captions.len() {
                                                    header_captions[caption_index] = value;
                                                }
                                            }
                                        },
                                        input_type: "text".to_owned(),
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
                                CheckboxField {
                                    label: "Mostrar observaciones".to_owned(),
                                    description: None,
                                    checked: show_observations,
                                    onchange: move |checked: bool| {
                                        if let Section::MedicationTable { show_observations, .. } =
                                            &mut document.write().panels[panel_index].sections[section_index]
                                        {
                                            *show_observations = checked;
                                        }
                                    },
                                    disabled: false,
                                    wide: false,
                                }
                            },
                            Section::IndicationList { show_observations } => rsx! {
                                CheckboxField {
                                    label: "Mostrar observaciones".to_owned(),
                                    description: None,
                                    checked: show_observations,
                                    onchange: move |checked: bool| {
                                        if let Section::IndicationList { show_observations } =
                                            &mut document.write().panels[panel_index].sections[section_index]
                                        {
                                            *show_observations = checked;
                                        }
                                    },
                                    disabled: false,
                                    wide: false,
                                }
                            },
                            Section::Letterhead
                            | Section::WarningSigns
                            | Section::Recommendations
                            | Section::PrescriberBlock
                            | Section::ReceptionBlock
                            | Section::SignatureQr => rsx! {
                                p { class: "ui-text-body-sm", "Sin campos editables." }
                            },
                        }
                        ActionRow {
                            Button {
                                variant: ButtonVariant::Secondary,
                                disabled: section_index == 0,
                                onclick: move |_| {
                                    if section_index > 0 {
                                        document.write().panels[panel_index].sections.swap(section_index - 1, section_index);
                                    }
                                },
                                "Subir"
                            }
                            Button {
                                variant: ButtonVariant::Secondary,
                                disabled: section_index + 1
                                    >= document().panels[panel_index].sections.len(),
                                onclick: move |_| {
                                    let len = document.write().panels[panel_index].sections.len();
                                    if section_index + 1 < len {
                                        document.write().panels[panel_index].sections.swap(section_index, section_index + 1);
                                    }
                                },
                                "Bajar"
                            }
                            Button {
                                variant: ButtonVariant::Danger,
                                onclick: move |_| {
                                    document.write().panels[panel_index].sections.remove(section_index);
                                },
                                "Quitar"
                            }
                        }
                    }
                }
                FormGrid { columns: FormColumns::Two,
                    SelectField {
                        label: "Tipo de sección".to_owned(),
                        value: new_section_type(),
                        onchange: move |value: String| new_section_type.set(value),
                        wide: false,
                        hint: None,
                        error: None,
                        disabled: false,
                        hide_label: false,
                        option { value: "letterhead", "Membrete" }
                        option { value: "patient_fields", "Datos del paciente" }
                        option { value: "validity", "Vigencia" }
                        option { value: "diagnosis", "Diagnóstico" }
                        option { value: "free_text", "Texto libre" }
                        option { value: "medication_table", "Tabla de medicamentos" }
                        option { value: "indication_list", "Lista de indicaciones" }
                        option { value: "warning_signs", "Signos de alarma" }
                        option { value: "recommendations", "Recomendaciones" }
                        option { value: "prescriber_block", "Datos del prescriptor" }
                        option { value: "reception_block", "Recepción" }
                        option { value: "signature_qr", "Firma y QR" }
                    }
                }
                ActionRow {
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| {
                            let chosen = new_section_type();
                            document.write().panels[panel_index].sections.push(default_section(&chosen));
                        },
                        "Agregar sección"
                    }
                }
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
                            label: "Cantidad (número)".to_owned(),
                            value: document().medications[index].quantity_number.map(|n| n.to_string()).unwrap_or_default(),
                            oninput: move |value: String| {
                                document.write().medications[index].quantity_number =
                                    value.trim().parse::<i32>().ok();
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
