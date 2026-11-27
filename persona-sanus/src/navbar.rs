use dioxus::prelude::*;
use dioxus::router::router;

use crate::ui::{
    Button, ButtonLink, ButtonVariant, LogoLockup, NavLink, PillBadge, SyncStatus,
    SyncStatusIndicator,
};
use crate::{api::Me, auth, auth::SessionUser, clear_persisted_session, probe_sync_status, Route};

#[component]
pub(crate) fn Navbar() -> Element {
    let mut user = use_context::<Signal<Option<SessionUser>>>();
    let mut actor = use_context::<Signal<Option<Me>>>();
    let mut sync = use_context::<Signal<SyncStatus>>();
    let navigator = use_navigator();
    let current = user();
    let profile = actor();
    let status = sync();
    let router = router();
    use_effect(move || {
        let _route: Route = router.current();
        spawn(async move {
            let current_status = probe_sync_status().await;
            sync.set(current_status);
        });
    });

    rsx! {
        header { id: "navbar", class: "ui-navbar",
            div { class: "ui-navbar-inner",
                div { class: "ui-navbar-brand",
                    LogoLockup {}
                    PillBadge {}
                }
                if current.is_some() {
                    nav { class: "ui-navbar-links",
                        NavLink { to: Route::Dashboard {}, label: "Panel".to_owned() }
                        if profile
                            .as_ref()
                            .is_some_and(|actor| actor.can_read_patients())
                        {
                            NavLink { to: Route::Patients {}, label: "Pacientes".to_owned() }
                        }
                        if profile
                            .as_ref()
                            .is_some_and(|actor| actor.can_read_prescriptions())
                        {
                            NavLink {
                                to: Route::Prescriptions { status: None },
                                label: "Recetas".to_owned(),
                            }
                        }
                        if profile
                            .as_ref()
                            .is_some_and(|actor| actor.can_manage_templates())
                        {
                            NavLink { to: Route::Templates {}, label: "Plantillas".to_owned() }
                        }
                        NavLink { to: Route::Settings {}, label: "Configuración".to_owned() }
                    }
                }
                div { class: "ui-navbar-account",
                    if let Some(session) = current {
                        SyncStatusIndicator { status }
                        if status.pending > 0 {
                            Link {
                                class: "ui-sync-link ui-sync-link--pending",
                                to: Route::Prescriptions { status: Some("pending_sync".to_owned()) },
                                if status.pending == 1 {
                                    "· 1 pendiente"
                                } else {
                                    "· {status.pending} pendientes"
                                }
                            }
                        }
                        if status.failed > 0 {
                            Link {
                                class: "ui-sync-link",
                                to: Route::Prescriptions { status: Some("failed".to_owned()) },
                                "{status.failed} con error"
                            }
                        }
                        span { class: "ui-navbar-user", "{session.email}" }
                        Button {
                            variant: ButtonVariant::Ghost,
                            onclick: move |_| {
                                spawn(async move {
                                    let _ = auth::sign_out().await;
                                    clear_persisted_session();
                                    actor.set(None);
                                    user.set(None);
                                    navigator.push(Route::Home {});
                                });
                            },
                            "Cerrar sesión"
                        }
                    } else {
                        ButtonLink {
                            to: Route::AuthPage {},
                            variant: ButtonVariant::Ghost,
                            active_class: None,
                            "Iniciar sesión"
                        }
                    }
                }
            }
        }
        Outlet::<Route> {}
    }
}
