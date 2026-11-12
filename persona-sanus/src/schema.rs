diesel::table! {
    users (id) {
        id -> Text,
        email -> Text,
        password_hash -> Text,
        display_name -> Text,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    sessions (id) {
        id -> Text,
        token_hash -> Text,
        user_id -> Text,
        expires_at -> Text,
        created_at -> Text,
    }
}

diesel::table! {
    profiles (id) {
        id -> Text,
        auth_user_id -> Text,
        display_name -> Text,
        status -> Text,
        created_at -> Text,
        updated_at -> Text,
        created_by -> Nullable<Text>,
    }
}

diesel::table! {
    clinics (id) {
        id -> Text,
        name -> Text,
        address -> Nullable<Text>,
        phone -> Nullable<Text>,
        logo_asset_id -> Nullable<Text>,
        timezone -> Text,
        llm_url_template -> Nullable<Text>,
        retention_days -> Integer,
        rx_block_size -> Integer,
        server_retention_until -> Text,
        data_grace_ends_at -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
        created_by -> Nullable<Text>,
    }
}

diesel::table! {
    assets (id) {
        id -> Text,
        clinic_id -> Text,
        object_key -> Text,
        owner_type -> Text,
        owner_id -> Text,
        kind -> Text,
        original_name -> Nullable<Text>,
        content_type -> Text,
        size_bytes -> Integer,
        sha256 -> Text,
        width -> Nullable<Integer>,
        height -> Nullable<Integer>,
        status -> Text,
        deleted_at -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
        created_by -> Nullable<Text>,
    }
}

diesel::table! {
    clinic_members (clinic_id, profile_id) {
        clinic_id -> Text,
        profile_id -> Text,
        role -> Text,
        created_at -> Text,
        updated_at -> Text,
        created_by -> Nullable<Text>,
    }
}

diesel::table! {
    devices (id) {
        id -> Text,
        profile_id -> Text,
        name -> Nullable<Text>,
        platform -> Text,
        app_version -> Nullable<Text>,
        last_seen_at -> Nullable<Text>,
        revoked_at -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
        created_by -> Nullable<Text>,
    }
}

diesel::table! {
    doctors (id) {
        id -> Text,
        profile_id -> Text,
        specialty -> Nullable<Text>,
        license_no -> Nullable<Text>,
        national_id -> Nullable<Text>,
        phone -> Nullable<Text>,
        email -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
        created_by -> Nullable<Text>,
    }
}

diesel::table! {
    patients (id) {
        id -> Text,
        clinic_id -> Text,
        full_name -> Text,
        national_id -> Nullable<Text>,
        birth_date -> Nullable<Text>,
        sex -> Nullable<Text>,
        phone -> Nullable<Text>,
        notes -> Nullable<Text>,
        client_id -> Nullable<Text>,
        deleted_at -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
        created_by -> Nullable<Text>,
    }
}

diesel::table! {
    templates (id) {
        id -> Text,
        owner_type -> Text,
        owner_id -> Text,
        name -> Text,
        current_version_id -> Nullable<Text>,
        status -> Text,
        client_id -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
        created_by -> Nullable<Text>,
    }
}

diesel::table! {
    template_versions (id) {
        id -> Text,
        template_id -> Text,
        version_no -> Integer,
        doc_json -> Text,
        variables_spec -> Text,
        theme -> Nullable<Text>,
        status -> Text,
        created_at -> Text,
        updated_at -> Text,
        created_by -> Nullable<Text>,
    }
}

diesel::table! {
    template_version_assets (template_version_id, asset_id) {
        template_version_id -> Text,
        asset_id -> Text,
    }
}

diesel::table! {
    rx_counters (clinic_id) {
        clinic_id -> Text,
        last_rx_number -> Integer,
    }
}

diesel::table! {
    rx_blocks (id) {
        id -> Text,
        clinic_id -> Text,
        device_id -> Text,
        start_number -> Integer,
        end_number -> Integer,
        consumed -> Integer,
        status -> Text,
        allocated_at -> Text,
        created_at -> Text,
        updated_at -> Text,
        created_by -> Nullable<Text>,
    }
}

diesel::table! {
    prescriptions (id) {
        id -> Text,
        patient_id -> Text,
        doctor_id -> Nullable<Text>,
        clinic_id -> Text,
        template_version_id -> Nullable<Text>,
        status -> Text,
        rx_number -> Nullable<Integer>,
        rx_block_id -> Nullable<Text>,
        diagnosis -> Nullable<Text>,
        validity -> Nullable<Text>,
        instructions -> Nullable<Text>,
        signed_at -> Nullable<Text>,
        synced_at -> Nullable<Text>,
        voided_at -> Nullable<Text>,
        void_reason -> Nullable<Text>,
        client_id -> Text,
        supersedes_id -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
        created_by -> Nullable<Text>,
    }
}

diesel::table! {
    prescription_items (id) {
        id -> Text,
        prescription_id -> Text,
        position -> Integer,
        med_name -> Text,
        dose -> Nullable<Text>,
        route -> Nullable<Text>,
        frequency -> Nullable<Text>,
        duration -> Nullable<Text>,
        quantity -> Nullable<Text>,
        notes -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
        created_by -> Nullable<Text>,
    }
}

diesel::table! {
    verification_codes (id) {
        id -> Text,
        prescription_id -> Text,
        code -> Text,
        code_hash -> Text,
        status -> Text,
        created_at -> Text,
        updated_at -> Text,
        created_by -> Nullable<Text>,
    }
}

diesel::table! {
    audit_log (id) {
        id -> Text,
        actor_id -> Nullable<Text>,
        action -> Text,
        entity -> Text,
        entity_id -> Text,
        metadata_json -> Nullable<Text>,
        at -> Text,
    }
}

diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(clinic_members -> clinics (clinic_id));
diesel::joinable!(clinic_members -> profiles (profile_id));
diesel::joinable!(doctors -> profiles (profile_id));
diesel::joinable!(profiles -> users (auth_user_id));

diesel::allow_tables_to_appear_in_same_query!(
    users,
    sessions,
    profiles,
    clinics,
    assets,
    clinic_members,
    devices,
    doctors,
    patients,
    templates,
    template_versions,
    template_version_assets,
    rx_counters,
    rx_blocks,
    prescriptions,
    prescription_items,
    verification_codes,
    audit_log,
);
