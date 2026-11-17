use dioxus::prelude::*;
use dioxus::router::NavigationTarget;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Ghost,
    Danger,
}

impl ButtonVariant {
    fn suffix(self) -> &'static str {
        match self {
            ButtonVariant::Primary => "primary",
            ButtonVariant::Secondary => "secondary",
            ButtonVariant::Ghost => "ghost",
            ButtonVariant::Danger => "danger",
        }
    }
}

fn classes(base: &str, variant: ButtonVariant, extra_class: &str) -> String {
    let class = format!("{base} ui-button--{} {extra_class}", variant.suffix());
    class.trim_end().to_owned()
}

#[component]
pub fn Button(
    #[props(default)] variant: ButtonVariant,
    #[props(default = "button".to_string())] button_type: String,
    #[props(default)] disabled: bool,
    #[props(default)] loading: bool,
    #[props(default)] extra_class: String,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    rsx! {
        button {
            class: classes("ui-button", variant, &extra_class),
            r#type: button_type,
            disabled: disabled || loading,
            aria_busy: "{loading}",
            onclick: move |event| onclick.call(event),
            {children}
        }
    }
}

#[component]
pub fn ButtonLink(
    #[props(into)] to: NavigationTarget,
    #[props(default)] variant: ButtonVariant,
    active_class: Option<String>,
    #[props(default)] extra_class: String,
    children: Element,
) -> Element {
    rsx! {
        Link {
            to,
            class: classes("ui-button", variant, &extra_class),
            active_class,
            {children}
        }
    }
}

#[component]
pub fn LinkAction(
    href: String,
    #[props(default)] variant: ButtonVariant,
    target: Option<String>,
    rel: Option<String>,
    #[props(default)] extra_class: String,
    children: Element,
) -> Element {
    let mut link_attributes = Vec::new();
    if let Some(value) = target {
        link_attributes.push(Attribute::new("target", value, None, false));
    }
    if let Some(value) = rel {
        link_attributes.push(Attribute::new("rel", value, None, false));
    }
    rsx! {
        a {
            class: classes("ui-button", variant, &extra_class),
            href: "{href}",
            ..link_attributes,
            {children}
        }
    }
}

#[component]
pub fn NavLink(#[props(into)] to: NavigationTarget, label: String) -> Element {
    rsx! {
        Link {
            to,
            class: "ui-nav-link",
            active_class: "ui-nav-link--active",
            "{label}"
        }
    }
}
