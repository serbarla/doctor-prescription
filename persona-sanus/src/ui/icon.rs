use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconName {
    Cross,
    Shield,
    Command,
    Windows,
    Globe,
    Check,
    Close,
    Alert,
    Info,
    Qr,
    ArrowRight,
    Lock,
    Image,
}

#[component]
pub fn Icon(
    name: IconName,
    #[props(default = 20)] size: u16,
    #[props(default)] extra_class: String,
) -> Element {
    let filled = name == IconName::Cross;
    let path = match name {
        IconName::Cross => "M5.293 6.707 11.586 13l-6.293 6.293 1.414 1.414L13 14.414l6.293 6.293 1.414-1.414L14.414 13l6.293-6.293-1.414-1.414L13 11.586 6.707 5.293z",
        IconName::Shield => "M12 3l7 3v5c0 4.5-3 7.5-7 9-4-1.5-7-4.5-7-9V6z",
        IconName::Command => "M9 6a3 3 0 1 1-3 3h12a3 3 0 1 1-3 3v6a3 3 0 1 0 3-3H6a3 3 0 1 0 3 3z",
        IconName::Windows => "M3 5l8-1v7H3zM13 3.5l8-1V11h-8zM3 13h8v7l-8-1zM13 13h8v8.5l-8-1z",
        IconName::Globe => "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zM3 12h18M12 3c2.5 2.5 4 5.5 4 9s-1.5 6.5-4 9c-2.5-2.5-4-5.5-4-9s1.5-6.5 4-9z",
        IconName::Check => "M4 12l5 5L20 6",
        IconName::Close => "M6 6l12 12M18 6 6 18",
        IconName::Alert => "M12 3l9 17H3zM12 9v5m0 3h.01",
        IconName::Info => "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zM12 10v6m0-9h.01",
        IconName::Qr => "M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM14 14h2v2h-2zM18 14h2v6h-2zM14 18h2v2h-2z",
        IconName::ArrowRight => "M4 12h16m-6-6 6 6-6 6",
        IconName::Lock => "M6 11h12v9H6zM9 11V8a3 3 0 0 1 6 0v3",
        IconName::Image => "M4 4h16v16H4zM4 16l5-5 4 4 3-3 4 4m-3-6h.01",
    };
    rsx! {
        svg {
            class: "{extra_class}",
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            "aria-hidden": "true",
            "focusable": "false",
            fill: if filled { "currentColor" } else { "none" },
            stroke: if filled { "none" } else { "currentColor" },
            stroke_width: "1.5",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: path }
        }
    }
}
