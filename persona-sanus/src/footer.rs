use dioxus::prelude::*;

use crate::ui::{LogoLockup, TrustSecurityRow};

#[derive(Clone, PartialEq)]
pub(crate) enum FooterSurface {
    Home,
    Auth,
    Verify,
}

#[component]
pub(crate) fn Footer(surface: FooterSurface) -> Element {
    #[cfg(feature = "desktop")]
    let version = Some(format!("Versión {}", env!("CARGO_PKG_VERSION")));
    #[cfg(not(feature = "desktop"))]
    let version: Option<String> = None;
    rsx! {
        footer { id: "ui-footer", class: "ui-footer",
            div { class: "ui-container",
                div { class: "ui-footer-inner",
                    if matches!(surface, FooterSurface::Home | FooterSurface::Auth) {
                        div { class: "ui-footer-brand",
                            LogoLockup {}
                            if matches!(surface, FooterSurface::Home) {
                                a { href: "#verificar", "Verificar una receta" }
                            }
                        }
                        hr { class: "ui-footer-rule", "aria-hidden": "true" }
                    }
                    div { class: "ui-footer-legal",
                        if matches!(surface, FooterSurface::Auth) {
                            TrustSecurityRow {}
                        }
                        if matches!(surface, FooterSurface::Verify) {
                            p { class: "ui-footer-disclosure", "Solo se muestran el médico, la fecha y el número de la receta." }
                        }
                        if let Some(version) = version {
                            p { class: "ui-footer-version", "{version}" }
                        }
                    }
                }
            }
        }
    }
}
