use dioxus::prelude::*;

use super::{Icon, IconName};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccentTone {
    #[default]
    Mint,
    Cream,
    Blossom,
    Conic,
}

impl AccentTone {
    fn suffix(self) -> &'static str {
        match self {
            AccentTone::Mint => "mint",
            AccentTone::Cream => "cream",
            AccentTone::Blossom => "blossom",
            AccentTone::Conic => "conic",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum A4MockKind {
    Patient,
    Medication,
    Verification,
}

#[component]
pub fn Hero(title: String, subtitle: String, children: Element) -> Element {
    rsx! {
        section { class: "ui-hero",
            h1 { class: "ui-display", "{title}" }
            p { class: "ui-hero-sub", "{subtitle}" }
            {children}
        }
    }
}

#[component]
pub fn SectionHeader(title: String, subtitle: Option<String>) -> Element {
    rsx! {
        div { class: "ui-section-head",
            h2 { class: "ui-section-title", "{title}" }
            if let Some(subtitle) = subtitle {
                p { class: "ui-section-sub", "{subtitle}" }
            }
        }
    }
}

#[component]
pub fn NumberedStepCard(
    number: u8,
    title: String,
    body: String,
    accent: AccentTone,
    mock: A4MockKind,
    tag: Option<String>,
) -> Element {
    rsx! {
        article { class: "ui-step",
            A4PreviewCard { accent, tag,
                A4Mock { kind: mock }
            }
            div { class: "ui-step-number", "{number}" }
            h3 { class: "ui-step-title", "{title}" }
            p { class: "ui-step-body", "{body}" }
        }
    }
}

#[component]
pub fn A4PreviewCard(
    #[props(default)] accent: AccentTone,
    tag: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "ui-a4-card ui-a4-card--{accent.suffix()}",
            div { class: "ui-a4-sheet", {children} }
            if let Some(label) = tag {
                VerificationTag { label }
            }
        }
    }
}

#[component]
pub fn VerificationTag(label: String) -> Element {
    rsx! {
        span { class: "ui-verification-tag", "{label}" }
    }
}

#[component]
pub fn A4Mock(kind: A4MockKind) -> Element {
    match kind {
        A4MockKind::Patient => rsx! {
            div { class: "ui-a4-mock",
                p { class: "ui-a4-kicker", "FICHA DEL PACIENTE" }
                p { class: "ui-a4-name", "María Fernanda Ruiz" }
                div { class: "ui-a4-row",
                    span { "Cédula" }
                    span { "0000000000" }
                }
                div { class: "ui-a4-row",
                    span { "Edad" }
                    span { "34 años" }
                }
                div { class: "ui-a4-row",
                    span { "Especialidad" }
                    span { "Medicina general" }
                }
            }
        },
        A4MockKind::Medication => rsx! {
            div { class: "ui-a4-mock",
                p { class: "ui-a4-kicker", "RECETA" }
                div { class: "ui-a4-row",
                    span { "Diagnóstico" }
                    span { "Faringitis aguda" }
                }
                p { class: "ui-a4-med", "1. Amoxicilina 500 mg" }
                p { class: "ui-a4-detail", "Cada 8 horas · 7 días" }
            }
        },
        A4MockKind::Verification => rsx! {
            div { class: "ui-a4-mock",
                p { class: "ui-a4-kicker", "VERIFICACIÓN" }
                div { class: "ui-a4-qr",
                    for cell in "1111111100000110111011010101101110110000011111111".chars() {
                        span {
                            class: if cell == '1' { "ui-a4-qr-cell ui-a4-qr-cell--on" } else { "ui-a4-qr-cell" },
                        }
                    }
                }
                p { class: "ui-a4-code", "PS-7K2F-9Q" }
                div { class: "ui-a4-ring",
                    Icon { name: IconName::Check, size: 20, extra_class: "ui-a4-check" }
                }
            }
        },
    }
}
