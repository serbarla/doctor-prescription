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

impl IconName {
    fn to_icon_type(self) -> icons::IconType {
        match self {
            IconName::Cross => icons::IconType::Cross,
            IconName::Shield => icons::IconType::Shield,
            IconName::Command => icons::IconType::Command,
            IconName::Windows => icons::IconType::Monitor,
            IconName::Globe => icons::IconType::Globe,
            IconName::Check => icons::IconType::Check,
            IconName::Close => icons::IconType::X,
            IconName::Alert => icons::IconType::TriangleAlert,
            IconName::Info => icons::IconType::Info,
            IconName::Qr => icons::IconType::QrCode,
            IconName::ArrowRight => icons::IconType::ArrowRight,
            IconName::Lock => icons::IconType::Lock,
            IconName::Image => icons::IconType::Image,
        }
    }
}

#[component]
pub fn Icon(
    name: IconName,
    #[props(default = 20)] size: u16,
    #[props(default)] extra_class: String,
) -> Element {
    let filled = name == IconName::Cross;
    let icon_type = name.to_icon_type();
    let class = format!(
        "ui-icon{}{}",
        if filled { " ui-icon--filled" } else { "" },
        if extra_class.is_empty() {
            "".to_string()
        } else {
            format!(" {extra_class}")
        }
    );
    rsx! {
        span {
            class: "ui-icon-wrap",
            style: "--icon-size: {size}px",
            "aria-hidden": "true",
            "focusable": "false",
            icons::DioxusIcon { icon: icon_type, class: class }
        }
    }
}
