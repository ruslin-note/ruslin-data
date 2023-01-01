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
    parent_id TEXT DEFAULT NULL,
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
    todo_due BOOLEAN NOT NULL DEFAULT FALSE,
    todo_completed BOOLEAN NOT NULL DEFAULT FALSE,
    source TEXT NOT NULL DEFAULT "",
    source_application TEXT NOT NULL DEFAULT "",
    application_data TEXT NOT NULL DEFAULT "",
    `order` BIGINT NOT NULL DEFAULT 0,
    user_created_time BIGINT NOT NULL DEFAULT 0,
    user_updated_time BIGINT NOT NULL DEFAULT 0,
    encryption_cipher_text TEXT NOT NULL DEFAULT "",
    encryption_applied BOOLEAN NOT NULL DEFAULT FALSE,
    markup_language BOOLEAN NOT NULL DEFAULT TRUE,
    is_shared BOOLEAN NOT NULL DEFAULT FALSE,
    share_id TEXT NOT NULL DEFAULT "",
    conflict_original_id TEXT DEFAULT NULL,
    master_key_id TEXT NOT NULL DEFAULT ""
);

CREATE INDEX notes_title ON notes (title);
CREATE INDEX notes_parent_id ON notes (parent_id);
CREATE INDEX notes_updated_time ON notes (updated_time);
CREATE INDEX notes_is_conflict ON notes (is_conflict);
CREATE INDEX notes_is_todo ON notes (is_todo);
CREATE INDEX notes_order ON notes (`order`);

---

CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(content='notes', content_rowid=rowid, title, body, id UNINDEXED, tokenize="jieba");

CREATE TRIGGER notes_after_insert AFTER INSERT ON notes BEGIN
    INSERT INTO notes_fts(rowid, title, body, id) VALUES (new.rowid, new.title, new.body, new.id);
END;
CREATE TRIGGER notes_after_delete AFTER DELETE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, title, body, id) VALUES ('delete', old.rowid, old.title, old.body, old.id);
END;
CREATE TRIGGER notes_after_update AFTER UPDATE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, title, body, id) VALUES ('delete', old.rowid, old.title, old.body, old.id);
    INSERT INTO notes_fts(rowid, title, body, id) VALUES (new.rowid, new.title, new.body, new.id);
END;

---

CREATE TABLE sync_items (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    sync_target INT NOT NULL,
    sync_time BIGINT NOT NULL DEFAULT 0,
    update_time BIGINT NOT NULL DEFAULT 0,
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

---

CREATE TABLE deleted_items (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    item_type INT NOT NULL,
    item_id TEXT NOT NULL,
    deleted_time BIGINT NOT NULL
);

CREATE INDEX deleted_items_item_id ON deleted_items (item_id);

---

CREATE TABLE settings(
    `key` TEXT NOT NULL PRIMARY KEY,
    `value` TEXT NOT NULL
);

---

CREATE TABLE tags (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    created_time BIGINT NOT NULL,
    updated_time BIGINT NOT NULL,
    user_created_time BIGINT NOT NULL DEFAULT 0,
    user_updated_time BIGINT NOT NULL DEFAULT 0,
    encryption_cipher_text TEXT NOT NULL DEFAULT "",
    encryption_applied BOOLEAN NOT NULL DEFAULT FALSE,
    is_shared BOOLEAN NOT NULL DEFAULT FALSE,
    parent_id TEXT DEFAULT NULL
);

CREATE TABLE note_tags (
    id TEXT PRIMARY KEY NOT NULL,
    note_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    created_time BIGINT NOT NULL,
    updated_time BIGINT NOT NULL,
    user_created_time BIGINT NOT NULL DEFAULT 0,
    user_updated_time BIGINT NOT NULL DEFAULT 0,
    encryption_cipher_text TEXT NOT NULL DEFAULT "",
    encryption_applied BOOLEAN NOT NULL DEFAULT FALSE,
    is_shared BOOLEAN NOT NULL DEFAULT FALSE
);

---

CREATE TABLE `resources`(
    `id` TEXT PRIMARY KEY NOT NULL,
    `title` TEXT NOT NULL DEFAULT "",
    `mime` TEXT NOT NULL,
    `filename` TEXT NOT NULL DEFAULT "",
    `created_time` BIGINT NOT NULL,
    `updated_time` BIGINT NOT NULL,
    `user_created_time` BIGINT NOT NULL DEFAULT 0,
    `user_updated_time` BIGINT NOT NULL DEFAULT 0,
    `file_extension` TEXT NOT NULL DEFAULT "",
    `encryption_cipher_text` TEXT NOT NULL DEFAULT "",
    `encryption_applied` BOOLEAN NOT NULL DEFAULT FALSE,
    `encryption_blob_encrypted` BOOLEAN NOT NULL DEFAULT FALSE,
    `size` INT NOT NULL DEFAULT -1,
    is_shared BOOLEAN NOT NULL DEFAULT FALSE,
    share_id TEXT NOT NULL DEFAULT "",
    master_key_id TEXT NOT NULL DEFAULT ""
);
