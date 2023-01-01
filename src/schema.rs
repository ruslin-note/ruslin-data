// @generated automatically by Diesel CLI.

diesel::table! {
    deleted_items (id) {
        id -> Integer,
        item_type -> Integer,
        item_id -> Text,
        deleted_time -> BigInt,
    }
}

diesel::table! {
    folders (id) {
        id -> Text,
        title -> Text,
        created_time -> BigInt,
        updated_time -> BigInt,
        user_created_time -> BigInt,
        user_updated_time -> BigInt,
        encryption_cipher_text -> Text,
        encryption_applied -> Bool,
        parent_id -> Nullable<Text>,
        is_shared -> Bool,
        share_id -> Text,
        master_key_id -> Text,
        icon -> Text,
    }
}

diesel::table! {
    note_tags (id) {
        id -> Text,
        note_id -> Text,
        tag_id -> Text,
        created_time -> BigInt,
        updated_time -> BigInt,
        user_created_time -> BigInt,
        user_updated_time -> BigInt,
        encryption_cipher_text -> Text,
        encryption_applied -> Bool,
        is_shared -> Bool,
    }
}

diesel::table! {
    notes (id) {
        id -> Text,
        parent_id -> Nullable<Text>,
        title -> Text,
        body -> Text,
        created_time -> BigInt,
        updated_time -> BigInt,
        is_conflict -> Bool,
        latitude -> Double,
        longitude -> Double,
        altitude -> Double,
        author -> Text,
        source_url -> Text,
        is_todo -> Bool,
        todo_due -> Bool,
        todo_completed -> Bool,
        source -> Text,
        source_application -> Text,
        application_data -> Text,
        order -> BigInt,
        user_created_time -> BigInt,
        user_updated_time -> BigInt,
        encryption_cipher_text -> Text,
        encryption_applied -> Bool,
        markup_language -> Bool,
        is_shared -> Bool,
        share_id -> Text,
        conflict_original_id -> Nullable<Text>,
        master_key_id -> Text,
    }
}

diesel::table! {
    resources (id) {
        id -> Text,
        title -> Text,
        mime -> Text,
        filename -> Text,
        created_time -> BigInt,
        updated_time -> BigInt,
        user_created_time -> BigInt,
        user_updated_time -> BigInt,
        file_extension -> Text,
        encryption_cipher_text -> Text,
        encryption_applied -> Bool,
        encryption_blob_encrypted -> Bool,
        size -> Integer,
        is_shared -> Bool,
        share_id -> Text,
        master_key_id -> Text,
    }
}

diesel::table! {
    settings (key) {
        key -> Text,
        value -> Text,
    }
}

diesel::table! {
    sync_items (id) {
        id -> Integer,
        sync_target -> Integer,
        sync_time -> BigInt,
        update_time -> BigInt,
        item_type -> Integer,
        item_id -> Text,
        sync_disabled -> Bool,
        sync_disabled_reason -> Text,
        force_sync -> Bool,
        item_location -> Integer,
    }
}

diesel::table! {
    tags (id) {
        id -> Text,
        title -> Text,
        created_time -> BigInt,
        updated_time -> BigInt,
        user_created_time -> BigInt,
        user_updated_time -> BigInt,
        encryption_cipher_text -> Text,
        encryption_applied -> Bool,
        is_shared -> Bool,
        parent_id -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    deleted_items,
    folders,
    note_tags,
    notes,
    resources,
    settings,
    sync_items,
    tags,
);
