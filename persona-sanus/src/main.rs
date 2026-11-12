mod api;

#[cfg(feature = "server")]
mod api_server;
mod auth;

#[cfg(feature = "server")]
mod auth_server;
#[cfg(feature = "server")]
mod db;
mod http;
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
                Some(_) => api::fetch_me().await.ok(),
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
                        Some(_) => api::fetch_me().await.ok(),
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
                Link { to: Route::SettingsProfile {}, "Mi perfil" }
            }
        },
        _ => rsx! {
            div { id: "echo", p { "Redirigiendo..." } }
        },
    }
}

#[component]
fn SettingsUsers() -> Element {
    let actor = use_context::<Signal<Option<Me>>>();
    let checked = use_context::<Signal<bool>>();
    let navigator = use_navigator();
    let mut users = use_signal(Vec::<api::UserRow>::new);
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

    rsx! {
        div {
            id: "echo",
            h4 { "Usuarios y roles" }
            p { "Asigna un rol o deshabilita el acceso de cada persona." }
            if !message().is_empty() {
                p { "{message}" }
            }
            for row in users() {
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
                button { disabled: true, "Nueva receta" }
                p { class: "hint", "La nueva receta se habilita con las plantillas de recetas." }
            }
            p { class: "hint", "El historial de recetas estará disponible próximamente." }
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

fn format_date(value: &str) -> String {
    let date = value.get(..10).unwrap_or(value);
    match chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(parsed) => parsed.format("%d-%m-%Y").to_string(),
        Err(_) => value.to_owned(),
    }
}
