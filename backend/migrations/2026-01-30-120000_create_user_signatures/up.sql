CREATE TABLE IF NOT EXISTS user_signatures (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    signature_data TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_user_signatures_user_id ON user_signatures(user_id);

-- Migrer les signatures existantes (une par utilisateur ayant signature_manuscrite)
-- md5(...) pour un id unique sans extension (gen_random_uuid nécessite pgcrypto ou PG 14+)
INSERT INTO user_signatures (id, user_id, signature_data, created_at)
SELECT
    md5(id || signature_manuscrite || clock_timestamp()::text),
    id,
    signature_manuscrite,
    NOW()
FROM users
WHERE signature_manuscrite IS NOT NULL AND signature_manuscrite != '';
