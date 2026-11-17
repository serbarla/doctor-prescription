use dioxus::prelude::*;

use super::{
    Badge, BadgeTone, Button, ButtonVariant, DataTable, EmptyState, HintText, Notice, NoticeTone,
};

#[derive(Debug, Clone, PartialEq)]
pub struct BillingTransaction {
    pub id: String,
    pub plan_name: String,
    pub amount_cents: i64,
    pub tax_cents: i64,
    pub status: String,
    pub requested_at: String,
}

#[component]
pub fn BillingStatusCard(
    status: String,
    trial_ends_at: Option<String>,
    next_charge_at: Option<String>,
    grace_ends_at: Option<String>,
    #[props(default)] cancel_at_period_end: bool,
    children: Element,
) -> Element {
    let (title, copy): (&str, Option<String>) = match status.as_str() {
        "trialing" => (
            "Período de prueba",
            trial_ends_at.as_deref().map(crate::ui::format_date),
        ),
        "active" => (
            "Suscripción activa",
            next_charge_at.as_deref().map(crate::ui::format_date),
        ),
        "past_due" => (
            "Pago pendiente",
            grace_ends_at.as_deref().map(crate::ui::format_date),
        ),
        "locked" => (
            "Sincronización bloqueada",
            Some(
                "Renueva para volver a sincronizar. La atención clínica sigue disponible."
                    .to_owned(),
            ),
        ),
        "canceled" => (
            "Suscripción cancelada",
            Some("Tus datos clínicos siguen disponibles.".to_owned()),
        ),
        _ => ("Estado desconocido", None),
    };
    rsx! {
        div { class: "ui-card",
            h2 { class: "ui-card-title", "{title}" }
            if let Some(copy) = copy {
                p { "{copy}" }
            }
            if cancel_at_period_end {
                p { "Se cancela al final del período actual." }
            }
            {children}
        }
    }
}

#[component]
pub fn PlanCard(
    code: String,
    name: String,
    amount_cents: i64,
    interval_months: i64,
    features: Vec<String>,
    #[props(default)] selected: bool,
    on_select: EventHandler<String>,
) -> Element {
    let interval = if interval_months == 12 {
        "/año"
    } else {
        "/mes"
    };
    let yearly = interval_months == 12;
    rsx! {
        div { class: "ui-plan",
            h3 { class: "ui-card-title", "{name}" }
            p {
                span { class: "ui-plan-price", "{format_usd_cents(amount_cents)}" }
                span { class: "ui-plan-interval", "{interval}" }
            }
            p { class: "ui-plan-interval", "Impuestos incluidos" }
            if selected {
                Badge { tone: BadgeTone::Mint, "Plan actual" }
            } else {
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| on_select.call(code.clone()),
                    "Elegir {name}"
                }
            }
            if yearly {
                Badge { tone: BadgeTone::Mint, "2 meses gratis" }
            }
            ul { class: "ui-plan-features",
                for feature in features {
                    li { "{feature}" }
                }
            }
        }
    }
}

#[component]
pub fn PaymentMethodCard(
    status: String,
    consented_at: Option<String>,
    on_revoke: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div { class: "ui-card",
            h2 { class: "ui-card-title", "Método de pago" }
            if status == "active" {
                p { "Token de tarjeta activo · PayPhone" }
                if let Some(consented_at) = consented_at {
                    p {
                        "Consentimiento: "
                        "{crate::ui::format_date(&consented_at)}"
                    }
                }
                Button {
                    variant: ButtonVariant::Danger,
                    onclick: move |event| on_revoke.call(event),
                    "Revocar token"
                }
            } else {
                Badge { tone: BadgeTone::Neutral, "Sin token activo" }
                HintText { text: "No hay un token de tarjeta guardado en esta clínica." }
            }
            p { class: "ui-hint",
                "Solo Visa y Mastercard. No se guarda el número completo ni el CVV."
            }
        }
    }
}

#[component]
pub fn BillingBanner(status: String, grace_ends_at: Option<String>) -> Element {
    match status.as_str() {
        "past_due" => {
            let copy = match grace_ends_at {
                Some(date) => {
                    format!(
                        "Pago pendiente · gracia hasta {}.",
                        crate::ui::format_date(&date)
                    )
                }
                None => "Pago pendiente.".to_owned(),
            };
            rsx! {
                Notice { tone: NoticeTone::Warning, "{copy}" }
            }
        }
        "locked" => rsx! {
            Notice { tone: NoticeTone::Error,
                "Sincronización bloqueada — renueva para sincronizar. La atención clínica sigue disponible."
            }
        },
        "canceled" => rsx! {
            Notice { tone: NoticeTone::Warning,
                "Suscripción cancelada. Tus datos clínicos siguen disponibles."
            }
        },
        _ => rsx! {},
    }
}

#[component]
pub fn TransactionTable(transactions: Vec<BillingTransaction>) -> Element {
    if transactions.is_empty() {
        return rsx! {
            EmptyState { title: "Sin transacciones" }
        };
    }
    rsx! {
        DataTable {
            columns: vec![
                "Fecha".to_string(),
                "Plan".to_string(),
                "Total".to_string(),
                "IVA".to_string(),
                "Estado".to_string(),
            ],
            caption: None,
            tbody {
                for transaction in transactions {
                    tr {
                        td { "{crate::ui::format_date(&transaction.requested_at)}" }
                        td { "{transaction.plan_name}" }
                        td { "{format_usd_cents(transaction.amount_cents)}" }
                        td { "{format_usd_cents(transaction.tax_cents)}" }
                        td {
                            match transaction.status.as_str() {
                                "pending" => rsx! { Badge { tone: BadgeTone::Cream, "Pendiente" } },
                                "approved" => rsx! { Badge { tone: BadgeTone::Mint, "Aprobada" } },
                                "canceled" => rsx! { Badge { tone: BadgeTone::Neutral, "Cancelada" } },
                                "failed" => rsx! { Badge { tone: BadgeTone::Blossom, "Fallida" } },
                                "reversed" => rsx! { Badge { tone: BadgeTone::Plum, "Reversada" } },
                                other => rsx! { Badge { tone: BadgeTone::Neutral, "{other}" } },
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn format_usd_cents(cents: i64) -> String {
    let negative = cents < 0;
    let absolute = cents.unsigned_abs();
    let dollars = absolute / 100;
    let remainder = absolute % 100;
    let grouped = group_dollars(dollars);
    let sign = if negative { "-" } else { "" };
    format!("{sign}${grouped},{remainder:02} USD")
}

fn group_dollars(value: u64) -> String {
    let digits = value.to_string();
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            grouped.push('.');
        }
        grouped.push(digit);
    }
    grouped
}
