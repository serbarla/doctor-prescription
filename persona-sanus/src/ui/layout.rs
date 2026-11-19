use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SurfaceTone {
    #[default]
    Paper,
    Cream,
    Mint,
    Blossom,
}

impl SurfaceTone {
    fn suffix(self) -> &'static str {
        match self {
            SurfaceTone::Paper => "paper",
            SurfaceTone::Cream => "cream",
            SurfaceTone::Mint => "mint",
            SurfaceTone::Blossom => "blossom",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormColumns {
    #[default]
    Two,
    One,
}

impl FormColumns {
    fn suffix(self) -> &'static str {
        match self {
            FormColumns::Two => "two",
            FormColumns::One => "one",
        }
    }
}

#[component]
pub fn PageShell(id: String, children: Element) -> Element {
    rsx! {
        div { class: "ui-page", id: "{id}",
            div { class: "ui-container ui-stack", {children} }
        }
    }
}

#[component]
pub fn PageHeader(title: String, subtitle: Option<String>, children: Element) -> Element {
    rsx! {
        header { class: "ui-page-head",
            div {
                h1 { class: "ui-page-title", "{title}" }
                if let Some(subtitle) = subtitle {
                    p { class: "ui-page-subtitle", "{subtitle}" }
                }
            }
            div { class: "ui-page-head-actions", {children} }
        }
    }
}

#[component]
pub fn Card(children: Element) -> Element {
    rsx! {
        div { class: "ui-card", {children} }
    }
}

#[component]
pub fn SectionCard(title: String, description: Option<String>, children: Element) -> Element {
    rsx! {
        div { class: "ui-card",
            h2 { class: "ui-card-title", "{title}" }
            if let Some(description) = description {
                p { class: "ui-card-description", "{description}" }
            }
            {children}
        }
    }
}

#[component]
pub fn FormGrid(#[props(default)] columns: FormColumns, children: Element) -> Element {
    rsx! {
        div { class: "ui-form-grid ui-form-grid--{columns.suffix()}", {children} }
    }
}

#[component]
pub fn DetailGrid(children: Element) -> Element {
    rsx! {
        dl { class: "ui-detail-grid", {children} }
    }
}

#[component]
pub fn DetailItem(label: String, #[props(default)] wide: bool, children: Element) -> Element {
    rsx! {
        div {
            class: if wide { "ui-detail-wide" } else { "" },
            dt { "{label}" }
            dd { {children} }
        }
    }
}

#[component]
pub fn ActionRow(children: Element) -> Element {
    rsx! {
        div { class: "ui-actions", {children} }
    }
}

#[component]
pub fn FilterBar(children: Element) -> Element {
    rsx! {
        div { class: "ui-filter-bar", {children} }
    }
}

#[component]
pub fn SectionBand(
    id: Option<String>,
    #[props(default)] tone: SurfaceTone,
    children: Element,
) -> Element {
    let mut section_attributes = Vec::new();
    if let Some(value) = id {
        section_attributes.push(Attribute::new("id", value, None, false));
    }
    rsx! {
        section {
            class: "ui-band ui-band--{tone.suffix()}",
            ..section_attributes,
            div { class: "ui-container", {children} }
        }
    }
}
