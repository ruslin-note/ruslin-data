// @generated automatically by Diesel CLI.

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
        is_shared -> Nullable<Integer>,
        share_id -> Text,
        master_key_id -> Text,
        icon -> Text,
    }
}

diesel::table! {
    notes (id) {
        id -> Text,
        parent_id -> Text,
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
        todo_due -> Integer,
        todo_completed -> Integer,
        source -> Text,
        source_application -> Text,
        application_data -> Text,
        custom_order -> Double,
        user_created_time -> BigInt,
        user_updated_time -> BigInt,
        encryption_cipher_text -> Text,
        encryption_applied -> Integer,
        markup_language -> Bool,
        is_shared -> Bool,
        share_id -> Text,
        conflict_original_id -> Nullable<Text>,
        master_key_id -> Text,
    }
}

diesel::table! {
    sync_items (id) {
        id -> Integer,
        sync_target -> Integer,
        sync_time -> BigInt,
        item_type -> Integer,
        item_id -> Text,
        sync_disabled -> Bool,
        sync_disabled_reason -> Text,
        force_sync -> Bool,
        item_location -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(folders, notes, sync_items,);
