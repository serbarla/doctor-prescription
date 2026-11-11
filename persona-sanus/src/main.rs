mod auth;

#[cfg(feature = "server")]
mod server;
use auth::SessionUser;
use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/login")]
    AuthPage {},
    #[route("/dashboard")]
    Dashboard {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(App);
}

#[cfg(feature = "server")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = dioxus::cli_config::fullstack_address_or_localhost();
    let router = server::router().await?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;
    Ok(())
}

#[component]
fn App() -> Element {
    let mut user = use_signal(|| None::<SessionUser>);
    let mut checked = use_signal(|| false);
    use_context_provider(|| user);
    use_context_provider(|| checked);

    use_effect(move || {
        spawn(async move {
            let session = auth::fetch_session().await;
            user.set(session);
            checked.set(true);
        });
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        div {
            id: "hero",
            h1 { "PersonaSanus" }
            p { "Your prescriptions and health records, kept organized." }
            div {
                id: "links",
                Link { to: Route::AuthPage {}, "Sign in" }
                Link { to: Route::Dashboard {}, "Dashboard" }
            }
        }
    }
}

#[component]
fn Navbar() -> Element {
    let mut user = use_context::<Signal<Option<SessionUser>>>();
    let navigator = use_navigator();
    let current = user();

    rsx! {
        div {
            id: "navbar",
            Link { to: Route::Home {}, "Home" }
            Link { to: Route::Dashboard {}, "Dashboard" }
            if let Some(session) = current {
                span { "{session.email}" }
                button {
                    onclick: move |_| {
                        spawn(async move {
                            let _ = auth::sign_out().await;
                            user.set(None);
                            navigator.push(Route::Home {});
                        });
                    },
                    "Sign out"
                }
            } else {
                Link { to: Route::AuthPage {}, "Sign in" }
            }
        }
        Outlet::<Route> {}
    }
}

#[component]
fn AuthPage() -> Element {
    let mut user = use_context::<Signal<Option<SessionUser>>>();
    let mut checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut sign_up_mode = use_signal(|| false);
    let mut name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut submitting = use_signal(|| false);
    let current = user();

    if let Some(session) = current {
        return rsx! {
            div {
                id: "echo",
                h4 { "Signed in" }
                p { "{session.name}" }
                p { "{session.email}" }
                button {
                    onclick: move |_| {
                        spawn(async move {
                            let _ = auth::sign_out().await;
                            user.set(None);
                            navigator.push(Route::Home {});
                        });
                    },
                    "Sign out"
                }
            }
        };
    }

    let submit = move |_| {
        if submitting() {
            return;
        }
        let name_value = name().trim().to_string();
        let email_value = email().trim().to_string();
        let password_value = password();
        if email_value.is_empty()
            || password_value.is_empty()
            || (sign_up_mode() && name_value.is_empty())
        {
            error.set("Fill in every field.".to_string());
            return;
        }
        error.set(String::new());
        submitting.set(true);
        spawn(async move {
            let result = if sign_up_mode() {
                auth::sign_up(&name_value, &email_value, &password_value).await
            } else {
                auth::sign_in(&email_value, &password_value).await
            };
            match result {
                Ok(from_response) => {
                    let refreshed = auth::fetch_session().await;
                    let session = refreshed.or(from_response);
                    user.set(session.clone());
                    checked.set(true);
                    submitting.set(false);
                    if session.is_some() {
                        navigator.push(Route::Dashboard {});
                    } else {
                        error.set("The auth server did not return an active session.".to_string());
                    }
                }
                Err(message) => {
                    submitting.set(false);
                    error.set(message);
                }
            }
        });
    };

    rsx! {
        div {
            id: "echo",
            h4 {
                if sign_up_mode() { "Create your account" } else { "Welcome back" }
            }
            div {
                button {
                    onclick: move |_| {
                        sign_up_mode.set(false);
                        error.set(String::new());
                    },
                    "Sign in"
                }
                button {
                    onclick: move |_| {
                        sign_up_mode.set(true);
                        error.set(String::new());
                    },
                    "Sign up"
                }
            }
            if sign_up_mode() {
                input {
                    placeholder: "Name",
                    value: "{name}",
                    oninput: move |event| name.set(event.value()),
                }
            }
            input {
                r#type: "email",
                placeholder: "Email",
                value: "{email}",
                oninput: move |event| email.set(event.value()),
            }
            input {
                r#type: "password",
                placeholder: "Password",
                value: "{password}",
                oninput: move |event| password.set(event.value()),
            }
            if !error().is_empty() {
                p { "{error}" }
            }
            button {
                disabled: submitting(),
                onclick: submit,
                if submitting() {
                    "Please wait..."
                } else if sign_up_mode() {
                    "Sign up"
                } else {
                    "Sign in"
                }
            }
        }
    }
}

#[component]
fn Dashboard() -> Element {
    let mut user = use_context::<Signal<Option<SessionUser>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();

    use_effect(move || {
        if checked() && user().is_none() {
            navigator.replace(Route::AuthPage {});
        }
    });

    if !checked() {
        return rsx! {
            div {
                id: "echo",
                p { "Loading..." }
            }
        };
    }

    match user() {
        Some(session) => rsx! {
            div {
                id: "echo",
                h4 { "Dashboard" }
                p { "{session.name}" }
                p { "{session.email}" }
                button {
                    onclick: move |_| {
                        spawn(async move {
                            let _ = auth::sign_out().await;
                            user.set(None);
                            navigator.push(Route::Home {});
                        });
                    },
                    "Sign out"
                }
            }
        },
        None => rsx! {
            div {
                id: "echo",
                p { "Redirecting to sign in..." }
            }
        },
    }
}
