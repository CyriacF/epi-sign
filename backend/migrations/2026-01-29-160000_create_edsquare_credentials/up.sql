CREATE TABLE IF NOT EXISTS edsquare_credentials (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    email TEXT NOT NULL,
    password TEXT NOT NULL
);

