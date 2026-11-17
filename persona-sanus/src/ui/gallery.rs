use dioxus::prelude::*;

use super::*;
use crate::api;
use crate::Route;

const SAMPLE_PNG: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=";

fn color_swatch(token: &str, name: &str) -> Element {
    rsx! {
        div { class: "ui-gallery-frame",
            div { class: "ui-swatch", style: "background-color: var({token});" }
            p { class: "ui-gallery-label", "{name}" }
            code { class: "ui-mono", "{token}" }
        }
    }
}

fn type_sample(label: &str, token: &str) -> Element {
    rsx! {
        div { class: "ui-gallery-frame",
            p { class: "ui-gallery-label", "{label}" }
            p { style: "font-size: var({token}); line-height: 1.2; margin: 0;", "PersonaSanus" }
            code { class: "ui-mono", "{token}" }
        }
    }
}

fn icon_row(name: IconName, label: &str) -> Element {
    rsx! {
        div { class: "ui-gallery-icon-row",
            Icon { name, size: 20 }
            span { class: "ui-mono", "{label}" }
        }
    }
}

fn sample_asset(id: &str, name: &str, status: &str, url: Option<String>) -> api::AssetRow {
    api::AssetRow {
        id: id.to_owned(),
        kind: "image".to_owned(),
        original_name: Some(name.to_owned()),
        content_type: "image/png".to_owned(),
        size_bytes: 524288,
        sha256: "demo".to_owned(),
        width: Some(1200),
        height: Some(1600),
        status: status.to_owned(),
        url,
        created_at: "2026-01-15T10:00:00Z".to_owned(),
    }
}

fn sample_rx_status(active: bool, remaining: i32, exhausted: bool) -> api::RxBlockStatus {
    api::RxBlockStatus {
        active,
        block_id: active.then(|| "block-demo".to_owned()),
        start_number: active.then_some(1),
        end_number: active.then_some(100),
        consumed: active.then_some(100 - remaining),
        remaining,
        limit: 100,
        exhausted,
        device_id: active.then(|| "device-demo".to_owned()),
    }
}

fn sample_verification(status: &str) -> api::VerificationResult {
    api::VerificationResult {
        status: status.to_owned(),
        doctor_name: Some("Dra. Ejemplo Uno".to_owned()),
        issued_at: Some("2026-01-15".to_owned()),
        rx_number: Some(7),
    }
}

fn sample_transaction(status: &str) -> BillingTransaction {
    BillingTransaction {
        id: format!("transaction-{status}"),
        plan_name: "Plan mensual".to_owned(),
        amount_cents: 500,
        tax_cents: 48,
        status: status.to_owned(),
        requested_at: "2026-01-15".to_owned(),
    }
}

fn frame(label: &str, child: Element) -> Element {
    rsx! {
        div { class: "ui-gallery-frame",
            p { class: "ui-gallery-label", "{label}" }
            {child}
        }
    }
}

#[component]
pub fn DevUiGallery() -> Element {
    rsx! {
        PageShell { id: "dev-ui-page",
            div { class: "ui-gallery",
                header { class: "ui-gallery-head",
                    LogoLockup {}
                    h1 { "Sistema de diseño" }
                    p { "Componentes reutilizables de PersonaSanus. Solo desarrollo." }
                    ButtonLink {
                        to: Route::Home {},
                        variant: ButtonVariant::Secondary,
                        active_class: None,
                        "Volver al inicio"
                    }
                }
                section { id: "gallery-tokens", class: "ui-gallery-section",
                    h2 { "Tokens" }
                    div { class: "ui-gallery-grid",
                        {color_swatch("--color-deep-teal", "Deep Teal")}
                        {color_swatch("--color-ink", "Ink")}
                        {color_swatch("--color-midnight-teal", "Midnight Teal")}
                        {color_swatch("--color-mist", "Mist")}
                        {color_swatch("--color-hairline", "Hairline")}
                        {color_swatch("--color-paper", "Paper")}
                        {color_swatch("--color-cream", "Cream")}
                        {color_swatch("--color-mint-glass", "Mint Glass")}
                        {color_swatch("--color-bright-teal", "Bright Teal")}
                        {color_swatch("--color-plum", "Plum")}
                        {color_swatch("--color-blossom", "Blossom")}
                        {color_swatch("--color-ember-ring", "Ember Ring")}
                    }
                    div { class: "ui-gallery-grid",
                        {type_sample("Caption", "--text-caption")}
                        {type_sample("Body SM", "--text-body-sm")}
                        {type_sample("Body", "--text-body")}
                        {type_sample("Body LG", "--text-body-lg")}
                        {type_sample("Subheading", "--text-subheading")}
                        {type_sample("Heading SM", "--text-heading-sm")}
                        {type_sample("Heading", "--text-heading")}
                        {type_sample("Heading LG", "--text-heading-lg")}
                        {type_sample("Display", "--text-display")}
                    }
                    div { class: "ui-gallery-grid",
                        div { class: "ui-gallery-frame",
                            p { class: "ui-gallery-label", "Espaciado" }
                            p { class: "ui-mono", "--spacing-4 · --spacing-8 · --spacing-12 · --spacing-16 · --spacing-24 · --spacing-32 · --spacing-48 · --spacing-80" }
                            p { class: "ui-mono", "--section-gap: 80px · --section-gap-compact: 64px · --element-gap: 8px · --element-gap-lg: 16px" }
                        }
                        div { class: "ui-gallery-frame",
                            p { class: "ui-gallery-label", "Radios" }
                            div { class: "ui-gallery-radii",
                                span { class: "ui-radius-sample", style: "border-radius: var(--radius-buttons);", "8" }
                                span { class: "ui-radius-sample", style: "border-radius: var(--radius-xl);", "12" }
                                span { class: "ui-radius-sample", style: "border-radius: var(--radius-cards);", "24" }
                                span { class: "ui-radius-sample", style: "border-radius: var(--radius-pill);", "∞" }
                            }
                        }
                        div { class: "ui-gallery-frame",
                            p { class: "ui-gallery-label", "Superficies" }
                            div { class: "ui-gallery-surfaces",
                                span { class: "ui-surface-sample", style: "background-color: var(--surface-canvas);", "Canvas" }
                                span { class: "ui-surface-sample", style: "background-color: var(--surface-cream-section);", "Cream" }
                                span { class: "ui-surface-sample", style: "background-color: var(--surface-mint-section);", "Mint" }
                                span { class: "ui-surface-sample", style: "background-color: var(--surface-pink-card);", "Blossom" }
                                span { class: "ui-surface-sample", style: "background-color: var(--surface-card);", "Card" }
                            }
                        }
                        div { class: "ui-gallery-frame",
                            p { class: "ui-gallery-label", "Elevación" }
                            p { "La única sombra es el halo de foco o --shadow-paper. Las tarjetas y los botones no llevan sombra." }
                        }
                        div { class: "ui-gallery-frame",
                            p { class: "ui-gallery-label", "Foco compartido" }
                            Button { variant: ButtonVariant::Secondary, "Ejemplo de foco" }
                        }
                    }
                    div { class: "ui-gallery-frame",
                        p { class: "ui-gallery-label", "Iconos" }
                        div { class: "ui-gallery-icons",
                            {icon_row(IconName::Cross, "Cross")}
                            {icon_row(IconName::Shield, "Shield")}
                            {icon_row(IconName::Command, "Command")}
                            {icon_row(IconName::Windows, "Windows")}
                            {icon_row(IconName::Globe, "Globe")}
                            {icon_row(IconName::Check, "Check")}
                            {icon_row(IconName::Close, "Close")}
                            {icon_row(IconName::Alert, "Alert")}
                            {icon_row(IconName::Info, "Info")}
                            {icon_row(IconName::Qr, "Qr")}
                            {icon_row(IconName::ArrowRight, "ArrowRight")}
                            {icon_row(IconName::Lock, "Lock")}
                            {icon_row(IconName::Image, "Image")}
                        }
                    }
                }
                section { id: "gallery-layout", class: "ui-gallery-section",
                    h2 { "Layout" }
                    div { class: "ui-gallery-grid",
                        {frame("PageHeader", rsx! {
                            PageHeader {
                                title: "Título de página",
                                subtitle: Some("Subtítulo de ejemplo.".to_owned()),
                                Button { variant: ButtonVariant::Secondary, "Acción" }
                            }
                        })}
                        {frame("Card", rsx! {
                            Card { p { "Contenido de tarjeta." } }
                        })}
                        {frame("SectionCard con descripción", rsx! {
                            SectionCard {
                                title: "Sección",
                                description: Some("Descripción de la sección.".to_owned()),
                                p { "Contenido de la sección." }
                            }
                        })}
                        {frame("SectionCard sin descripción", rsx! {
                            SectionCard { title: "Sección", description: None,
                                p { "Contenido de la sección." }
                            }
                        })}
                        {frame("FormGrid Two", rsx! {
                            FormGrid { columns: FormColumns::Two,
                                TextField { label: "Uno", value: String::new(), oninput: move |_value: String| {}, placeholder: String::new(), hint: None, error: None, autocomplete: "off".to_owned(), spellcheck: true, required: false, disabled: false, readonly: false, min: None, max: None, step: None, wide: false, hide_label: false }
                                TextField { label: "Dos", value: String::new(), oninput: move |_value: String| {}, placeholder: String::new(), hint: None, error: None, autocomplete: "off".to_owned(), spellcheck: true, required: false, disabled: false, readonly: false, min: None, max: None, step: None, wide: false, hide_label: false }
                            }
                        })}
                        {frame("FormGrid One", rsx! {
                            FormGrid { columns: FormColumns::One,
                                TextField { label: "Uno", value: String::new(), oninput: move |_value: String| {}, placeholder: String::new(), hint: None, error: None, autocomplete: "off".to_owned(), spellcheck: true, required: false, disabled: false, readonly: false, min: None, max: None, step: None, wide: false, hide_label: false }
                            }
                        })}
                        {frame("DetailGrid", rsx! {
                            DetailGrid {
                                DetailItem { label: "Normal", "Valor de ejemplo" }
                                DetailItem { label: "Ancho", wide: true, "Valor ancho de ejemplo" }
                            }
                        })}
                        {frame("ActionRow", rsx! {
                            ActionRow {
                                Button { variant: ButtonVariant::Primary, "Primario" }
                                Button { variant: ButtonVariant::Secondary, "Secundario" }
                            }
                        })}
                        {frame("FilterBar", rsx! {
                            FilterBar {
                                TextField { label: "Filtro", value: String::new(), oninput: move |_value: String| {}, placeholder: "Buscar".to_owned(), hint: None, error: None, autocomplete: "off".to_owned(), spellcheck: true, required: false, disabled: false, readonly: false, min: None, max: None, step: None, wide: false, hide_label: false }
                            }
                        })}
                    }
                    div { class: "ui-gallery-grid",
                        {frame("Band Paper", rsx! { SectionBand { id: None, tone: SurfaceTone::Paper, p { "Paper" } } })}
                        {frame("Band Cream", rsx! { SectionBand { id: None, tone: SurfaceTone::Cream, p { "Cream" } } })}
                        {frame("Band Mint", rsx! { SectionBand { id: None, tone: SurfaceTone::Mint, p { "Mint" } } })}
                        {frame("Band Blossom", rsx! { SectionBand { id: None, tone: SurfaceTone::Blossom, p { "Blossom" } } })}
                    }
                }
                section { id: "gallery-buttons", class: "ui-gallery-section",
                    h2 { "Botones" }
                    div { class: "ui-gallery-grid",
                        {frame("Primary", rsx! { Button { variant: ButtonVariant::Primary, "Emitir receta" } })}
                        {frame("Secondary", rsx! { Button { variant: ButtonVariant::Secondary, "Vista previa A4" } })}
                        {frame("Ghost", rsx! { Button { variant: ButtonVariant::Ghost, "Iniciar sesión" } })}
                        {frame("Danger", rsx! { Button { variant: ButtonVariant::Danger, "Anular" } })}
                        {frame("Primary deshabilitado", rsx! { Button { variant: ButtonVariant::Primary, disabled: true, "No disponible" } })}
                        {frame("Secondary cargando", rsx! { Button { variant: ButtonVariant::Secondary, loading: true, "Guardando…" } })}
                        {frame("ButtonLink Primary", rsx! {
                            ButtonLink { to: Route::Home {}, variant: ButtonVariant::Primary, active_class: None, "Ir al inicio" }
                        })}
                        {frame("ButtonLink Secondary", rsx! {
                            ButtonLink { to: Route::Home {}, variant: ButtonVariant::Secondary, active_class: None, "Ir al inicio" }
                        })}
                        {frame("LinkAction interno", rsx! {
                            LinkAction { href: "#gallery-brand".to_owned(), variant: ButtonVariant::Secondary, target: None, rel: None, "Ver marca" }
                        })}
                        {frame("LinkAction externo", rsx! {
                            LinkAction { href: "https://example.com".to_owned(), variant: ButtonVariant::Secondary, target: Some("_blank".to_owned()), rel: Some("noopener noreferrer".to_owned()), "Sitio externo" }
                        })}
                        {frame("NavLink", rsx! {
                            NavLink { to: Route::Home {}, label: "Inicio".to_owned() }
                            NavLink { to: Route::SettingsProfile {}, label: "Mi perfil".to_owned() }
                        })}
                    }
                }
                section { id: "gallery-forms", class: "ui-gallery-section",
                    h2 { "Formularios" }
                    div { class: "ui-gallery-grid",
                        TextField { label: "Vacío", value: String::new(), oninput: move |_value: String| {}, placeholder: String::new(), hint: None, error: None, autocomplete: "off".to_owned(), spellcheck: true, required: false, disabled: false, readonly: false, min: None, max: None, step: None, wide: false, hide_label: false }
                        TextField { label: "Con valor", value: "Ana Ejemplo".to_owned(), oninput: move |_value: String| {}, placeholder: String::new(), hint: None, error: None, autocomplete: "off".to_owned(), spellcheck: true, required: false, disabled: false, readonly: false, min: None, max: None, step: None, wide: false, hide_label: false }
                        TextField { label: "Correo", value: "ana@example.com".to_owned(), oninput: move |_value: String| {}, input_type: "email".to_owned(), placeholder: "correo@ejemplo.com".to_owned(), hint: None, error: None, autocomplete: "email".to_owned(), spellcheck: false, required: false, disabled: false, readonly: false, min: None, max: None, step: None, wide: false, hide_label: false }
                        TextField { label: "Edad", value: "34".to_owned(), oninput: move |_value: String| {}, input_type: "number".to_owned(), placeholder: String::new(), hint: None, error: None, autocomplete: "off".to_owned(), spellcheck: false, required: false, disabled: false, readonly: false, min: Some("1".to_owned()), max: Some("120".to_owned()), step: Some("1".to_owned()), wide: false, hide_label: false }
                        TextField { label: "Nacimiento", value: "1990-01-01".to_owned(), oninput: move |_value: String| {}, input_type: "date".to_owned(), placeholder: String::new(), hint: None, error: None, autocomplete: "off".to_owned(), spellcheck: false, required: false, disabled: false, readonly: false, min: None, max: None, step: None, wide: false, hide_label: false }
                        TextField { label: "Obligatorio", value: String::new(), oninput: move |_value: String| {}, placeholder: String::new(), hint: None, error: None, autocomplete: "off".to_owned(), spellcheck: true, required: true, disabled: false, readonly: false, min: None, max: None, step: None, wide: false, hide_label: false }
                        TextField { label: "Ancho", value: String::new(), oninput: move |_value: String| {}, placeholder: String::new(), hint: None, error: None, autocomplete: "off".to_owned(), spellcheck: true, required: false, disabled: false, readonly: false, min: None, max: None, step: None, wide: true, hide_label: false }
                        TextField { label: "Con ayuda", value: String::new(), oninput: move |_value: String| {}, placeholder: String::new(), hint: Some("Máximo 10 MB.".to_owned()), error: None, autocomplete: "off".to_owned(), spellcheck: true, required: false, disabled: false, readonly: false, min: None, max: None, step: None, wide: false, hide_label: false }
                        TextField { label: "Con error", value: String::new(), oninput: move |_value: String| {}, placeholder: String::new(), hint: None, error: Some("No se pudo guardar el cambio.".to_owned()), autocomplete: "off".to_owned(), spellcheck: true, required: false, disabled: false, readonly: false, min: None, max: None, step: None, wide: false, hide_label: false }
                        TextField { label: "Deshabilitado", value: "Fijo".to_owned(), oninput: move |_value: String| {}, placeholder: String::new(), hint: None, error: None, autocomplete: "off".to_owned(), spellcheck: true, required: false, disabled: true, readonly: false, min: None, max: None, step: None, wide: false, hide_label: false }
                        TextField { label: "Sólo lectura", value: "Fijo".to_owned(), oninput: move |_value: String| {}, placeholder: String::new(), hint: None, error: None, autocomplete: "off".to_owned(), spellcheck: true, required: false, disabled: false, readonly: true, min: None, max: None, step: None, wide: false, hide_label: false }
                        TextField { label: "Etiqueta oculta", value: "Valor".to_owned(), oninput: move |_value: String| {}, placeholder: "Sin etiqueta visible".to_owned(), hint: None, error: None, autocomplete: "off".to_owned(), spellcheck: true, required: false, disabled: false, readonly: false, min: None, max: None, step: None, wide: false, hide_label: true }
                        TextAreaField { label: "Notas", value: "Notas de ejemplo.".to_owned(), oninput: move |_value: String| {}, placeholder: "Escribe una nota".to_owned(), wide: false, hint: None, error: None, required: false, disabled: false, readonly: false, hide_label: false, rows: Some(4) }
                        TextAreaField { label: "Notas anchas", value: String::new(), oninput: move |_value: String| {}, placeholder: String::new(), wide: true, hint: None, error: None, required: false, disabled: false, readonly: false, hide_label: false, rows: None }
                        SelectField { label: "Tipo", value: "a".to_owned(), onchange: move |_value: String| {}, wide: false, hint: None, error: None, disabled: false, hide_label: false,
                            option { value: "a", "Opción A" }
                            option { value: "b", "Opción B" }
                            option { value: "c", "Opción C" }
                        }
                        SelectField { label: "Con ayuda", value: "a".to_owned(), onchange: move |_value: String| {}, wide: false, hint: Some("Selecciona una opción.".to_owned()), error: None, disabled: false, hide_label: false,
                            option { value: "a", "Opción A" }
                        }
                        SelectField { label: "Con error", value: "a".to_owned(), onchange: move |_value: String| {}, wide: false, hint: None, error: Some("Selecciona una opción válida.".to_owned()), disabled: false, hide_label: false,
                            option { value: "a", "Opción A" }
                        }
                        CheckboxField { label: "Aceptar", description: Some("Descripción de ejemplo.".to_owned()), checked: true, onchange: move |_value: bool| {}, disabled: false, wide: false }
                        FileField { label: "Archivo", accept: "image/jpeg,image/png".to_owned(), disabled: false, hint: Some("JPEG o PNG, máximo 10 MB.".to_owned()), on_change: move |_event: FormEvent| {}, wide: true }
                        SearchField { label: "Buscar", value: String::new(), oninput: move |_value: String| {}, placeholder: "Buscar por nombre".to_owned(), wide: false }
                    }
                }
                section { id: "gallery-badges", class: "ui-gallery-section",
                    h2 { "Insignias" }
                    div { class: "ui-gallery-grid",
                        {frame("Neutral", rsx! { Badge { tone: BadgeTone::Neutral, "Neutral" } })}
                        {frame("Mint", rsx! { Badge { tone: BadgeTone::Mint, "Mint" } })}
                        {frame("Cream", rsx! { Badge { tone: BadgeTone::Cream, "Cream" } })}
                        {frame("Blossom", rsx! { Badge { tone: BadgeTone::Blossom, "Blossom" } })}
                        {frame("Plum", rsx! { Badge { tone: BadgeTone::Plum, "Plum" } })}
                        {frame("Pill", rsx! { PillBadge {} })}
                    }
                }
                section { id: "gallery-feedback", class: "ui-gallery-section",
                    h2 { "Retroalimentación" }
                    div { class: "ui-gallery-grid",
                        {frame("Info", rsx! { Notice { tone: NoticeTone::Info, p { strong { "Información" } " Texto de ejemplo informativo." } } })}
                        {frame("Success", rsx! { Notice { tone: NoticeTone::Success, p { strong { "Listo" } " La operación se completó." } } })}
                        {frame("Warning", rsx! { Notice { tone: NoticeTone::Warning, p { strong { "Atención" } " Revisa este dato antes de continuar." } } })}
                        {frame("Error", rsx! { Notice { tone: NoticeTone::Error, p { strong { "Error" } " No se pudo completar la operación." } } })}
                        {frame("ErrorText", rsx! { ErrorText { text: "No se pudo guardar el cambio.".to_owned() } })}
                        {frame("HintText", rsx! { HintText { text: "Máximo 10 MB.".to_owned() } })}
                        {frame("EmptyState", rsx! {
                            EmptyState { title: "Sin registros".to_owned(), description: "Todavía no hay registros para mostrar.".to_owned() }
                        })}
                        {frame("EmptyState sin descripción", rsx! { EmptyState { title: "Sin registros".to_owned() } })}
                        {frame("LoadingState", rsx! { LoadingState {} })}
                        {frame("LoadingState personalizado", rsx! { LoadingState { label: "Verificando receta…".to_owned() } })}
                        {frame("ConfirmPanel", rsx! {
                            ConfirmPanel {
                                message: "Esta acción no se puede deshacer.".to_owned(),
                                confirm_label: "Confirmar".to_owned(),
                                cancel_label: "Cancelar".to_owned(),
                                busy: false,
                                on_confirm: move |_event: MouseEvent| {},
                                on_cancel: move |_event: MouseEvent| {},
                                p { "Contenido de ejemplo." }
                            }
                        })}
                    }
                }
                section { id: "gallery-data", class: "ui-gallery-section",
                    h2 { "Datos" }
                    div { class: "ui-gallery-grid",
                        {frame("DataTable", rsx! {
                            DataTable {
                                columns: vec!["Nombre".to_owned(), "Correo".to_owned(), "Estado".to_owned()],
                                caption: Some("Ejemplo de tabla".to_owned()),
                                tbody {
                                    tr { td { "Ana Ejemplo" } td { "ana@example.com" } td { "Activa" } }
                                    tr { td { "Luis Ejemplo" } td { "luis@example.com" } td { "Pendiente" } }
                                    tr { td { "Marta Ejemplo" } td { "marta@example.com" } td { "Inactiva" } }
                                }
                            }
                        })}
                        {frame("ListRow", rsx! {
                            ListRow {
                                title: "María Fernanda Ruiz".to_owned(),
                                meta: vec!["Cédula 0000000000".to_owned(), "Nacimiento 01/01/1990".to_owned()],
                                badge: Some(rsx! { Badge { tone: BadgeTone::Neutral, "Borrador" } }),
                                Button { variant: ButtonVariant::Secondary, "Abrir" }
                            }
                        })}
                    }
                }
                section { id: "gallery-brand", class: "ui-gallery-section",
                    h2 { "Marca" }
                    div { class: "ui-gallery-grid",
                        {frame("LogoLockup", rsx! { LogoLockup {} })}
                        {frame("TrustSecurityRow", rsx! { TrustSecurityRow {} })}
                    }
                    {frame("TrustLogoStrip", rsx! {
                        div {
                            TrustLogoStrip {
                                caption: "Con la confianza de…".to_owned(),
                                logos: vec![
                                    TrustLogo { name: "Clínica Demo Norte".to_owned(), src: None },
                                    TrustLogo { name: "Consultorio Demo Sur".to_owned(), src: None },
                                    TrustLogo { name: "Centro Demo Andes".to_owned(), src: None },
                                    TrustLogo { name: "Clínica Demo Costa".to_owned(), src: None },
                                    TrustLogo { name: "Salud Demo".to_owned(), src: None },
                                ],
                            }
                            p { "Logos de ejemplo; no son clientes reales." }
                        }
                    })}
                }
                section { id: "gallery-marketing", class: "ui-gallery-section",
                    h2 { "Marketing" }
                    Hero {
                        title: "Hero de ejemplo".to_owned(),
                        subtitle: "Título y subtítulo de demostración.".to_owned(),
                        ActionRow {
                            Button { variant: ButtonVariant::Primary, "Acción principal" }
                            Button { variant: ButtonVariant::Secondary, "Acción secundaria" }
                        }
                    }
                    SectionHeader {
                        title: "Sección con subtítulo".to_owned(),
                        subtitle: Some("Subtítulo de ejemplo.".to_owned()),
                    }
                    SectionHeader { title: "Sección sin subtítulo".to_owned(), subtitle: None }
                    div { class: "ui-steps",
                        NumberedStepCard {
                            number: 1,
                            title: "Elige el paciente".to_owned(),
                            body: "Paso de ejemplo.".to_owned(),
                            accent: AccentTone::Mint,
                            mock: A4MockKind::Patient,
                            tag: None
                        }
                        NumberedStepCard {
                            number: 2,
                            title: "Completa la receta".to_owned(),
                            body: "Paso de ejemplo.".to_owned(),
                            accent: AccentTone::Cream,
                            mock: A4MockKind::Medication,
                            tag: None
                        }
                        NumberedStepCard {
                            number: 3,
                            title: "Firma y verifica".to_owned(),
                            body: "Paso de ejemplo.".to_owned(),
                            accent: AccentTone::Blossom,
                            mock: A4MockKind::Verification,
                            tag: Some("Verificable".to_owned())
                        }
                    }
                    div { class: "ui-gallery-grid",
                        {frame("A4 Mint", rsx! {
                            A4PreviewCard { accent: AccentTone::Mint, tag: None,
                                A4Mock { kind: A4MockKind::Patient }
                            }
                        })}
                        {frame("A4 Cream", rsx! {
                            A4PreviewCard { accent: AccentTone::Cream, tag: None,
                                A4Mock { kind: A4MockKind::Medication }
                            }
                        })}
                        {frame("A4 Blossom", rsx! {
                            A4PreviewCard { accent: AccentTone::Blossom, tag: None,
                                A4Mock { kind: A4MockKind::Verification }
                            }
                        })}
                        {frame("A4 Conic", rsx! {
                            A4PreviewCard { accent: AccentTone::Conic, tag: Some("Verificable".to_owned()),
                                A4Mock { kind: A4MockKind::Verification }
                            }
                        })}
                    }
                    {frame("VerificationTag", rsx! { VerificationTag { label: "Verificable".to_owned() } })}
                }
                section { id: "gallery-status", class: "ui-gallery-section",
                    h2 { "Estado" }
                    div { class: "ui-gallery-grid",
                        {frame("Sync online", rsx! { SyncStatusIndicator { status: SyncStatus { online: true, pending: 0, failed: 0 } } })}
                        {frame("Sync offline", rsx! { SyncStatusIndicator { status: SyncStatus { online: false, pending: 0, failed: 0 } } })}
                        {frame("Sync pending 1", rsx! { SyncStatusIndicator { status: SyncStatus { online: true, pending: 1, failed: 0 } } })}
                        {frame("Sync pending 3", rsx! { SyncStatusIndicator { status: SyncStatus { online: true, pending: 3, failed: 0 } } })}
                        {frame("Sync failed", rsx! { SyncStatusIndicator { status: SyncStatus { online: true, pending: 3, failed: 2 } } })}
                        {frame("Prescription draft", rsx! { PrescriptionStatusBadge { status: "draft".to_owned() } })}
                        {frame("Prescription signed", rsx! { PrescriptionStatusBadge { status: "signed".to_owned() } })}
                        {frame("Prescription superseded", rsx! { PrescriptionStatusBadge { status: "superseded".to_owned() } })}
                        {frame("Prescription voided", rsx! { PrescriptionStatusBadge { status: "voided".to_owned() } })}
                        {frame("Prescription pending_sync", rsx! { PrescriptionStatusBadge { status: "pending_sync".to_owned() } })}
                        {frame("Prescription failed", rsx! { PrescriptionStatusBadge { status: "failed".to_owned() } })}
                        {frame("Prescription unknown", rsx! { PrescriptionStatusBadge { status: "other".to_owned() } })}
                        {frame("Template draft", rsx! { TemplateStatusBadge { status: "draft".to_owned() } })}
                        {frame("Template published", rsx! { TemplateStatusBadge { status: "published".to_owned() } })}
                        {frame("Template archived", rsx! { TemplateStatusBadge { status: "archived".to_owned() } })}
                        {frame("Template unknown", rsx! { TemplateStatusBadge { status: "other".to_owned() } })}
                        {frame("Account active", rsx! { AccountStatusBadge { status: "active".to_owned() } })}
                        {frame("Account disabled", rsx! { AccountStatusBadge { status: "disabled".to_owned() } })}
                        {frame("Account unknown", rsx! { AccountStatusBadge { status: "other".to_owned() } })}
                        {frame("Device active", rsx! { DeviceStatusBadge { revoked: false } })}
                        {frame("Device revoked", rsx! { DeviceStatusBadge { revoked: true } })}
                        {frame("Rx none", rsx! {
                            RxBlockIndicator { status: None, allocating: false, on_allocate: move |_event: MouseEvent| {} }
                        })}
                        {frame("Rx inactive", rsx! {
                            RxBlockIndicator { status: Some(sample_rx_status(false, 0, false)), allocating: false, on_allocate: move |_event: MouseEvent| {} }
                        })}
                        {frame("Rx active", rsx! {
                            RxBlockIndicator { status: Some(sample_rx_status(true, 42, false)), allocating: false, on_allocate: move |_event: MouseEvent| {} }
                        })}
                        {frame("Rx exhausted", rsx! {
                            RxBlockIndicator { status: Some(sample_rx_status(true, 0, true)), allocating: false, on_allocate: move |_event: MouseEvent| {} }
                        })}
                        {frame("Rx allocating", rsx! {
                            RxBlockIndicator { status: Some(sample_rx_status(true, 0, true)), allocating: true, on_allocate: move |_event: MouseEvent| {} }
                        })}
                    }
                }
                section { id: "gallery-verification", class: "ui-gallery-section",
                    h2 { "Verificación" }
                    div { class: "ui-gallery-grid",
                        {frame("Seal valid", rsx! { VerificationSeal { status: "valid".to_owned() } })}
                        {frame("Seal voided", rsx! { VerificationSeal { status: "voided".to_owned() } })}
                        {frame("Seal superseded", rsx! { VerificationSeal { status: "superseded".to_owned() } })}
                        {frame("Seal not found", rsx! { VerificationSeal { status: "not found".to_owned() } })}
                        {frame("Seal unknown", rsx! { VerificationSeal { status: "unknown".to_owned() } })}
                    }
                    div { class: "ui-gallery-grid",
                        {frame("Card valid", rsx! { VerificationResultCard { verification: sample_verification("valid") } })}
                        {frame("Card voided", rsx! { VerificationResultCard { verification: sample_verification("voided") } })}
                        {frame("Card superseded", rsx! { VerificationResultCard { verification: sample_verification("superseded") } })}
                        {frame("Card not found", rsx! { VerificationResultCard { verification: sample_verification("not found") } })}
                        {frame("Card unknown", rsx! { VerificationResultCard { verification: sample_verification("unknown") } })}
                    }
                    div { class: "ui-gallery-grid",
                        {frame("Code form Primary", rsx! { VerificationCodeForm { button_variant: ButtonVariant::Primary, on_submit: move |_value: String| {} } })}
                        {frame("Code form Secondary", rsx! { VerificationCodeForm { button_variant: ButtonVariant::Secondary, on_submit: move |_value: String| {} } })}
                    }
                }
                section { id: "gallery-library", class: "ui-gallery-section",
                    h2 { "Biblioteca" }
                    div { class: "ui-gallery-grid",
                        {frame("Quota 1 de 5", rsx! { QuotaMeter { used: 1, limit: 5 } })}
                        {frame("Quota 5 de 5", rsx! { QuotaMeter { used: 5, limit: 5 } })}
                        {frame("Quota 0 de 0", rsx! { QuotaMeter { used: 0, limit: 0 } })}
                        {frame("Library empty", rsx! {
                            ImageLibrary { assets: Vec::new(), used: 0, limit: 5, uploading: false, deleting: None, online: true, on_delete: move |_id: String| {}, on_upload: move |_event: FormEvent| {} }
                        })}
                        {frame("Library pending", rsx! {
                            ImageLibrary {
                                assets: vec![sample_asset("asset-2", "Pendiente de ejemplo", "pending", None)],
                                used: 1, limit: 5, uploading: false, deleting: None, online: true,
                                on_delete: move |_id: String| {},
                                on_upload: move |_event: FormEvent| {}
                            }
                        })}
                        {frame("Library ready URL", rsx! {
                            ImageLibrary {
                                assets: vec![sample_asset("asset-1", "Imagen de ejemplo", "ready", Some(SAMPLE_PNG.to_owned()))],
                                used: 1, limit: 5, uploading: false, deleting: None, online: true,
                                on_delete: move |_id: String| {},
                                on_upload: move |_event: FormEvent| {}
                            }
                        })}
                        {frame("Library ready no URL", rsx! {
                            ImageLibrary {
                                assets: vec![sample_asset("asset-2", "Imagen de ejemplo", "ready", None)],
                                used: 1, limit: 5, uploading: false, deleting: None, online: true,
                                on_delete: move |_id: String| {},
                                on_upload: move |_event: FormEvent| {}
                            }
                        })}
                        {frame("Library uploading", rsx! {
                            ImageLibrary {
                                assets: vec![sample_asset("asset-1", "Imagen de ejemplo", "ready", Some(SAMPLE_PNG.to_owned()))],
                                used: 1, limit: 5, uploading: true, deleting: None, online: true,
                                on_delete: move |_id: String| {},
                                on_upload: move |_event: FormEvent| {}
                            }
                        })}
                        {frame("Library deleting", rsx! {
                            ImageLibrary {
                                assets: vec![sample_asset("asset-1", "Imagen de ejemplo", "ready", Some(SAMPLE_PNG.to_owned()))],
                                used: 1, limit: 5, uploading: false, deleting: Some("asset-1".to_owned()), online: true,
                                on_delete: move |_id: String| {},
                                on_upload: move |_event: FormEvent| {}
                            }
                        })}
                        {frame("Library offline", rsx! {
                            ImageLibrary {
                                assets: vec![sample_asset("asset-1", "Imagen de ejemplo", "ready", Some(SAMPLE_PNG.to_owned()))],
                                used: 1, limit: 5, uploading: false, deleting: None, online: false,
                                on_delete: move |_id: String| {},
                                on_upload: move |_event: FormEvent| {}
                            }
                        })}
                        {frame("Library full", rsx! {
                            ImageLibrary {
                                assets: vec![sample_asset("asset-1", "Imagen de ejemplo", "ready", Some(SAMPLE_PNG.to_owned()))],
                                used: 5, limit: 5, uploading: false, deleting: None, online: true,
                                on_delete: move |_id: String| {},
                                on_upload: move |_event: FormEvent| {}
                            }
                        })}
                    }
                }
                section { id: "gallery-billing", class: "ui-gallery-section",
                    h2 { "Facturación" }
                    div { class: "ui-gallery-grid",
                        {frame("$5,00 USD", rsx! { code { class: "ui-mono", "{format_usd_cents(500)}" } })}
                        {frame("$50,00 USD", rsx! { code { class: "ui-mono", "{format_usd_cents(5000)}" } })}
                        {frame("$43,48 USD", rsx! { code { class: "ui-mono", "{format_usd_cents(4348)}" } })}
                        {frame("$1.234,56 USD", rsx! { code { class: "ui-mono", "{format_usd_cents(123456)}" } })}
                        {frame("-$5,00 USD", rsx! { code { class: "ui-mono", "{format_usd_cents(-500)}" } })}
                    }
                    div { class: "ui-billing-plans",
                        BillingStatusCard {
                            status: "trialing".to_owned(),
                            trial_ends_at: Some("2026-02-15".to_owned()),
                            next_charge_at: None,
                            grace_ends_at: None,
                            cancel_at_period_end: false,
                            p { "Ejemplo de período de prueba." }
                        }
                        BillingStatusCard {
                            status: "active".to_owned(),
                            trial_ends_at: None,
                            next_charge_at: Some("2026-02-15".to_owned()),
                            grace_ends_at: None,
                            cancel_at_period_end: true,
                            p { "Ejemplo de suscripción activa." }
                        }
                        BillingStatusCard {
                            status: "past_due".to_owned(),
                            trial_ends_at: None,
                            next_charge_at: None,
                            grace_ends_at: Some("2026-02-15".to_owned()),
                            cancel_at_period_end: false,
                            p { "Ejemplo de pago pendiente." }
                        }
                        BillingStatusCard {
                            status: "locked".to_owned(),
                            trial_ends_at: None,
                            next_charge_at: None,
                            grace_ends_at: None,
                            cancel_at_period_end: false,
                            p { "Ejemplo de sincronización bloqueada." }
                        }
                        BillingStatusCard {
                            status: "canceled".to_owned(),
                            trial_ends_at: None,
                            next_charge_at: None,
                            grace_ends_at: None,
                            cancel_at_period_end: false,
                            p { "Ejemplo de suscripción cancelada." }
                        }
                        BillingStatusCard {
                            status: "unknown".to_owned(),
                            trial_ends_at: None,
                            next_charge_at: None,
                            grace_ends_at: None,
                            cancel_at_period_end: false,
                            p { "Ejemplo de estado desconocido." }
                        }
                        PlanCard {
                            code: "monthly".to_owned(),
                            name: "Mensual".to_owned(),
                            amount_cents: 500,
                            interval_months: 1,
                            features: vec!["Atención clínica".to_owned(), "Biblioteca de imágenes".to_owned()],
                            selected: false,
                            on_select: move |_code: String| {}
                        }
                        PlanCard {
                            code: "yearly".to_owned(),
                            name: "Anual".to_owned(),
                            amount_cents: 5000,
                            interval_months: 12,
                            features: vec!["Atención clínica".to_owned(), "Biblioteca de imágenes".to_owned()],
                            selected: true,
                            on_select: move |_code: String| {}
                        }
                        PaymentMethodCard {
                            status: "active".to_owned(),
                            consented_at: Some("2026-01-15".to_owned()),
                            on_revoke: move |_event: MouseEvent| {}
                        }
                        PaymentMethodCard {
                            status: "revoked".to_owned(),
                            consented_at: None,
                            on_revoke: move |_event: MouseEvent| {}
                        }
                    }
                    div { class: "ui-gallery-grid",
                        {frame("Banner past_due con gracia", rsx! { BillingBanner { status: "past_due".to_owned(), grace_ends_at: Some("2026-02-15".to_owned()) } })}
                        {frame("Banner past_due sin gracia", rsx! { BillingBanner { status: "past_due".to_owned(), grace_ends_at: None } })}
                        {frame("Banner locked", rsx! { BillingBanner { status: "locked".to_owned(), grace_ends_at: None } })}
                        {frame("Banner canceled", rsx! { BillingBanner { status: "canceled".to_owned(), grace_ends_at: None } })}
                        {frame("Banner active", rsx! { BillingBanner { status: "active".to_owned(), grace_ends_at: None } })}
                    }
                    div { class: "ui-gallery-grid",
                        {frame("Transacciones", rsx! {
                            TransactionTable {
                                transactions: vec![
                                    sample_transaction("pending"),
                                    sample_transaction("approved"),
                                    sample_transaction("canceled"),
                                    sample_transaction("failed"),
                                    sample_transaction("reversed"),
                                ]
                            }
                        })}
                        {frame("Transacciones vacías", rsx! {
                            TransactionTable { transactions: Vec::new() }
                        })}
                    }
                }
            }
        }
    }
}
