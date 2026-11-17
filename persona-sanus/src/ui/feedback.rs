use dioxus::prelude::*;

use super::{Button, ButtonVariant, Icon, IconName};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeTone {
    #[default]
    Neutral,
    Mint,
    Cream,
    Blossom,
    Plum,
}

impl BadgeTone {
    fn suffix(self) -> &'static str {
        match self {
            BadgeTone::Neutral => "neutral",
            BadgeTone::Mint => "mint",
            BadgeTone::Cream => "cream",
            BadgeTone::Blossom => "blossom",
            BadgeTone::Plum => "plum",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NoticeTone {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

impl NoticeTone {
    fn suffix(self) -> &'static str {
        match self {
            NoticeTone::Info => "info",
            NoticeTone::Success => "success",
            NoticeTone::Warning => "warning",
            NoticeTone::Error => "error",
        }
    }

    fn icon(self) -> IconName {
        match self {
            NoticeTone::Info => IconName::Info,
            NoticeTone::Success => IconName::Check,
            NoticeTone::Warning => IconName::Alert,
            NoticeTone::Error => IconName::Close,
        }
    }
}

#[component]
pub fn Badge(#[props(default)] tone: BadgeTone, children: Element) -> Element {
    rsx! {
        span { class: "ui-badge ui-badge--{tone.suffix()}", {children} }
    }
}

#[component]
pub fn PillBadge(#[props(default = "Receta verificable".to_string())] label: String) -> Element {
    rsx! {
        span { class: "ui-pill", "{label}" }
    }
}

#[component]
pub fn Notice(#[props(default)] tone: NoticeTone, children: Element) -> Element {
    rsx! {
        div {
            class: "ui-notice ui-notice--{tone.suffix()}",
            role: (tone == NoticeTone::Error).then_some("alert"),
            Icon { name: tone.icon(), size: 20 }
            div { class: "ui-notice-body", {children} }
        }
    }
}

#[component]
pub fn ErrorText(text: String) -> Element {
    rsx! {
        p { class: "ui-error", role: "alert",
            Icon { name: IconName::Alert, size: 16 }
            "{text}"
        }
    }
}

#[component]
pub fn HintText(text: String) -> Element {
    rsx! {
        p { class: "ui-hint", "{text}" }
    }
}

#[component]
pub fn EmptyState(title: String, #[props(default)] description: String) -> Element {
    rsx! {
        div { class: "ui-empty",
            p { class: "ui-empty-title", "{title}" }
            if !description.is_empty() {
                p { class: "ui-empty-description", "{description}" }
            }
        }
    }
}

#[component]
pub fn LoadingState(#[props(default = "Cargando...".to_string())] label: String) -> Element {
    rsx! {
        div { class: "ui-loading", role: "status", aria_live: "polite",
            span { class: "ui-spinner" }
            span { "{label}" }
        }
    }
}

#[component]
pub fn ConfirmPanel(
    message: String,
    confirm_label: String,
    #[props(default = "Cancelar".to_string())] cancel_label: String,
    busy: bool,
    on_confirm: EventHandler<MouseEvent>,
    on_cancel: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "ui-confirm",
            {children}
            div { class: "ui-actions",
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: on_cancel,
                    "{cancel_label}"
                }
                Button {
                    variant: ButtonVariant::Danger,
                    disabled: busy,
                    onclick: on_confirm,
                    "{confirm_label}"
                }
            }
        }
    }
}

#[component]
pub fn DataTable(columns: Vec<String>, caption: Option<String>, children: Element) -> Element {
    rsx! {
        div { class: "ui-table-wrap",
            table { class: "ui-table",
                if let Some(caption) = caption {
                    caption { "{caption}" }
                }
                thead {
                    tr {
                        for column in columns {
                            th { scope: "col", "{column}" }
                        }
                    }
                }
                {children}
            }
        }
    }
}

#[component]
pub fn ListRow(
    title: String,
    meta: Vec<String>,
    badge: Option<Element>,
    children: Element,
) -> Element {
    let meta_text = meta.join(" · ");
    rsx! {
        div { class: "ui-row",
            div { class: "ui-row-body",
                h3 { class: "ui-row-title", "{title}" }
                if let Some(badge) = badge {
                    {badge}
                }
                if !meta_text.is_empty() {
                    p { class: "ui-row-meta", "{meta_text}" }
                }
            }
            div { class: "ui-row-actions", {children} }
        }
    }
}
