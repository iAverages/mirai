CREATE TABLE seen_wallpapers (
    id TEXT PRIMARY KEY,
    seen BOOLEAN,
    manager_id INTEGER
);

CREATE TABLE meta (
    id INTEGER PRIMARY KEY,
    last_update datetime,
    last_used TEXT
);
