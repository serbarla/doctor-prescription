mod api;

#[cfg(feature = "server")]
mod api_server;
mod auth;
mod device;

#[cfg(feature = "server")]
mod assets;
#[cfg(feature = "server")]
mod auth_server;
#[cfg(feature = "server")]
mod db;
mod forms;
mod http;
#[cfg(any(feature = "server", feature = "desktop"))]
mod pdf;
#[cfg(feature = "server")]
mod pdf_server;
#[cfg(feature = "server")]
mod schema;
#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
mod service;
use api::Me;
use auth::SessionUser;
use dioxus::prelude::*;
use dioxus::router::Navigator;

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
    #[route("/patients")]
    Patients {},
    #[route("/patients/new")]
    PatientNew {},
    #[route("/patients/:id")]
    PatientDetail { id: String },
    #[route("/patients/:id/edit")]
    PatientEdit { id: String },
    #[route("/settings")]
    Settings {},
    #[route("/settings/users")]
    SettingsUsers {},
    #[route("/settings/profile")]
    SettingsProfile {},
    #[route("/settings/clinic")]
    SettingsClinic {},
    #[route("/templates")]
    Templates {},
    #[route("/templates/new")]
    TemplateNew {},
    #[route("/templates/:id/edit")]
    TemplateEdit { id: String },
    #[route("/templates/:id/preview")]
    TemplatePreview { id: String },
    #[route("/prescriptions")]
    Prescriptions {},
    #[route("/prescriptions/new?:patient_id&:template_id")]
    PrescriptionNew {
        patient_id: Option<String>,
        template_id: Option<String>,
    },
    #[route("/prescriptions/:id")]
    PrescriptionDetail { id: String },
    #[route("/prescriptions/:id/preview")]
    PrescriptionPreview { id: String },
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
    let mut actor = use_signal(|| None::<Me>);
    let mut checked = use_signal(|| false);
    use_context_provider(|| user);
    use_context_provider(|| actor);
    use_context_provider(|| checked);

    use_effect(move || {
        spawn(async move {
            let session = auth::fetch_session().await;
            let profile = match session {
                Some(_) => {
                    let _ = api::register_current_device().await;
                    api::fetch_me().await.ok()
                }
                None => None,
            };
            actor.set(profile);
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
    let mut actor = use_context::<Signal<Option<Me>>>();
    let navigator = use_navigator();
    let current = user();
    let profile = actor();

    rsx! {
        div {
            id: "navbar",
            Link { to: Route::Home {}, "Home" }
            if current.is_some() {
                Link { to: Route::Dashboard {}, "Dashboard" }
                if profile
                    .as_ref()
                    .is_some_and(|actor| actor.can_read_patients())
                {
                    Link { to: Route::Patients {}, "Pacientes" }
                }
                if profile
                    .as_ref()
                    .is_some_and(|actor| actor.can_read_prescriptions())
                {
                    Link { to: Route::Prescriptions {}, "Recetas" }
                }
                if profile
                    .as_ref()
                    .is_some_and(|actor| actor.can_manage_templates())
                {
                    Link { to: Route::Templates {}, "Plantillas" }
                }
                Link { to: Route::SettingsProfile {}, "Mi perfil" }
                if profile.as_ref().is_some_and(|actor| actor.is_administrator()) {
                    Link { to: Route::SettingsUsers {}, "Usuarios" }
                }
                Link { to: Route::Settings {}, "Configuración" }
            }
            if let Some(session) = current {
                span { "{session.email}" }
                button {
                    onclick: move |_| {
                        spawn(async move {
                            let _ = auth::sign_out().await;
                            actor.set(None);
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
    let mut actor = use_context::<Signal<Option<Me>>>();
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
                            actor.set(None);
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
                    let profile = match session {
                        Some(_) => {
                            let _ = api::register_current_device().await;
                            api::fetch_me().await.ok()
                        }
                        None => None,
                    };
                    actor.set(profile);
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
    let mut actor = use_context::<Signal<Option<Me>>>();
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
                if let Some(profile) = actor() {
                    p { "Rol: {role_label(profile.role.as_deref())}" }
                    if let Some(clinic) = profile.clinic {
                        p { "Clínica: {clinic.name}" }
                    } else {
                        p { "Sin clínica asignada." }
                    }
                }
                button {
                    onclick: move |_| {
                        spawn(async move {
                            let _ = auth::sign_out().await;
                            actor.set(None);
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

#[component]
fn Settings() -> Element {
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();

    use_effect(move || {
        if checked() {
            match actor() {
                None => {
                    navigator.replace(Route::AuthPage {});
                }
                Some(profile) if !profile.is_administrator() => {
                    navigator.replace(Route::SettingsProfile {});
                }
                _ => {}
            }
        }
    });

    let profile = actor();
    if !checked() || profile.is_none() {
        return rsx! {
            div { id: "echo", p { "Cargando..." } }
        };
    }
    match profile {
        Some(profile) if profile.is_administrator() => rsx! {
            div {
                id: "echo",
                h4 { "Configuración" }
                p { "Administra el acceso del personal y tu perfil." }
                Link { to: Route::SettingsUsers {}, "Usuarios y roles" }
                Link { to: Route::SettingsClinic {}, "Clínica y biblioteca" }
                Link { to: Route::SettingsProfile {}, "Mi perfil" }
            }
        },
        _ => rsx! {
            div { id: "echo", p { "Redirigiendo..." } }
        },
    }
}

#[component]
fn SettingsClinic() -> Element {
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut name = use_signal(String::new);
    let mut address = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut llm_url = use_signal(String::new);
    let mut message = use_signal(String::new);
    let mut saving = use_signal(|| false);
    let mut uploading = use_signal(|| false);
    let mut generation = use_signal(|| 0_u32);
    let mut settings = use_signal(|| None::<api::ClinicSettings>);
    let mut library = use_signal(|| None::<api::AssetLibrary>);

    use_effect(move || {
        let _ = generation();
        if !checked() {
            return;
        }
        match actor() {
            None => {
                navigator.replace(Route::AuthPage {});
                return;
            }
            Some(profile) if !profile.is_administrator() => {
                navigator.replace(Route::SettingsProfile {});
                return;
            }
            _ => {}
        }
        spawn(async move {
            match api::fetch_clinic_settings().await {
                Ok(found) => {
                    name.set(found.name.clone());
                    address.set(found.address.clone().unwrap_or_default());
                    phone.set(found.phone.clone().unwrap_or_default());
                    llm_url.set(found.llm_url_template.clone().unwrap_or_default());
                    settings.set(Some(found));
                }
                Err(error) => message.set(error),
            }
            match api::fetch_assets().await {
                Ok(loaded) => library.set(Some(loaded)),
                Err(error) => message.set(error),
            }
        });
    });

    if !checked() || actor().is_none() {
        return rsx! {
            div { id: "settings-page", p { "Cargando..." } }
        };
    }
    if let Some(profile) = actor() {
        if !profile.is_administrator() {
            return rsx! {
                div { id: "settings-page", p { "Redirigiendo..." } }
            };
        }
    }

    let clinic = settings();
    let logo_url = clinic
        .as_ref()
        .and_then(|clinic| clinic.logo.as_ref())
        .and_then(|logo| logo.url.clone());
    let library_state = library();
    let quota = library_state
        .as_ref()
        .map(|library| (library.quota.used, library.quota.limit));
    let library_assets: Vec<api::AssetRow> = library_state
        .as_ref()
        .map(|library| {
            library
                .assets
                .iter()
                .filter(|asset| asset.kind == "image")
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    rsx! {
        div {
            id: "settings-page",
            div {
                class: "page-head",
                div {
                    h4 { "Clínica" }
                    p { "Datos de la clínica, logo y biblioteca de imágenes." }
                }
                Link { class: "button secondary", to: Route::Settings {}, "Volver" }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            section {
                class: "editor-section",
                h4 { "Datos de la clínica" }
                div {
                    class: "form-grid",
                    label {
                        class: "field",
                        span { "Nombre" }
                        input {
                            value: "{name}",
                            oninput: move |event| name.set(event.value()),
                        }
                    }
                    label {
                        class: "field",
                        span { "Dirección" }
                        input {
                            value: "{address}",
                            oninput: move |event| address.set(event.value()),
                        }
                    }
                    label {
                        class: "field",
                        span { "Teléfono" }
                        input {
                            value: "{phone}",
                            oninput: move |event| phone.set(event.value()),
                        }
                    }
                    label {
                        class: "field wide",
                        span { "Plantilla del enlace LLM" }
                        input {
                            placeholder: "https://chat.openai.com/?q=",
                            value: "{llm_url}",
                            oninput: move |event| llm_url.set(event.value()),
                        }
                    }
                }
                div {
                    class: "actions",
                    button {
                        disabled: saving(),
                        onclick: move |_| {
                            if saving() {
                                return;
                            }
                            saving.set(true);
                            let value = name();
                            let street = address();
                            let telephone = phone();
                            let llm = llm_url();
                            spawn(async move {
                                match api::update_clinic_settings(&value, &street, &telephone, &llm).await {
                                    Ok(_) => {
                                        message.set("Datos de la clínica actualizados.".to_owned());
                                        generation += 1;
                                    }
                                    Err(error) => message.set(error),
                                }
                                saving.set(false);
                            });
                        },
                        if saving() { "Guardando..." } else { "Guardar datos" }
                    }
                }
            }
            section {
                class: "editor-section",
                h4 { "Logo" }
                if let Some(url) = logo_url {
                    img { class: "clinic-logo", src: "{url}", alt: "Logo de la clínica" }
                } else {
                    p { class: "hint", "La clínica todavía no tiene logo." }
                }
                label {
                    class: "field",
                    span { "Subir logo (JPEG o PNG, máximo 10 MB)" }
                    input {
                        r#type: "file",
                        accept: "image/jpeg,image/png",
                        disabled: uploading(),
                        onchange: move |event| {
                            let Some(file) = event.files().first().cloned() else {
                                return;
                            };
                            if uploading() {
                                return;
                            }
                            uploading.set(true);
                            let file_name = file.name();
                            spawn(async move {
                                match file.read_bytes().await {
                                    Ok(bytes) => {
                                        match api::upload_logo(file_name, bytes.to_vec()).await {
                                            Ok(_) => {
                                                message.set("Logo actualizado.".to_owned());
                                                generation += 1;
                                            }
                                            Err(error) => message.set(error),
                                        }
                                    }
                                    Err(_) => message.set("No se pudo leer el archivo.".to_owned()),
                                }
                                uploading.set(false);
                            });
                        },
                    }
                }
            }
            section {
                class: "editor-section",
                h4 { "Biblioteca de imágenes" }
                if let Some((used, limit)) = quota {
                    p { class: "hint", "Uso de la biblioteca: {used} de {limit}." }
                }
                for asset in library_assets.iter() {
                    div {
                        key: "{asset.id}",
                        class: "asset-row",
                        div {
                            strong {
                                "{asset.original_name.clone().unwrap_or_else(|| asset.id.clone())}"
                            }
                            p {
                                class: "hint",
                                "{asset.width.unwrap_or(0)} × {asset.height.unwrap_or(0)} · {asset.size_bytes} bytes · sha256 {asset.sha256.chars().take(12).collect::<String>()}"
                            }
                        }
                        button {
                            class: "danger",
                            onclick: {
                                let asset_id = asset.id.clone();
                                move |_| {
                                    let asset_id = asset_id.clone();
                                    spawn(async move {
                                        match api::delete_asset(&asset_id).await {
                                            Ok(()) => {
                                                message.set("Imagen eliminada.".to_owned());
                                                generation += 1;
                                            }
                                            Err(error) => message.set(error),
                                        }
                                    });
                                }
                            },
                            "Eliminar"
                        }
                    }
                }
                label {
                    class: "field",
                    span { "Subir imagen de la biblioteca (JPEG o PNG, máximo 10 MB)" }
                    input {
                        r#type: "file",
                        accept: "image/jpeg,image/png",
                        disabled: uploading(),
                        onchange: move |event| {
                            let Some(file) = event.files().first().cloned() else {
                                return;
                            };
                            if uploading() {
                                return;
                            }
                            uploading.set(true);
                            let file_name = file.name();
                            spawn(async move {
                                match file.read_bytes().await {
                                    Ok(bytes) => {
                                        match api::upload_asset(file_name, bytes.to_vec()).await {
                                            Ok(_) => {
                                                message.set("Imagen agregada a la biblioteca.".to_owned());
                                                generation += 1;
                                            }
                                            Err(error) => message.set(error),
                                        }
                                    }
                                    Err(_) => message.set("No se pudo leer el archivo.".to_owned()),
                                }
                                uploading.set(false);
                            });
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn SettingsUsers() -> Element {
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut users = use_signal(Vec::<api::UserRow>::new);
    let mut devices = use_signal(Vec::<api::DeviceRow>::new);
    let refresh = use_signal(|| 0);
    let mut message = use_signal(String::new);

    use_effect(move || {
        let _ = refresh();
        if !checked() {
            return;
        }
        let Some(profile) = actor() else {
            navigator.replace(Route::AuthPage {});
            return;
        };
        if !profile.is_administrator() {
            return;
        }
        spawn(async move {
            match api::fetch_users().await {
                Ok(rows) => users.set(rows),
                Err(error) => message.set(error),
            }
            match api::fetch_devices().await {
                Ok(rows) => devices.set(rows),
                Err(error) => message.set(error),
            }
        });
    });

    let profile = actor();
    if !checked() {
        return rsx! {
            div { id: "echo", p { "Cargando..." } }
        };
    }
    if !profile
        .as_ref()
        .is_some_and(|profile| profile.is_administrator())
    {
        return rsx! {
            div {
                id: "echo",
                h4 { "Acceso denegado" }
                p { "Solo el administrador puede gestionar usuarios y roles." }
                Link { to: Route::SettingsProfile {}, "Ir a mi perfil" }
            }
        };
    }

    let rows = users();
    let device_rows = devices();
    rsx! {
        div {
            id: "echo",
            h4 { "Usuarios y roles" }
            p { "Asigna un rol o deshabilita el acceso de cada persona." }
            if !message().is_empty() {
                p { "{message}" }
            }
            for row in rows {
                div {
                    key: "{row.profile_id}",
                    p { "{row.display_name}" }
                    p { "{row.email}" }
                    p { "Alta: {format_date(&row.created_at)}" }
                    p { "Estado: {status_label(&row.status)}" }
                    select {
                        value: row.role.clone().unwrap_or_default(),
                        onchange: {
                            let profile_id = row.profile_id.clone();
                            let mut refresh = refresh;
                            let mut message = message;
                            move |event| {
                                let value = event.value();
                                if value.is_empty() {
                                    return;
                                }
                                let profile_id = profile_id.clone();
                                spawn(async move {
                                    match api::set_role(&profile_id, &value).await {
                                        Ok(()) => refresh += 1,
                                        Err(error) => message.set(error),
                                    }
                                });
                            }
                        },
                        option { value: "", disabled: true, "Sin rol" }
                        option { value: "administrator", "Administrador" }
                        option { value: "doctor", "Doctor" }
                        option { value: "nursing", "Enfermería" }
                        option { value: "secretary", "Secretaría" }
                    }
                    if row.role.is_some() {
                        button {
                            onclick: {
                                let profile_id = row.profile_id.clone();
                                let mut refresh = refresh;
                                let mut message = message;
                                move |_| {
                                    let profile_id = profile_id.clone();
                                    spawn(async move {
                                        match api::remove_member(&profile_id).await {
                                            Ok(()) => refresh += 1,
                                            Err(error) => message.set(error),
                                        }
                                    });
                                }
                            },
                            "Quitar del equipo"
                        }
                    }
                    if row.status == "active" {
                        button {
                            onclick: {
                                let profile_id = row.profile_id.clone();
                                let mut refresh = refresh;
                                let mut message = message;
                                move |_| {
                                    let profile_id = profile_id.clone();
                                    spawn(async move {
                                        match api::disable_profile(&profile_id, true).await {
                                            Ok(()) => refresh += 1,
                                            Err(error) => message.set(error),
                                        }
                                    });
                                }
                            },
                            "Deshabilitar"
                        }
                    } else {
                        button {
                            onclick: {
                                let profile_id = row.profile_id.clone();
                                let mut refresh = refresh;
                                let mut message = message;
                                move |_| {
                                    let profile_id = profile_id.clone();
                                    spawn(async move {
                                        match api::disable_profile(&profile_id, false).await {
                                            Ok(()) => refresh += 1,
                                            Err(error) => message.set(error),
                                        }
                                    });
                                }
                            },
                            "Habilitar"
                        }
                    }
                }
            }
            if !device_rows.is_empty() {
                div {
                    h4 { "Dispositivos" }
                    p { "Equipos registrados y su último acceso." }
                    for device in device_rows {
                        div {
                            key: "{device.id}",
                            p { "{device.profile_name} · {device_name_label(&device)}" }
                            p {
                                "{platform_label(&device.platform)} · Última conexión: {last_seen_label(device.last_seen_at.as_deref())}"
                            }
                            if device.app_version.is_some() {
                                p { "Versión: {device.app_version.clone().unwrap_or_default()}" }
                            }
                            if device.revoked_at.is_some() {
                                p { "Estado: Revocado" }
                            } else {
                                button {
                                    class: "danger",
                                    onclick: {
                                        let device_id = device.id.clone();
                                        let mut refresh = refresh;
                                        let mut message = message;
                                        move |_| {
                                            let device_id = device_id.clone();
                                            spawn(async move {
                                                match api::revoke_device(&device_id).await {
                                                    Ok(()) => refresh += 1,
                                                    Err(error) => message.set(error),
                                                }
                                            });
                                        }
                                    },
                                    "Revocar"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SettingsProfile() -> Element {
    let mut actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut initialized = use_signal(|| false);
    let mut display_name = use_signal(String::new);
    let mut specialty = use_signal(String::new);
    let mut license_no = use_signal(String::new);
    let mut national_id = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut message = use_signal(String::new);
    let mut saving = use_signal(|| false);

    use_effect(move || {
        if initialized() {
            return;
        }
        if !checked() {
            return;
        }
        let Some(profile) = actor() else {
            navigator.replace(Route::AuthPage {});
            return;
        };
        display_name.set(profile.profile.display_name);
        if let Some(doctor) = profile.doctor {
            specialty.set(doctor.specialty.unwrap_or_default());
            license_no.set(doctor.license_no.unwrap_or_default());
            national_id.set(doctor.national_id.unwrap_or_default());
            phone.set(doctor.phone.unwrap_or_default());
            email.set(doctor.email.unwrap_or_default());
        }
        initialized.set(true);
    });

    let current = actor();
    if !checked() {
        return rsx! {
            div { id: "echo", p { "Cargando..." } }
        };
    }
    let Some(profile) = current else {
        return rsx! {
            div { id: "echo", p { "Redirigiendo..." } }
        };
    };
    let is_doctor = profile.is_doctor();

    rsx! {
        div {
            id: "echo",
            h4 { "Mi perfil" }
            if let Some(clinic) = profile.clinic {
                p { "Clínica: {clinic.name}" }
            } else {
                p { "Sin clínica asignada. Un administrador debe asignarte un rol." }
            }
            p { "Rol: {role_label(profile.role.as_deref())}" }
            div {
                input {
                    placeholder: "Nombre visible",
                    value: "{display_name}",
                    oninput: move |event| display_name.set(event.value()),
                }
            }
            if is_doctor {
                div {
                    input {
                        placeholder: "Especialidad",
                        value: "{specialty}",
                        oninput: move |event| specialty.set(event.value()),
                    }
                    input {
                        placeholder: "Número de licencia",
                        value: "{license_no}",
                        oninput: move |event| license_no.set(event.value()),
                    }
                    input {
                        placeholder: "Cédula",
                        value: "{national_id}",
                        oninput: move |event| national_id.set(event.value()),
                    }
                    input {
                        placeholder: "Teléfono",
                        value: "{phone}",
                        oninput: move |event| phone.set(event.value()),
                    }
                    input {
                        placeholder: "Correo",
                        value: "{email}",
                        oninput: move |event| email.set(event.value()),
                    }
                }
            }
            if !message().is_empty() {
                p { "{message}" }
            }
            button {
                disabled: saving(),
                onclick: move |_| {
                    if saving() {
                        return;
                    }
                    saving.set(true);
                    message.set(String::new());
                    spawn(async move {
                        let result = api::update_profile(
                            &display_name(),
                            &specialty(),
                            &license_no(),
                            &national_id(),
                            &phone(),
                            &email(),
                        )
                        .await;
                        saving.set(false);
                        match result {
                            Ok(updated) => {
                                actor.set(Some(updated));
                                message.set("Perfil guardado.".to_string());
                            }
                            Err(error) => message.set(error),
                        }
                    });
                },
                if saving() { "Guardando..." } else { "Guardar" }
            }
        }
    }
}

#[component]
fn Patients() -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut query = use_signal(String::new);
    let mut patients = use_signal(Vec::<api::Patient>::new);
    let mut message = use_signal(String::new);

    use_effect(move || {
        if !checked() {
            return;
        }
        if user().is_none() {
            navigator.replace(Route::AuthPage {});
            return;
        }
        let term = query();
        spawn(async move {
            match api::fetch_patients(&term).await {
                Ok(found) => {
                    message.set(String::new());
                    patients.set(found);
                }
                Err(error) => message.set(error),
            }
        });
    });

    if !checked() {
        return rsx! {
            div { id: "patients-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "patients-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_read_patients() {
        return rsx! {
            div {
                id: "patients-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Necesitas un rol en la clínica para consultar pacientes." }
                }
            }
        };
    }

    rsx! {
        div {
            id: "patients-page",
            div {
                class: "page-head",
                div {
                    h4 { "Pacientes" }
                    p { "Busca por nombre o cédula dentro de tu clínica." }
                }
                if profile.can_write_patients() {
                    Link { class: "button", to: Route::PatientNew {}, "Nuevo paciente" }
                }
            }
            div {
                class: "search-row",
                input {
                    r#type: "search",
                    placeholder: "Nombre o cédula",
                    value: "{query}",
                    oninput: move |event| query.set(event.value()),
                }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            div {
                class: "patient-list",
                if patients().is_empty() {
                    p { class: "hint", "No hay pacientes para esta búsqueda." }
                }
                for patient in patients() {
                    div {
                        key: "{patient.id}",
                        class: "patient-row",
                        div {
                            p { class: "patient-name", "{patient.full_name}" }
                            p {
                                class: "patient-meta",
                                "Cédula: {patient.national_id.clone().unwrap_or_else(|| \"—\".to_string())}"
                            }
                            if let Some(birth_date) = patient.birth_date.clone() {
                                p { class: "patient-meta", "Nacimiento: {format_date(&birth_date)}" }
                            }
                        }
                        Link {
                            class: "button secondary",
                            to: Route::PatientDetail { id: patient.id.clone() },
                            "Ver ficha"
                        }
                    }
                }
            }
        }
    }
}

fn submit_new_patient(
    mut saving: Signal<bool>,
    mut message: Signal<String>,
    mut warning_ids: Signal<Vec<String>>,
    navigator: Navigator,
    form: api::PatientForm,
    confirm: bool,
) {
    if saving() {
        return;
    }
    if form.full_name.trim().is_empty() {
        message.set("Ingresa el nombre del paciente.".to_string());
        return;
    }
    if !form.birth_date.is_empty() && !valid_birth_date(&form.birth_date) {
        message.set("Ingresa una fecha de nacimiento válida.".to_string());
        return;
    }
    saving.set(true);
    message.set(String::new());
    spawn(async move {
        match api::create_patient(&form, confirm).await {
            Ok(api::PatientCreateOutcome::Created(patient)) => {
                saving.set(false);
                navigator.push(Route::PatientDetail { id: patient.id });
            }
            Ok(api::PatientCreateOutcome::LikelyDuplicate(ids)) => {
                saving.set(false);
                warning_ids.set(ids);
                message.set("Ya existe un paciente con un nombre similar.".to_string());
            }
            Err(error) => {
                saving.set(false);
                message.set(error);
            }
        }
    });
}

#[component]
fn PatientNew() -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut full_name = use_signal(String::new);
    let mut national_id = use_signal(String::new);
    let mut birth_date = use_signal(String::new);
    let mut sex = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let warning_ids = use_signal(Vec::<String>::new);
    let message = use_signal(String::new);
    let saving = use_signal(|| false);

    use_effect(move || {
        if checked() && user().is_none() {
            navigator.replace(Route::AuthPage {});
        }
    });

    if !checked() {
        return rsx! {
            div { id: "patients-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "patients-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_write_patients() {
        return rsx! {
            div {
                id: "patients-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Solo doctor, enfermería y secretaría pueden crear pacientes." }
                    Link { to: Route::Patients {}, "Volver a pacientes" }
                }
            }
        };
    }

    let form = move || api::PatientForm {
        full_name: full_name(),
        national_id: national_id(),
        birth_date: birth_date(),
        sex: sex(),
        phone: phone(),
        notes: notes(),
    };

    rsx! {
        div {
            id: "patients-page",
            div {
                class: "page-head",
                div {
                    h4 { "Nuevo paciente" }
                    p { "Los datos se guardan en la clínica actual." }
                }
                Link { class: "button secondary", to: Route::Patients {}, "Cancelar" }
            }
            div {
                class: "form-grid",
                label {
                    class: "field",
                    span { "Nombres y apellidos" }
                    input {
                        value: "{full_name}",
                        oninput: move |event| full_name.set(event.value()),
                    }
                }
                label {
                    class: "field",
                    span { "Cédula" }
                    input {
                        value: "{national_id}",
                        oninput: move |event| national_id.set(event.value()),
                    }
                }
                label {
                    class: "field",
                    span { "Fecha de nacimiento" }
                    input {
                        r#type: "date",
                        value: "{birth_date}",
                        oninput: move |event| birth_date.set(event.value()),
                    }
                }
                label {
                    class: "field",
                    span { "Sexo" }
                    select {
                        value: "{sex}",
                        onchange: move |event| sex.set(event.value()),
                        option { value: "", "Sin especificar" }
                        option { value: "female", "Femenino" }
                        option { value: "male", "Masculino" }
                        option { value: "other", "Otro" }
                        option { value: "unspecified", "Prefiere no decir" }
                    }
                }
                label {
                    class: "field",
                    span { "Teléfono" }
                    input {
                        r#type: "tel",
                        value: "{phone}",
                        oninput: move |event| phone.set(event.value()),
                    }
                }
                label {
                    class: "field wide",
                    span { "Notas" }
                    textarea {
                        value: "{notes}",
                        oninput: move |event| notes.set(event.value()),
                    }
                }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            if !warning_ids().is_empty() {
                div {
                    class: "notice",
                    p { "Posible duplicado por nombre. Coincidencias: {warning_ids().join(\", \")}" }
                    button {
                        disabled: saving(),
                        onclick: move |_| {
                            submit_new_patient(
                                saving,
                                message,
                                warning_ids,
                                navigator,
                                form(),
                                true,
                            );
                        },
                        "Crear de todas formas"
                    }
                }
            }
            div {
                class: "actions",
                button {
                    disabled: saving(),
                    onclick: move |_| {
                        submit_new_patient(
                            saving,
                            message,
                            warning_ids,
                            navigator,
                            form(),
                            false,
                        );
                    },
                    if saving() { "Guardando..." } else { "Guardar paciente" }
                }
            }
        }
    }
}

#[component]
fn PatientDetail(id: String) -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut patient = use_signal(|| None::<api::Patient>);
    let mut message = use_signal(String::new);
    let mut confirm_delete = use_signal(|| false);
    let mut deleting = use_signal(|| false);
    let load_id = id.clone();

    use_effect(move || {
        if !checked() {
            return;
        }
        if user().is_none() {
            navigator.replace(Route::AuthPage {});
            return;
        }
        let load_id = load_id.clone();
        spawn(async move {
            match api::fetch_patient(&load_id).await {
                Ok(found) => patient.set(Some(found)),
                Err(error) => message.set(error),
            }
        });
    });

    if !checked() {
        return rsx! {
            div { id: "patients-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "patients-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_read_patients() {
        return rsx! {
            div {
                id: "patients-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Necesitas un rol en la clínica para consultar pacientes." }
                }
            }
        };
    }
    let Some(patient) = patient() else {
        return rsx! {
            div {
                id: "patients-page",
                if message().is_empty() {
                    p { "Cargando..." }
                } else {
                    p { class: "error", "{message}" }
                    Link { to: Route::Patients {}, "Volver a pacientes" }
                }
            }
        };
    };

    rsx! {
        div {
            id: "patients-page",
            div {
                class: "page-head",
                div {
                    h4 { "{patient.full_name}" }
                    p { "Ficha del paciente" }
                }
                div {
                    class: "actions",
                    if profile.can_write_patients() {
                        Link {
                            class: "button secondary",
                            to: Route::PatientEdit { id: patient.id.clone() },
                            "Editar"
                        }
                    } else {
                        span { class: "badge", "Solo lectura" }
                    }
                }
            }
            div {
                class: "detail-grid",
                div { class: "field", span { "Cédula" }, span { "{patient.national_id.clone().unwrap_or_else(|| \"—\".to_string())}" } }
                div { class: "field", span { "Fecha de nacimiento" }, span { "{optional_date(patient.birth_date.as_deref())}" } }
                div { class: "field", span { "Sexo" }, span { "{sex_label(patient.sex.as_deref())}" } }
                div { class: "field", span { "Teléfono" }, span { "{patient.phone.clone().unwrap_or_else(|| \"—\".to_string())}" } }
                div { class: "field wide", span { "Notas" }, span { class: "notes", "{patient.notes.clone().unwrap_or_else(|| \"—\".to_string())}" } }
            }
            div {
                class: "actions",
                if profile.can_create_prescriptions() {
                    Link {
                        class: "button",
                        to: Route::PrescriptionNew {
                            patient_id: Some(patient.id.clone()),
                            template_id: None,
                        },
                        "Nueva receta"
                    }
                }
                Link {
                    class: "button secondary",
                    to: Route::Prescriptions {},
                    "Historial de recetas"
                }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            if confirm_delete() {
                div {
                    class: "notice",
                    p { "¿Eliminar este paciente? La ficha se oculta de la búsqueda y queda registrada en auditoría." }
                    div {
                        class: "actions",
                        button {
                            disabled: deleting(),
                            onclick: {
                                let patient_id = patient.id.clone();
                                move |_| {
                                    if deleting() {
                                        return;
                                    }
                                    deleting.set(true);
                                    let patient_id = patient_id.clone();
                                    spawn(async move {
                                        match api::delete_patient(&patient_id).await {
                                            Ok(()) => {
                                                navigator.push(Route::Patients {});
                                            }
                                            Err(error) => {
                                                deleting.set(false);
                                                message.set(error);
                                            }
                                        }
                                    });
                                }
                            },
                            if deleting() { "Eliminando..." } else { "Confirmar eliminación" }
                        }
                        button {
                            class: "secondary",
                            disabled: deleting(),
                            onclick: move |_| confirm_delete.set(false),
                            "Cancelar"
                        }
                    }
                }
            } else if profile.can_write_patients() {
                div {
                    class: "actions",
                    button {
                        class: "danger",
                        onclick: move |_| confirm_delete.set(true),
                        "Eliminar paciente"
                    }
                }
            }
            div {
                class: "actions",
                Link { class: "button secondary", to: Route::Patients {}, "Volver a pacientes" }
            }
        }
    }
}

fn submit_edit_patient(
    mut saving: Signal<bool>,
    mut message: Signal<String>,
    navigator: Navigator,
    patient_id: String,
    form: api::PatientForm,
) {
    if saving() {
        return;
    }
    if form.full_name.trim().is_empty() {
        message.set("Ingresa el nombre del paciente.".to_string());
        return;
    }
    if !form.birth_date.is_empty() && !valid_birth_date(&form.birth_date) {
        message.set("Ingresa una fecha de nacimiento válida.".to_string());
        return;
    }
    saving.set(true);
    message.set(String::new());
    spawn(async move {
        match api::update_patient(&patient_id, &form).await {
            Ok(patient) => {
                saving.set(false);
                navigator.push(Route::PatientDetail { id: patient.id });
            }
            Err(error) => {
                saving.set(false);
                message.set(error);
            }
        }
    });
}

#[component]
fn PatientEdit(id: String) -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut full_name = use_signal(String::new);
    let mut national_id = use_signal(String::new);
    let mut birth_date = use_signal(String::new);
    let mut sex = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut message = use_signal(String::new);
    let saving = use_signal(|| false);
    let mut initialized = use_signal(|| false);
    let patient_id = id.clone();

    use_effect({
        let patient_id = patient_id.clone();
        move || {
            if initialized() || !checked() {
                return;
            }
            if user().is_none() {
                navigator.replace(Route::AuthPage {});
                return;
            }
            let patient_id = patient_id.clone();
            spawn(async move {
                match api::fetch_patient(&patient_id).await {
                    Ok(patient) => {
                        full_name.set(patient.full_name);
                        national_id.set(patient.national_id.unwrap_or_default());
                        birth_date.set(patient.birth_date.unwrap_or_default());
                        sex.set(patient.sex.unwrap_or_default());
                        phone.set(patient.phone.unwrap_or_default());
                        notes.set(patient.notes.unwrap_or_default());
                        initialized.set(true);
                    }
                    Err(error) => message.set(error),
                }
            });
        }
    });

    if !checked() {
        return rsx! {
            div { id: "patients-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "patients-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_write_patients() {
        return rsx! {
            div {
                id: "patients-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Solo doctor, enfermería y secretaría pueden editar pacientes." }
                    Link { to: Route::PatientDetail { id: patient_id.clone() }, "Ver ficha" }
                }
            }
        };
    }

    let form = move || api::PatientForm {
        full_name: full_name(),
        national_id: national_id(),
        birth_date: birth_date(),
        sex: sex(),
        phone: phone(),
        notes: notes(),
    };

    rsx! {
        div {
            id: "patients-page",
            div {
                class: "page-head",
                div {
                    h4 { "Editar paciente" }
                    p { "Los cambios se auditan en la ficha del paciente." }
                }
                Link {
                    class: "button secondary",
                    to: Route::PatientDetail { id: patient_id.clone() },
                    "Cancelar"
                }
            }
            if !initialized() && message().is_empty() {
                p { "Cargando..." }
            }
            div {
                class: "form-grid",
                label {
                    class: "field",
                    span { "Nombres y apellidos" }
                    input {
                        value: "{full_name}",
                        oninput: move |event| full_name.set(event.value()),
                    }
                }
                label {
                    class: "field",
                    span { "Cédula" }
                    input {
                        value: "{national_id}",
                        oninput: move |event| national_id.set(event.value()),
                    }
                }
                label {
                    class: "field",
                    span { "Fecha de nacimiento" }
                    input {
                        r#type: "date",
                        value: "{birth_date}",
                        oninput: move |event| birth_date.set(event.value()),
                    }
                }
                label {
                    class: "field",
                    span { "Sexo" }
                    select {
                        value: "{sex}",
                        onchange: move |event| sex.set(event.value()),
                        option { value: "", "Sin especificar" }
                        option { value: "female", "Femenino" }
                        option { value: "male", "Masculino" }
                        option { value: "other", "Otro" }
                        option { value: "unspecified", "Prefiere no decir" }
                    }
                }
                label {
                    class: "field",
                    span { "Teléfono" }
                    input {
                        r#type: "tel",
                        value: "{phone}",
                        oninput: move |event| phone.set(event.value()),
                    }
                }
                label {
                    class: "field wide",
                    span { "Notas" }
                    textarea {
                        value: "{notes}",
                        oninput: move |event| notes.set(event.value()),
                    }
                }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            div {
                class: "actions",
                button {
                    disabled: saving(),
                    onclick: {
                        let patient_id = patient_id.clone();
                        move |_| {
                            submit_edit_patient(
                                saving,
                                message,
                                navigator,
                                patient_id.clone(),
                                form(),
                            );
                        }
                    },
                    if saving() { "Guardando..." } else { "Guardar cambios" }
                }
            }
        }
    }
}

#[component]
fn Templates() -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut templates = use_signal(Vec::<api::Template>::new);
    let mut message = use_signal(String::new);
    let refresh = use_signal(|| 0);

    use_effect(move || {
        let _ = refresh();
        if !checked() {
            return;
        }
        if user().is_none() {
            navigator.replace(Route::AuthPage {});
            return;
        }
        spawn(async move {
            match api::fetch_templates().await {
                Ok(rows) => {
                    message.set(String::new());
                    templates.set(rows);
                }
                Err(error) => message.set(error),
            }
        });
    });

    if !checked() {
        return rsx! {
            div { id: "templates-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "templates-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_manage_templates() {
        return rsx! {
            div {
                id: "templates-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Solo doctor y administrador pueden gestionar plantillas." }
                }
            }
        };
    }
    let rows = templates();

    rsx! {
        div {
            id: "templates-page",
            div {
                class: "page-head",
                div {
                    h4 { "Plantillas" }
                    p { "Plantillas del médico y maestras de la clínica, con su versión vigente." }
                }
                Link { class: "button", to: Route::TemplateNew {}, "Nueva plantilla" }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            div {
                class: "template-list",
                if rows.is_empty() {
                    p { class: "hint", "Todavía no hay plantillas." }
                }
                for template in rows {
                    div {
                        key: "{template.id}",
                        class: "template-row",
                        div {
                            p { class: "patient-name", "{template.name}" }
                            p {
                                class: "patient-meta",
                                "{template_owner_label(&template.owner_type)} · {template_status_label(&template.status)}"
                            }
                            if let Some(version) = template.current_version.clone() {
                                p {
                                    class: "patient-meta",
                                    "Versión vigente: {version.version_no} ({version_status_label(&version.status)})"
                                }
                            } else {
                                p { class: "patient-meta", "Sin versión publicada." }
                            }
                        }
                        if template.status != "archived" {
                            div {
                                class: "actions",
                                Link {
                                    class: "button secondary",
                                    to: Route::TemplateEdit { id: template.id.clone() },
                                    "Editar"
                                }
                                Link {
                                    class: "button secondary",
                                    to: Route::TemplatePreview { id: template.id.clone() },
                                    "Vista previa"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TemplateNew() -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut name = use_signal(String::new);
    let mut message = use_signal(String::new);
    let mut saving = use_signal(|| false);

    use_effect(move || {
        if checked() && user().is_none() {
            navigator.replace(Route::AuthPage {});
        }
    });

    if !checked() {
        return rsx! {
            div { id: "templates-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "templates-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_manage_templates() {
        return rsx! {
            div {
                id: "templates-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Solo doctor y administrador pueden crear plantillas." }
                    Link { to: Route::Templates {}, "Volver a plantillas" }
                }
            }
        };
    }
    let owner = if profile.is_administrator() {
        "Maestra de la clínica"
    } else {
        "Plantilla del médico"
    };

    rsx! {
        div {
            id: "templates-page",
            div {
                class: "page-head",
                div {
                    h4 { "Nueva plantilla" }
                    p { "Se crea un borrador; el contenido se define en el editor." }
                }
                Link { class: "button secondary", to: Route::Templates {}, "Cancelar" }
            }
            label {
                class: "field",
                span { "Nombre" }
                input {
                    placeholder: "Nombre de la plantilla",
                    value: "{name}",
                    oninput: move |event| name.set(event.value()),
                }
            }
            p { class: "hint", "Tipo: {owner}" }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            div {
                class: "actions",
                button {
                    disabled: saving(),
                    onclick: move |_| {
                        if saving() {
                            return;
                        }
                        let value = name().trim().to_owned();
                        if value.is_empty() {
                            message.set("Escribe el nombre de la plantilla.".to_owned());
                            return;
                        }
                        let owner_type = if profile.is_administrator() {
                            "clinic"
                        } else {
                            "doctor"
                        };
                        saving.set(true);
                        message.set(String::new());
                        spawn(async move {
                            match api::create_template(&value, owner_type).await {
                                Ok(template) => {
                                    navigator.push(Route::TemplateEdit { id: template.id });
                                }
                                Err(error) => {
                                    saving.set(false);
                                    message.set(error);
                                }
                            }
                        });
                    },
                    if saving() { "Creando..." } else { "Crear plantilla" }
                }
            }
        }
    }
}

#[component]
fn TemplateEdit(id: String) -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut template = use_signal(|| None::<api::Template>);
    let mut saved_draft = use_signal(|| None::<String>);
    let mut confirming_archive = use_signal(|| false);
    let mut message = use_signal(String::new);
    let mut generation = use_signal(|| 0);
    let load_id = id.clone();
    let template_id = id.clone();

    use_effect(move || {
        let _ = generation();
        if !checked() {
            return;
        }
        if user().is_none() {
            navigator.replace(Route::AuthPage {});
            return;
        }
        let load_id = load_id.clone();
        spawn(async move {
            match api::fetch_template(&load_id).await {
                Ok(found) => {
                    let draft = found
                        .versions
                        .iter()
                        .find(|version| version.status == "draft")
                        .map(|version| version.id.clone());
                    saved_draft.set(draft);
                    template.set(Some(found));
                }
                Err(error) => message.set(error),
            }
        });
    });

    if !checked() {
        return rsx! {
            div { id: "templates-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "templates-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_manage_templates() {
        return rsx! {
            div {
                id: "templates-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Solo doctor y administrador pueden editar plantillas." }
                }
            }
        };
    }
    let Some(model) = template() else {
        return rsx! {
            div {
                id: "templates-page",
                if message().is_empty() {
                    p { "Cargando..." }
                } else {
                    p { class: "error", "{message}" }
                    Link { to: Route::Templates {}, "Volver a plantillas" }
                }
            }
        };
    };
    let draft = model
        .versions
        .iter()
        .find(|version| version.status == "draft");
    let base = draft
        .or(model.current_version.as_ref())
        .or(model.versions.first());
    let initial_doc = base
        .map(|version| version.doc_json.clone())
        .unwrap_or_else(forms::default_document_json);
    let initial_margin = base
        .map(|version| forms::template_theme_margin(version.theme.as_deref()))
        .unwrap_or(15);
    let version_id = draft.map(|version| version.id.clone());
    let archived = model.status == "archived";

    rsx! {
        div {
            id: "templates-page",
            div {
                class: "page-head",
                div {
                    h4 { "{model.name}" }
                    p { "{template_owner_label(&model.owner_type)} · {template_status_label(&model.status)}" }
                }
                div {
                    class: "actions",
                    Link {
                        class: "button secondary",
                        to: Route::TemplatePreview { id: model.id.clone() },
                        "Vista previa"
                    }
                    Link { class: "button secondary", to: Route::Templates {}, "Volver" }
                }
            }
            if archived {
                div {
                    class: "notice",
                    p { "La plantilla está archivada y no admite cambios." }
                }
            } else {
                forms::TemplateDocumentForm {
                    key: "{model.id}-{generation}",
                    template_id: template_id.clone(),
                    version_id,
                    initial_doc,
                    initial_margin_mm: initial_margin,
                    on_saved: move |(version_id, _, _)| {
                        saved_draft.set(Some(version_id));
                        message.set("Borrador guardado.".to_owned());
                    },
                }
                if !message().is_empty() {
                    p { class: "error", "{message}" }
                }
                div {
                    class: "actions",
                    button {
                        disabled: saved_draft().is_none(),
                        onclick: {
                            let template_id = template_id.clone();
                            move |_| {
                                let Some(version_id) = saved_draft() else {
                                    return;
                                };
                                let template_id = template_id.clone();
                                spawn(async move {
                                    match api::publish_template(&template_id, &version_id).await {
                                        Ok(published) => {
                                            saved_draft.set(None);
                                            generation += 1;
                                            template.set(Some(published));
                                            message.set("Versión publicada e inmutable.".to_owned());
                                        }
                                        Err(error) => message.set(error),
                                    }
                                });
                            }
                        },
                        "Publicar versión"
                    }
                    button {
                        class: "secondary",
                        disabled: !matches!(model.status.as_str(), "draft" | "published"),
                        onclick: move |_| {
                            generation += 1;
                        },
                        "Recargar"
                    }
                    if confirming_archive() {
                        button {
                            class: "danger",
                            onclick: {
                                let template_id = template_id.clone();
                                move |_| {
                                    let template_id = template_id.clone();
                                    spawn(async move {
                                        match api::archive_template(&template_id).await {
                                            Ok(archived) => {
                                                saved_draft.set(None);
                                                confirming_archive.set(false);
                                                template.set(Some(archived));
                                                message.set("Plantilla archivada.".to_owned());
                                            }
                                            Err(error) => message.set(error),
                                        }
                                    });
                                }
                            },
                            "Confirmar archivo"
                        }
                        button {
                            class: "secondary",
                            onclick: move |_| confirming_archive.set(false),
                            "Cancelar"
                        }
                    } else {
                        button {
                            class: "danger",
                            onclick: move |_| confirming_archive.set(true),
                            "Archivar plantilla"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TemplatePreview(id: String) -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut template = use_signal(|| None::<api::Template>);
    let mut selected = use_signal(|| None::<String>);
    let mut margin = use_signal(|| 15_u32);
    let mut message = use_signal(String::new);
    let load_id = id.clone();

    use_effect(move || {
        if !checked() {
            return;
        }
        if user().is_none() {
            navigator.replace(Route::AuthPage {});
            return;
        }
        let load_id = load_id.clone();
        spawn(async move {
            match api::fetch_template(&load_id).await {
                Ok(found) => {
                    let chosen = found.current_version.as_ref().or(found.versions.first());
                    selected.set(chosen.map(|version| version.id.clone()));
                    margin.set(
                        chosen
                            .map(|version| forms::template_theme_margin(version.theme.as_deref()))
                            .unwrap_or(15),
                    );
                    template.set(Some(found));
                }
                Err(error) => message.set(error),
            }
        });
    });

    if !checked() {
        return rsx! {
            div { id: "templates-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "templates-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_manage_templates() {
        return rsx! {
            div {
                id: "templates-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Solo doctor y administrador pueden ver la vista previa." }
                }
            }
        };
    }
    let Some(model) = template() else {
        return rsx! {
            div {
                id: "templates-page",
                if message().is_empty() {
                    p { "Cargando..." }
                } else {
                    p { class: "error", "{message}" }
                    Link { to: Route::Templates {}, "Volver a plantillas" }
                }
            }
        };
    };
    let version = selected()
        .and_then(|id| {
            model
                .versions
                .iter()
                .find(|version| version.id == id)
                .cloned()
        })
        .or(model.current_version.clone())
        .or_else(|| model.versions.first().cloned());
    let Some(version) = version else {
        return rsx! {
            div {
                id: "templates-page",
                div { class: "notice",
                    p { "La plantilla todavía no tiene versiones para previsualizar." }
                    Link { to: Route::TemplateEdit { id: model.id.clone() }, "Abrir editor" }
                }
            }
        };
    };
    let warnings = forms::parse_document(&version.doc_json)
        .map(|document| forms::document_warnings(&document))
        .unwrap_or_default();
    let warnings_joined = warnings.join(", ");
    let versions = model.versions.clone();

    rsx! {
        div {
            id: "templates-page",
            div {
                class: "page-head",
                div {
                    h4 { "Vista previa A4 — {model.name}" }
                    p { "Proyección 210 × 297 mm con valores de ejemplo." }
                }
                div {
                    class: "actions",
                    Link { to: Route::Templates {}, "Volver a plantillas" }
                    Link {
                        class: "button secondary",
                        to: Route::TemplateEdit { id: model.id.clone() },
                        "Abrir editor"
                    }
                }
            }
            div {
                class: "preview-controls",
                label {
                    class: "field",
                    span { "Versión" }
                    select {
                        value: "{selected().unwrap_or_default()}",
                        onchange: move |event| {
                            let value = event.value();
                            if let Some(version) = model.versions.iter().find(|version| version.id == value) {
                                margin.set(forms::template_theme_margin(version.theme.as_deref()));
                            }
                            selected.set(Some(value));
                        },
                        for version in versions {
                            option {
                                value: "{version.id}",
                                "Versión {version.version_no} — {version_status_label(&version.status)}"
                            }
                        }
                    }
                }
                label {
                    class: "field",
                    span { "Márgenes (mm)" }
                    input {
                        r#type: "number",
                        min: "5",
                        max: "40",
                        value: "{margin}",
                        oninput: move |event| {
                            if let Ok(value) = event.value().parse::<u32>() {
                                margin.set(value.clamp(5, 40));
                            }
                        },
                    }
                }
            }
            if !warnings.is_empty() {
                div {
                    class: "notice",
                    p { "Variables no válidas: {warnings_joined}" }
                    p { class: "hint", "En la proyección se marcan como texto de advertencia, nunca como {{clave}}." }
                }
            }
            div {
                class: "preview-scroll",
                forms::A4Preview {
                    doc_json: version.doc_json.clone(),
                    values: forms::sample_values(),
                    margin_mm: margin(),
                }
            }
        }
    }
}

fn template_status_label(status: &str) -> &'static str {
    match status {
        "draft" => "Borrador",
        "published" => "Publicada",
        "archived" => "Archivada",
        _ => "Desconocido",
    }
}

fn version_status_label(status: &str) -> &'static str {
    match status {
        "draft" => "Borrador",
        "published" => "Publicada",
        "archived" => "Archivada",
        _ => "Desconocido",
    }
}

fn template_owner_label(owner_type: &str) -> &'static str {
    match owner_type {
        "doctor" => "Plantilla del médico",
        "clinic" => "Maestra de la clínica",
        _ => "Plantilla",
    }
}

fn valid_birth_date(value: &str) -> bool {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
}

fn optional_date(value: Option<&str>) -> String {
    match value {
        Some(date) => format_date(date),
        None => "—".to_string(),
    }
}

fn sex_label(sex: Option<&str>) -> &'static str {
    match sex {
        Some("female") => "Femenino",
        Some("male") => "Masculino",
        Some("other") => "Otro",
        Some("unspecified") => "Prefiere no decir",
        _ => "Sin especificar",
    }
}

fn role_label(role: Option<&str>) -> &'static str {
    match role {
        Some("administrator") => "Administrador",
        Some("doctor") => "Doctor",
        Some("nursing") => "Enfermería",
        Some("secretary") => "Secretaría",
        _ => "Sin rol",
    }
}

fn status_label(status: &str) -> &'static str {
    if status == "active" {
        "Activo"
    } else {
        "Deshabilitado"
    }
}

fn platform_label(platform: &str) -> &'static str {
    match platform {
        "macos" => "macOS",
        "windows" => "Windows",
        "web" => "Web",
        _ => "Desconocido",
    }
}

fn device_name_label(device: &api::DeviceRow) -> String {
    device
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| platform_label(&device.platform).to_owned())
}

fn last_seen_label(value: Option<&str>) -> String {
    match value {
        Some(value) => format_timestamp(value),
        None => "Nunca".to_owned(),
    }
}

fn format_timestamp(value: &str) -> String {
    match chrono::DateTime::parse_from_rfc3339(value) {
        Ok(parsed) => parsed.format("%d-%m-%Y %H:%M").to_string(),
        Err(_) => value.to_owned(),
    }
}

fn format_date(value: &str) -> String {
    let date = value.get(..10).unwrap_or(value);
    match chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(parsed) => parsed.format("%d-%m-%Y").to_string(),
        Err(_) => value.to_owned(),
    }
}

fn prescription_status_label(status: &str) -> &'static str {
    match status {
        "draft" => "Borrador",
        "signed" => "Firmada",
        "superseded" => "Reemplazada",
        "voided" => "Anulada",
        _ => "Desconocida",
    }
}

fn llm_explain_url(template: Option<&str>, medication: &str) -> Option<String> {
    let name = medication.trim();
    if name.is_empty() {
        return None;
    }
    let encoded = crate::http::encode_query(name);
    match template.map(str::trim).filter(|value| !value.is_empty()) {
        Some(template) => Some(
            template
                .replace("{med}", &encoded)
                .replace("{medication}", &encoded),
        ),
        None => Some(format!("https://chat.openai.com/?q={encoded}")),
    }
}

fn new_prescription_item_form() -> api::PrescriptionItemForm {
    api::PrescriptionItemForm::default()
}

#[component]
fn Prescriptions() -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut rows = use_signal(Vec::<api::Prescription>::new);
    let mut query = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut date = use_signal(String::new);
    let mut message = use_signal(String::new);
    let mut loading = use_signal(|| true);

    use_effect(move || {
        if !checked() {
            return;
        }
        if user().is_none() {
            navigator.replace(Route::AuthPage {});
            return;
        }
        let selected_status = status();
        let selected_date = date();
        loading.set(true);
        spawn(async move {
            match api::fetch_prescriptions("", &selected_status, &selected_date).await {
                Ok(found) => {
                    rows.set(found);
                    message.set(String::new());
                }
                Err(error) => message.set(error),
            }
            loading.set(false);
        });
    });

    if !checked() {
        return rsx! {
            div { id: "prescriptions-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "prescriptions-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_read_prescriptions() {
        return rsx! {
            div {
                id: "prescriptions-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Necesitas un rol en la clínica para consultar recetas." }
                }
            }
        };
    }
    let search = query().trim().to_lowercase();
    let visible = rows()
        .into_iter()
        .filter(|row| search.is_empty() || row.patient_name.to_lowercase().contains(&search))
        .collect::<Vec<_>>();

    rsx! {
        div {
            id: "prescriptions-page",
            div {
                class: "page-head",
                div {
                    h4 { "Recetas" }
                    p { "Historial con filtros por paciente, fecha y estado." }
                }
                if profile.can_create_prescriptions() {
                    Link {
                        class: "button",
                        to: Route::PrescriptionNew { patient_id: None, template_id: None },
                        "Nueva receta"
                    }
                }
            }
            div {
                class: "prescription-filters",
                input {
                    r#type: "search",
                    placeholder: "Paciente",
                    value: "{query}",
                    oninput: move |event| query.set(event.value()),
                }
                select {
                    value: "{status}",
                    onchange: move |event| status.set(event.value()),
                    option { value: "", "Todos los estados" }
                    option { value: "draft", "Borradores" }
                    option { value: "signed", "Firmadas" }
                    option { value: "superseded", "Reemplazadas" }
                    option { value: "voided", "Anuladas" }
                }
                input {
                    r#type: "date",
                    value: "{date}",
                    oninput: move |event| date.set(event.value()),
                }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            div {
                class: "patient-list",
                if loading() {
                    p { class: "hint", "Cargando recetas..." }
                } else if visible.is_empty() {
                    p { class: "hint", "No hay recetas para estos filtros." }
                }
                for row in visible {
                    div {
                        key: "{row.id}",
                        class: "patient-row",
                        div {
                            p { class: "patient-name", "{row.patient_name}" }
                            p { class: "patient-meta",
                                "{prescription_status_label(&row.status)} · {format_date(&row.created_at)}"
                                if let Some(number) = row.rx_number {
                                    " · No. {number}"
                                }
                            }
                        }
                        Link {
                            class: "button secondary",
                            to: Route::PrescriptionDetail { id: row.id.clone() },
                            "Abrir"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PrescriptionNew(patient_id: Option<String>, template_id: Option<String>) -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut patients = use_signal(Vec::<api::Patient>::new);
    let mut templates = use_signal(Vec::<api::Template>::new);
    let mut selected_patient = use_signal(|| patient_id.clone().unwrap_or_default());
    let mut selected_template = use_signal(|| template_id.clone().unwrap_or_default());
    let mut diagnosis = use_signal(String::new);
    let mut validity = use_signal(String::new);
    let mut instructions = use_signal(String::new);
    let mut message = use_signal(String::new);
    let mut saving = use_signal(|| false);
    let mut loaded = use_signal(|| false);

    use_effect(move || {
        if loaded() || !checked() {
            return;
        }
        if user().is_none() {
            navigator.replace(Route::AuthPage {});
            return;
        }
        spawn(async move {
            match api::fetch_patients("").await {
                Ok(found) => patients.set(found),
                Err(error) => message.set(error),
            }
            match api::fetch_templates().await {
                Ok(found) => templates.set(found),
                Err(error) => message.set(error),
            }
            loaded.set(true);
        });
    });

    if !checked() {
        return rsx! {
            div { id: "prescriptions-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "prescriptions-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_create_prescriptions() {
        return rsx! {
            div {
                id: "prescriptions-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "Solo doctor o enfermería pueden crear recetas." }
                    Link { to: Route::Prescriptions {}, "Volver a recetas" }
                }
            }
        };
    }

    let published = templates()
        .into_iter()
        .filter(|template| {
            template.current_version_id.is_some()
                && template
                    .current_version
                    .as_ref()
                    .is_some_and(|version| version.status == "published")
        })
        .collect::<Vec<_>>();

    rsx! {
        div {
            id: "prescriptions-page",
            div {
                class: "page-head",
                div {
                    h4 { "Nueva receta" }
                    p { "Paciente y plantilla publicada." }
                }
                Link { class: "button secondary", to: Route::Prescriptions {}, "Cancelar" }
            }
            div {
                class: "form-grid",
                label {
                    class: "field",
                    span { "Paciente" }
                    select {
                        value: "{selected_patient}",
                        onchange: move |event| selected_patient.set(event.value()),
                        option { value: "", "Selecciona un paciente" }
                        for patient in patients() {
                            option { value: "{patient.id}", "{patient.full_name}" }
                        }
                    }
                }
                label {
                    class: "field",
                    span { "Plantilla" }
                    select {
                        value: "{selected_template}",
                        onchange: move |event| selected_template.set(event.value()),
                        option { value: "", "Selecciona una plantilla" }
                        for template in published {
                            option {
                                value: "{template.current_version_id.clone().unwrap_or_default()}",
                                "{template.name}"
                            }
                        }
                    }
                }
                label {
                    class: "field",
                    span { "Diagnóstico" }
                    textarea {
                        value: "{diagnosis}",
                        oninput: move |event| diagnosis.set(event.value()),
                    }
                }
                label {
                    class: "field",
                    span { "Vigencia" }
                    input {
                        value: "{validity}",
                        oninput: move |event| validity.set(event.value()),
                    }
                }
                label {
                    class: "field wide",
                    span { "Indicaciones generales" }
                    textarea {
                        value: "{instructions}",
                        oninput: move |event| instructions.set(event.value()),
                    }
                }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            div {
                class: "actions",
                button {
                    disabled: saving(),
                    onclick: move |_| {
                        if saving() {
                            return;
                        }
                        let patient = selected_patient();
                        let template = selected_template();
                        if patient.is_empty() || template.is_empty() {
                            message.set("Selecciona paciente y plantilla.".to_string());
                            return;
                        }
                        saving.set(true);
                        message.set(String::new());
                        let diagnosis = diagnosis();
                        let validity = validity();
                        let instructions = instructions();
                        spawn(async move {
                            match api::create_prescription(
                                &patient,
                                &template,
                                "",
                                &diagnosis,
                                &validity,
                                &instructions,
                            )
                            .await
                            {
                                Ok(prescription) => {
                                    saving.set(false);
                                    navigator.push(Route::PrescriptionDetail {
                                        id: prescription.id,
                                    });
                                }
                                Err(error) => {
                                    saving.set(false);
                                    message.set(error);
                                }
                            }
                        });
                    },
                    if saving() { "Creando..." } else { "Crear borrador" }
                }
            }
        }
    }
}

fn reload_prescription(
    mut prescription: Signal<Option<api::Prescription>>,
    mut items: Signal<Vec<api::PrescriptionItemForm>>,
    mut message: Signal<String>,
    id: String,
) {
    spawn(async move {
        match api::fetch_prescription(&id).await {
            Ok(found) => {
                items.set(
                    found
                        .items
                        .iter()
                        .map(|item| api::PrescriptionItemForm {
                            id: Some(item.id.clone()),
                            med_name: item.med_name.clone(),
                            dose: item.dose.clone().unwrap_or_default(),
                            route: item.route.clone().unwrap_or_default(),
                            frequency: item.frequency.clone().unwrap_or_default(),
                            duration: item.duration.clone().unwrap_or_default(),
                            quantity: item.quantity.clone().unwrap_or_default(),
                            notes: item.notes.clone().unwrap_or_default(),
                        })
                        .collect(),
                );
                prescription.set(Some(found));
            }
            Err(error) => message.set(error),
        }
    });
}

#[component]
fn PrescriptionPreview(id: String) -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut preview = use_signal(|| None::<api::PrescriptionPreview>);
    let mut message = use_signal(String::new);
    let mut signing = use_signal(|| false);
    let mut needs_ack = use_signal(|| false);
    let mut generation = use_signal(|| 0_u32);
    let prescription_id = id.clone();

    use_effect(move || {
        if !checked() {
            return;
        }
        if user().is_none() {
            navigator.replace(Route::AuthPage {});
            return;
        }
        let _ = generation();
        let load_id = prescription_id.clone();
        spawn(async move {
            match api::fetch_prescription_preview(&load_id).await {
                Ok(found) => preview.set(Some(found)),
                Err(error) => message.set(error),
            }
        });
    });

    if !checked() {
        return rsx! {
            div { id: "prescriptions-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "prescriptions-page", p { "Redirigiendo..." } }
        };
    };
    if !profile.can_read_prescriptions() {
        return rsx! {
            div {
                id: "prescriptions-page",
                div { class: "notice",
                    h4 { "Acceso denegado" }
                    p { "No tienes permiso para ver esta receta." }
                }
            }
        };
    }
    let Some(current) = preview() else {
        return rsx! {
            div {
                id: "prescriptions-page",
                if message().is_empty() {
                    p { "Cargando vista previa..." }
                } else {
                    p { class: "error", "{message}" }
                    Link { to: Route::Prescriptions {}, "Volver a recetas" }
                }
            }
        };
    };
    let is_draft = current.status == "draft";
    let can_sign = is_draft && profile.can_sign_prescriptions();
    let patient_name = current
        .variables
        .iter()
        .find(|variable| variable.key == "patient.name")
        .map(|variable| variable.value.clone())
        .unwrap_or_else(|| "Receta".to_owned());
    let values: Vec<(String, String)> = current
        .variables
        .iter()
        .map(|variable| (variable.key.clone(), variable.value.clone()))
        .collect();
    let image_sources: Vec<(String, String)> = current
        .image_sources
        .iter()
        .map(|source| (source.asset_id.clone(), source.data_url.clone()))
        .collect();
    let meta = forms::DocumentMeta {
        verification_code: current.verification_code.clone(),
        rx_number: current.rx_number,
        issued_at: Some(current.issued_at.clone()),
        provisional: current.provisional,
        qr_data_url: current.qr_data_url.clone(),
    };
    let pdf_url = format!("/api/prescriptions/{}/pdf", current.id);
    let preview_id = current.id.clone();

    rsx! {
        div {
            id: "prescriptions-page",
            div {
                class: "page-head",
                div {
                    h4 { "Vista previa A4 — {patient_name}" }
                    p {
                        "{prescription_status_label(&current.status)}"
                        if let Some(number) = current.rx_number {
                            " · No. {number}"
                        }
                        " · {current.issued_at}"
                    }
                }
                div {
                    class: "actions",
                    Link {
                        class: "button secondary",
                        to: Route::PrescriptionDetail { id: current.id.clone() },
                        "Volver a la receta"
                    }
                    if is_draft {
                        if can_sign {
                            button {
                                disabled: signing(),
                                onclick: {
                                    let preview_id = preview_id.clone();
                                    move |_| {
                                        if signing() {
                                            return;
                                        }
                                        signing.set(true);
                                        needs_ack.set(false);
                                        let preview_id = preview_id.clone();
                                        spawn(async move {
                                            match api::sign_prescription(&preview_id, false).await {
                                                Ok(_) => {
                                                    signing.set(false);
                                                    generation += 1;
                                                }
                                                Err(error) => {
                                                    signing.set(false);
                                                    if error.contains("variables desconocidas") {
                                                        needs_ack.set(true);
                                                    }
                                                    message.set(error);
                                                }
                                            }
                                        });
                                    }
                                },
                                if signing() { "Firmando..." } else { "Firmar y emitir" }
                            }
                            if needs_ack() {
                                button {
                                    class: "secondary",
                                    onclick: {
                                        let preview_id = preview_id.clone();
                                        move |_| {
                                            let preview_id = preview_id.clone();
                                            spawn(async move {
                                                match api::sign_prescription(&preview_id, true).await {
                                                    Ok(_) => {
                                                        needs_ack.set(false);
                                                        generation += 1;
                                                    }
                                                    Err(error) => message.set(error),
                                                }
                                            });
                                        }
                                    },
                                    "Firmar reconociendo las advertencias"
                                }
                            }
                        }
                    } else {
                        a {
                            class: "button",
                            href: "{pdf_url}",
                            "Descargar PDF"
                        }
                        a {
                            class: "button secondary",
                            href: "{pdf_url}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            "Imprimir"
                        }
                        {
                            #[cfg(target_arch = "wasm32")]
                            rsx! {
                                button {
                                    class: "secondary",
                                    onclick: {
                                        let pdf_url = pdf_url.clone();
                                        move |_| {
                                            let pdf_url = pdf_url.clone();
                                            spawn(async move {
                                                if let Some(window) = web_sys::window() {
                                                    let origin = window
                                                        .location()
                                                        .origin()
                                                        .unwrap_or_default();
                                                    let data = web_sys::ShareData::new();
                                                    data.set_title("Receta médica");
                                                    data.set_url(&format!("{origin}{pdf_url}"));
                                                    let _ = window.navigator().share_with_data(&data);
                                                }
                                            });
                                        }
                                    },
                                    "Compartir"
                                }
                            }
                        }
                    }
                }
            }
            if current.provisional {
                div {
                    class: "notice",
                    p { "Documento provisional: se muestra sin QR escaneable hasta sincronizar." }
                }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            div {
                class: "preview-scroll",
                forms::A4Preview {
                    doc_json: current.doc_json.clone(),
                    values,
                    margin_mm: current.margin_mm,
                    medications: Some(current.medications.clone()),
                    meta: Some(meta),
                    image_sources,
                }
            }
        }
    }
}

#[component]
fn PrescriptionDetail(id: String) -> Element {
    let user = use_context::<Signal<Option<SessionUser>>>();
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut prescription = use_signal(|| None::<api::Prescription>);
    let mut items = use_signal(Vec::<api::PrescriptionItemForm>::new);
    let mut block = use_signal(|| None::<api::RxBlockStatus>);
    let mut message = use_signal(String::new);
    let mut draft_diagnosis = use_signal(String::new);
    let mut draft_validity = use_signal(String::new);
    let mut draft_instructions = use_signal(String::new);
    let mut initialized = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut signing = use_signal(|| false);
    let mut needs_ack = use_signal(|| false);
    let mut void_reason = use_signal(String::new);
    let mut show_void = use_signal(|| false);
    let prescription_id = use_signal(|| id.clone());

    use_effect(move || {
        if !checked() {
            return;
        }
        if user().is_none() {
            navigator.replace(Route::AuthPage {});
            return;
        }
        let load_id = prescription_id();
        spawn(async move {
            match api::fetch_prescription(&load_id).await {
                Ok(found) => {
                    draft_diagnosis.set(found.diagnosis.clone().unwrap_or_default());
                    draft_validity.set(found.validity.clone().unwrap_or_default());
                    draft_instructions.set(found.instructions.clone().unwrap_or_default());
                    items.set(
                        found
                            .items
                            .iter()
                            .map(|item| api::PrescriptionItemForm {
                                id: Some(item.id.clone()),
                                med_name: item.med_name.clone(),
                                dose: item.dose.clone().unwrap_or_default(),
                                route: item.route.clone().unwrap_or_default(),
                                frequency: item.frequency.clone().unwrap_or_default(),
                                duration: item.duration.clone().unwrap_or_default(),
                                quantity: item.quantity.clone().unwrap_or_default(),
                                notes: item.notes.clone().unwrap_or_default(),
                            })
                            .collect(),
                    );
                    prescription.set(Some(found));
                }
                Err(error) => message.set(error),
            }
            if let Ok(status) = api::fetch_rx_block_status().await {
                block.set(Some(status));
            }
            initialized.set(true);
        });
    });

    if !checked() {
        return rsx! {
            div { id: "prescriptions-page", p { "Cargando..." } }
        };
    }
    let Some(profile) = actor() else {
        return rsx! {
            div { id: "prescriptions-page", p { "Redirigiendo..." } }
        };
    };
    let Some(current) = prescription() else {
        return rsx! {
            div {
                id: "prescriptions-page",
                if message().is_empty() && !initialized() {
                    p { "Cargando..." }
                } else {
                    p { class: "error", "{message}" }
                    Link { to: Route::Prescriptions {}, "Volver a recetas" }
                }
            }
        };
    };
    let is_draft = current.status == "draft";
    let can_edit = is_draft && profile.can_create_prescriptions();
    let can_sign = is_draft
        && profile.can_sign_prescriptions()
        && current.doctor_id.as_ref().is_none_or(|owner| {
            profile
                .doctor
                .as_ref()
                .is_some_and(|doctor| &doctor.id == owner)
        });
    let can_transition = !is_draft
        && current.status == "signed"
        && profile.can_sign_prescriptions()
        && current.doctor_id.as_ref().is_some_and(|owner| {
            profile
                .doctor
                .as_ref()
                .is_some_and(|doctor| &doctor.id == owner)
        });
    let block_state = block();
    let llm_template = profile
        .clinic
        .as_ref()
        .and_then(|clinic| clinic.llm_url_template.clone());

    rsx! {
        div {
            id: "prescriptions-page",
            div {
                class: "page-head",
                div {
                    h4 { "{current.patient_name}" }
                    p {
                        "{prescription_status_label(&current.status)}"
                        if let Some(number) = current.rx_number {
                            " · No. {number}"
                        }
                        " · {format_date(&current.created_at)}"
                    }
                }
                div {
                    class: "actions",
                    Link {
                        class: "button secondary",
                        to: Route::PrescriptionPreview { id: current.id.clone() },
                        "Vista previa A4"
                    }
                    Link { class: "button secondary", to: Route::Prescriptions {}, "Volver" }
                }
            }
            div {
                class: "block-indicator",
                if let Some(status) = block_state.as_ref() {
                    if status.active {
                        span { "Números disponibles: {status.remaining} de {status.limit}" }
                    } else {
                        span { "Sin bloque de números activo." }
                        button {
                            class: "secondary",
                            onclick: move |_| {
                                spawn(async move {
                                    match api::allocate_rx_block().await {
                                        Ok(()) => {
                                            if let Ok(status) = api::fetch_rx_block_status().await {
                                                block.set(Some(status));
                                            }
                                        }
                                        Err(error) => message.set(error),
                                    }
                                });
                            },
                            "Reservar bloque"
                        }
                    }
                    if status.exhausted {
                        p { class: "warning", "Bloque agotado. Reserva otro antes de firmar." }
                    }
                }
            }
            if !message().is_empty() {
                p { class: "error", "{message}" }
            }
            if can_edit {
                div {
                    class: "form-grid",
                    label {
                        class: "field",
                        span { "Diagnóstico" }
                        textarea {
                            value: "{draft_diagnosis}",
                            oninput: move |event| draft_diagnosis.set(event.value()),
                        }
                    }
                    label {
                        class: "field",
                        span { "Vigencia" }
                        input {
                            value: "{draft_validity}",
                            oninput: move |event| draft_validity.set(event.value()),
                        }
                    }
                    label {
                        class: "field wide",
                        span { "Indicaciones generales" }
                        textarea {
                            value: "{draft_instructions}",
                            oninput: move |event| draft_instructions.set(event.value()),
                        }
                    }
                }
                div {
                    class: "actions",
                    button {
                        disabled: saving(),
                        onclick: move |_| {
                            if saving() {
                                return;
                            }
                            saving.set(true);
                            let id = prescription_id();
                            let diagnosis = draft_diagnosis();
                            let validity = draft_validity();
                            let instructions = draft_instructions();
                            spawn(async move {
                                match api::update_prescription(
                                    &id,
                                    &diagnosis,
                                    &validity,
                                    &instructions,
                                )
                                .await
                                {
                                    Ok(updated) => {
                                        saving.set(false);
                                        prescription.set(Some(updated));
                                    }
                                    Err(error) => {
                                        saving.set(false);
                                        message.set(error);
                                    }
                                }
                            });
                        },
                        if saving() { "Guardando..." } else { "Guardar borrador" }
                    }
                }
            } else {
                div {
                    class: "detail-grid",
                    div { class: "field", span { "Diagnóstico" }, span { "{current.diagnosis.clone().unwrap_or_else(|| \"—\".to_string())}" } }
                    div { class: "field", span { "Vigencia" }, span { "{current.validity.clone().unwrap_or_else(|| \"—\".to_string())}" } }
                    div { class: "field wide", span { "Indicaciones" }, span { class: "notes", "{current.instructions.clone().unwrap_or_else(|| \"—\".to_string())}" } }
                    if let Some(verification) = current.verification.as_ref() {
                        div { class: "field", span { "Código de verificación" }, span { "{verification.code}" } }
                        div { class: "field", span { "Estado del código" }, span { "{verification.status}" } }
                    }
                    if let Some(reason) = current.void_reason.as_ref() {
                        div { class: "field wide", span { "Motivo de anulación" }, span { "{reason}" } }
                    }
                }
            }
            div {
                class: "medication-list",
                h4 { "Medicamentos" }
                if items().is_empty() {
                    p { class: "hint", "Sin medicamentos." }
                }
                for (index, item) in items().into_iter().enumerate() {
                    div {
                        key: "{index}",
                        class: "item-card",
                        if can_edit {
                            div {
                                class: "form-grid",
                                label {
                                    class: "field",
                                    span { "Medicamento" }
                                    input {
                                        value: "{item.med_name}",
                                        oninput: move |event| {
                                            let value = event.value();
                                            items.with_mut(|list| {
                                                if let Some(entry) = list.get_mut(index) {
                                                    entry.med_name = value;
                                                }
                                            });
                                        },
                                    }
                                }
                                label {
                                    class: "field",
                                    span { "Dosis" }
                                    input {
                                        value: "{item.dose}",
                                        oninput: move |event| {
                                            let value = event.value();
                                            items.with_mut(|list| {
                                                if let Some(entry) = list.get_mut(index) {
                                                    entry.dose = value;
                                                }
                                            });
                                        },
                                    }
                                }
                                label {
                                    class: "field",
                                    span { "Vía" }
                                    input {
                                        value: "{item.route}",
                                        oninput: move |event| {
                                            let value = event.value();
                                            items.with_mut(|list| {
                                                if let Some(entry) = list.get_mut(index) {
                                                    entry.route = value;
                                                }
                                            });
                                        },
                                    }
                                }
                                label {
                                    class: "field",
                                    span { "Frecuencia" }
                                    input {
                                        value: "{item.frequency}",
                                        oninput: move |event| {
                                            let value = event.value();
                                            items.with_mut(|list| {
                                                if let Some(entry) = list.get_mut(index) {
                                                    entry.frequency = value;
                                                }
                                            });
                                        },
                                    }
                                }
                                label {
                                    class: "field",
                                    span { "Duración" }
                                    input {
                                        value: "{item.duration}",
                                        oninput: move |event| {
                                            let value = event.value();
                                            items.with_mut(|list| {
                                                if let Some(entry) = list.get_mut(index) {
                                                    entry.duration = value;
                                                }
                                            });
                                        },
                                    }
                                }
                                label {
                                    class: "field",
                                    span { "Cantidad" }
                                    input {
                                        value: "{item.quantity}",
                                        oninput: move |event| {
                                            let value = event.value();
                                            items.with_mut(|list| {
                                                if let Some(entry) = list.get_mut(index) {
                                                    entry.quantity = value;
                                                }
                                            });
                                        },
                                    }
                                }
                                label {
                                    class: "field wide",
                                    span { "Notas" }
                                    input {
                                        value: "{item.notes}",
                                        oninput: move |event| {
                                            let value = event.value();
                                            items.with_mut(|list| {
                                                if let Some(entry) = list.get_mut(index) {
                                                    entry.notes = value;
                                                }
                                            });
                                        },
                                    }
                                }
                            }
                            div {
                                class: "actions",
                                button {
                                    onclick: {
                                        let id = prescription_id();
                                        let form = item.clone();
                                        move |_| {
                                            let id = id.clone();
                                            let form = form.clone();
                                            spawn(async move {
                                                match api::upsert_prescription_item(&id, &form).await {
                                                    Ok(_) => reload_prescription(
                                                        prescription,
                                                        items,
                                                        message,
                                                        id,
                                                    ),
                                                    Err(error) => message.set(error),
                                                }
                                            });
                                        }
                                    },
                                    "Guardar medicamento"
                                }
                                button {
                                    class: "secondary",
                                    disabled: index == 0,
                                    onclick: {
                                        let id = prescription_id();
                                        move |_| {
                                            let mut order = items()
                                                .iter()
                                                .filter_map(|entry| entry.id.clone())
                                                .collect::<Vec<_>>();
                                            if index == 0 || order.len() <= index {
                                                return;
                                            }
                                            order.swap(index, index - 1);
                                            let id = id.clone();
                                            spawn(async move {
                                                match api::reorder_prescription_items(&id, &order).await {
                                                    Ok(_) => reload_prescription(
                                                        prescription,
                                                        items,
                                                        message,
                                                        id,
                                                    ),
                                                    Err(error) => message.set(error),
                                                }
                                            });
                                        }
                                    },
                                    "Subir"
                                }
                                button {
                                    class: "secondary",
                                    disabled: index + 1 >= items().len(),
                                    onclick: {
                                        let id = prescription_id();
                                        move |_| {
                                            let mut order = items()
                                                .iter()
                                                .filter_map(|entry| entry.id.clone())
                                                .collect::<Vec<_>>();
                                            if index + 1 >= order.len() {
                                                return;
                                            }
                                            order.swap(index, index + 1);
                                            let id = id.clone();
                                            spawn(async move {
                                                match api::reorder_prescription_items(&id, &order).await {
                                                    Ok(_) => reload_prescription(
                                                        prescription,
                                                        items,
                                                        message,
                                                        id,
                                                    ),
                                                    Err(error) => message.set(error),
                                                }
                                            });
                                        }
                                    },
                                    "Bajar"
                                }
                                button {
                                    class: "danger",
                                    onclick: {
                                        let id = prescription_id();
                                        let item_id = item.id.clone().unwrap_or_default();
                                        move |_| {
                                            if item_id.is_empty() {
                                                return;
                                            }
                                            let id = id.clone();
                                            let item_id = item_id.clone();
                                            spawn(async move {
                                                match api::delete_prescription_item(&id, &item_id).await {
                                                    Ok(()) => reload_prescription(
                                                        prescription,
                                                        items,
                                                        message,
                                                        id,
                                                    ),
                                                    Err(error) => message.set(error),
                                                }
                                            });
                                        }
                                    },
                                    "Eliminar"
                                }
                            }
                        } else {
                            div {
                                class: "item-read",
                                p { class: "patient-name", "{item.med_name}" }
                                p { class: "patient-meta",
                                    "{item.dose} · {item.route} · {item.frequency} · {item.duration}"
                                }
                                if !item.notes.is_empty() {
                                    p { class: "patient-meta", "{item.notes}" }
                                }
                            }
                        }
                        if let Some(url) = llm_explain_url(llm_template.as_deref(), &item.med_name) {
                            div {
                                class: "llm-link",
                                a {
                                    href: "{url}",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    "Explicar con LLM"
                                }
                                p { class: "hint", "Se abre un sitio externo con el nombre del medicamento. No se envían datos del paciente." }
                            }
                        }
                    }
                }
                if can_edit {
                    div {
                        class: "actions",
                        button {
                            class: "secondary",
                            onclick: move |_| {
                                items.with_mut(|list| list.push(new_prescription_item_form()));
                            },
                            "Agregar medicamento"
                        }
                    }
                }
            }
            if can_sign {
                div {
                    class: "actions",
                    button {
                        disabled: signing(),
                        onclick: move |_| {
                            if signing() {
                                return;
                            }
                            signing.set(true);
                            needs_ack.set(false);
                            let id = prescription_id();
                            spawn(async move {
                                match api::sign_prescription(&id, false).await {
                                    Ok(signed) => {
                                        signing.set(false);
                                        prescription.set(Some(signed));
                                    }
                                    Err(error) => {
                                        signing.set(false);
                                        if error.contains("variables desconocidas") {
                                            needs_ack.set(true);
                                        }
                                        message.set(error);
                                    }
                                }
                            });
                        },
                        if signing() { "Firmando..." } else { "Firmar receta" }
                    }
                    if needs_ack() {
                        button {
                            class: "secondary",
                            onclick: move |_| {
                                let id = prescription_id();
                                spawn(async move {
                                    match api::sign_prescription(&id, true).await {
                                        Ok(signed) => {
                                            needs_ack.set(false);
                                            prescription.set(Some(signed));
                                        }
                                        Err(error) => message.set(error),
                                    }
                                });
                            },
                            "Firmar reconociendo las advertencias"
                        }
                    }
                }
                if block_state.as_ref().is_some_and(|status| status.exhausted) {
                    p { class: "warning", "Sin números disponibles: reserva un bloque para firmar." }
                }
            }
            if can_transition {
                div {
                    class: "actions",
                    button {
                        class: "secondary",
                        onclick: move |_| {
                            let id = prescription_id();
                            spawn(async move {
                                match api::supersede_prescription(&id).await {
                                    Ok(corrected) => {
                                        navigator.push(Route::PrescriptionDetail {
                                            id: corrected.id,
                                        });
                                    }
                                    Err(error) => message.set(error),
                                }
                            });
                        },
                        "Reemplazar (corrección)"
                    }
                    button {
                        class: "danger",
                        onclick: move |_| show_void.set(true),
                        "Anular"
                    }
                }
                if show_void() {
                    div {
                        class: "notice",
                        label {
                            class: "field",
                            span { "Motivo de anulación" }
                            input {
                                value: "{void_reason}",
                                oninput: move |event| void_reason.set(event.value()),
                            }
                        }
                        div {
                            class: "actions",
                            button {
                                class: "danger",
                                onclick: move |_| {
                                    let id = prescription_id();
                                    let reason = void_reason();
                                    spawn(async move {
                                        match api::void_prescription(&id, &reason).await {
                                            Ok(voided) => {
                                                show_void.set(false);
                                                prescription.set(Some(voided));
                                            }
                                            Err(error) => message.set(error),
                                        }
                                    });
                                },
                                "Confirmar anulación"
                            }
                            button {
                                class: "secondary",
                                onclick: move |_| show_void.set(false),
                                "Cancelar"
                            }
                        }
                    }
                }
            }
        }
    }
}
