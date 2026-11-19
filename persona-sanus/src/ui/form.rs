use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
struct FieldProps {
    label: String,
    hint: Option<String>,
    error: Option<String>,
    #[props(default)]
    wide: bool,
    #[props(default)]
    hide_label: bool,
    children: Element,
}

#[component]
fn Field(props: FieldProps) -> Element {
    let label_class = if props.hide_label {
        "ui-field-label ui-visually-hidden"
    } else {
        "ui-field-label"
    };
    rsx! {
        label {
            class: if props.wide { "ui-field ui-field--wide" } else { "ui-field" },
            span { class: "{label_class}", "{props.label}" }
            {props.children}
            if let Some(error) = props.error {
                p { class: "ui-field-error", role: "alert", "{error}" }
            } else if let Some(hint) = props.hint {
                p { class: "ui-field-hint", "{hint}" }
            }
        }
    }
}

#[component]
pub fn TextField(
    label: String,
    value: String,
    oninput: EventHandler<String>,
    #[props(default = "text".to_string())] input_type: String,
    #[props(default)] placeholder: String,
    #[props(default)] wide: bool,
    hint: Option<String>,
    error: Option<String>,
    #[props(default = "off".to_string())] autocomplete: String,
    #[props(default = true)] spellcheck: bool,
    #[props(default)] required: bool,
    #[props(default)] disabled: bool,
    #[props(default)] readonly: bool,
    min: Option<String>,
    max: Option<String>,
    step: Option<String>,
    #[props(default)] hide_label: bool,
) -> Element {
    let invalid = error.is_some();
    rsx! {
        Field { label, hint, error, wide, hide_label,
            input {
                class: "ui-input",
                r#type: input_type,
                placeholder,
                value,
                autocomplete,
                spellcheck,
                required,
                disabled,
                readonly,
                aria_invalid: "{invalid}",
                min: min.unwrap_or_default(),
                max: max.unwrap_or_default(),
                step: step.unwrap_or_default(),
                oninput: move |event| oninput.call(event.value()),
            }
        }
    }
}

#[component]
pub fn TextAreaField(
    label: String,
    value: String,
    oninput: EventHandler<String>,
    #[props(default)] placeholder: String,
    #[props(default)] wide: bool,
    hint: Option<String>,
    error: Option<String>,
    #[props(default)] required: bool,
    #[props(default)] disabled: bool,
    #[props(default)] readonly: bool,
    #[props(default)] hide_label: bool,
    rows: Option<u32>,
) -> Element {
    let invalid = error.is_some();
    let row_count = rows.unwrap_or(2) as usize;
    rsx! {
        Field { label, hint, error, wide, hide_label,
            textarea {
                class: "ui-textarea",
                placeholder,
                value,
                rows: row_count,
                required,
                disabled,
                readonly,
                aria_invalid: "{invalid}",
                oninput: move |event| oninput.call(event.value()),
            }
        }
    }
}

#[component]
pub fn SelectField(
    label: String,
    value: String,
    onchange: EventHandler<String>,
    #[props(default)] wide: bool,
    hint: Option<String>,
    error: Option<String>,
    #[props(default)] disabled: bool,
    #[props(default)] hide_label: bool,
    children: Element,
) -> Element {
    let invalid = error.is_some();
    rsx! {
        Field { label, hint, error, wide, hide_label,
            select {
                class: "ui-select",
                value,
                disabled,
                aria_invalid: "{invalid}",
                onchange: move |event| onchange.call(event.value()),
                {children}
            }
        }
    }
}

#[component]
pub fn CheckboxField(
    label: String,
    description: Option<String>,
    checked: bool,
    onchange: EventHandler<bool>,
    #[props(default)] disabled: bool,
    #[props(default)] wide: bool,
) -> Element {
    let class = if wide {
        "ui-check ui-check--wide"
    } else {
        "ui-check"
    };
    rsx! {
        label {
            class: "{class}",
            input {
                r#type: "checkbox",
                checked,
                disabled,
                onchange: move |event| onchange.call(event.checked()),
            }
            span { class: "ui-check-label", "{label}" }
            if let Some(description) = description {
                span { class: "ui-check-description", "{description}" }
            }
        }
    }
}

#[component]
pub fn FileField(
    label: String,
    accept: String,
    #[props(default)] disabled: bool,
    hint: Option<String>,
    on_change: EventHandler<FormEvent>,
    #[props(default)] wide: bool,
) -> Element {
    rsx! {
        Field { label, hint, wide,
            input {
                class: "ui-file-input",
                r#type: "file",
                accept,
                disabled,
                onchange: move |event| on_change.call(event),
            }
        }
    }
}

#[component]
pub fn SearchField(
    label: String,
    value: String,
    oninput: EventHandler<String>,
    placeholder: String,
    #[props(default)] wide: bool,
) -> Element {
    rsx! {
        Field { label, wide, hide_label: true,
            input {
                class: "ui-input",
                r#type: "search",
                placeholder,
                value,
                oninput: move |event| oninput.call(event.value()),
            }
        }
    }
}
