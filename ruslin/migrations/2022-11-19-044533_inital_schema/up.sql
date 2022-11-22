CREATE TABLE folders(
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL DEFAULT "",
    created_time BIGINT NOT NULL,
    updated_time BIGINT NOT NULL,
    user_created_time BIGINT NOT NULL DEFAULT 0,
    user_updated_time BIGINT NOT NULL DEFAULT 0,
    encryption_cipher_text TEXT NOT NULL DEFAULT "",
    encryption_applied BOOLEAN NOT NULL DEFAULT FALSE,
    parent_id TEXT DEFAULT NULL,
    is_shared BOOLEAN NOT NULL DEFAULT FALSE,
    share_id TEXT NOT NULL DEFAULT "",
    master_key_id TEXT NOT NULL DEFAULT "",
    icon TEXT NOT NULL DEFAULT ""
);

CREATE INDEX folders_title ON folders (title);
CREATE INDEX folders_updated_time ON folders (updated_time);

---

CREATE TABLE notes(
    id TEXT PRIMARY KEY NOT NULL,
    parent_id TEXT NOT NULL DEFAULT "",
    title TEXT NOT NULL DEFAULT "",
    body TEXT NOT NULL DEFAULT "",
    created_time BIGINT NOT NULL,
    updated_time BIGINT NOT NULL,
    is_conflict BOOLEAN NOT NULL DEFAULT FALSE,
    latitude NUMERIC NOT NULL DEFAULT 0,
    longitude NUMERIC NOT NULL DEFAULT 0,
    altitude NUMERIC NOT NULL DEFAULT 0,
    author TEXT NOT NULL DEFAULT "",
    source_url TEXT NOT NULL DEFAULT "",
    is_todo BOOLEAN NOT NULL DEFAULT FALSE,
    todo_due INT NOT NULL DEFAULT 0,
    todo_completed INT NOT NULL DEFAULT 0,
    source TEXT NOT NULL DEFAULT "",
    source_application TEXT NOT NULL DEFAULT "",
    application_data TEXT NOT NULL DEFAULT "",
    custom_order NUMERIC NOT NULL DEFAULT 0,
    user_created_time BIGINT NOT NULL DEFAULT 0,
    user_updated_time BIGINT NOT NULL DEFAULT 0,
    encryption_cipher_text TEXT NOT NULL DEFAULT "",
    encryption_applied INT NOT NULL DEFAULT 0,
    markup_language BOOLEAN NOT NULL DEFAULT TRUE,
    is_shared BOOLEAN NOT NULL DEFAULT FALSE,
    share_id TEXT NOT NULL DEFAULT "",
    conflict_original_id TEXT DEFAULT NULL,
    master_key_id TEXT NOT NULL DEFAULT ""
);

CREATE INDEX notes_title ON notes (title);
CREATE INDEX notes_updated_time ON notes (updated_time);
CREATE INDEX notes_is_conflict ON notes (is_conflict);
CREATE INDEX notes_is_todo ON notes (is_todo);
CREATE INDEX notes_custom_order ON notes (custom_order);

---

CREATE TABLE sync_items (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    sync_target INT NOT NULL,
    sync_time BIGINT NOT NULL DEFAULT 0,
    item_type INT NOT NULL,
    item_id TEXT NOT NULL,
    sync_disabled BOOLEAN NOT NULL DEFAULT FALSE,
    sync_disabled_reason TEXT NOT NULL DEFAULT "",
    force_sync BOOLEAN NOT NULL DEFAULT FALSE,
    item_location INT NOT NULL DEFAULT 1
);

CREATE INDEX sync_items_sync_time ON sync_items (sync_time);
CREATE INDEX sync_items_sync_target ON sync_items (sync_target);
CREATE INDEX sync_items_item_type ON sync_items (item_type);
CREATE INDEX sync_items_item_id ON sync_items (item_id);
