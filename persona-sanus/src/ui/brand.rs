use dioxus::prelude::*;

use super::{Icon, IconName};
use crate::Route;

#[derive(Clone, PartialEq)]
pub struct TrustLogo {
    pub name: String,
    pub src: Option<String>,
}

#[component]
pub fn LogoLockup() -> Element {
    rsx! {
        Link {
            class: "ui-logo",
            to: Route::Home {},
            aria_label: "PersonaSanus — inicio",
            span { class: "ui-logo-mark",
                Icon { name: IconName::Cross, size: 14, extra_class: "ui-logo-glyph" }
            }
            span { class: "ui-logo-word", "PersonaSanus" }
        }
    }
}

#[component]
pub fn TrustSecurityRow() -> Element {
    rsx! {
        div { class: "ui-trust",
            Icon { name: IconName::Shield, size: 18 }
            span { "Datos protegidos · Ley Orgánica de Protección de Datos Personales" }
            span { class: "ui-trust-platforms",
                Icon { name: IconName::Command, size: 18 }
                Icon { name: IconName::Windows, size: 18 }
                Icon { name: IconName::Globe, size: 18 }
                span { class: "ui-visually-hidden", "macOS, Windows, Web" }
            }
        }
    }
}

#[component]
pub fn TrustLogoStrip(
    #[props(default = "Con la confianza de…".to_string())] caption: String,
    logos: Vec<TrustLogo>,
) -> Element {
    rsx! {
        div { class: "ui-trust-strip",
            p { class: "ui-trust-caption", "{caption}" }
            div { class: "ui-trust-logos",
                for logo in logos {
                    if let Some(src) = logo.src {
                        img {
                            class: "ui-trust-logo-image",
                            src: "{src}",
                            alt: "{logo.name}",
                        }
                    } else {
                        span { class: "ui-trust-logo-text", "{logo.name}" }
                    }
                }
            }
        }
    }
}
