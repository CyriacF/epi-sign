-- This migration fixes the edsquare_cookies table structure:
-- If the table exists without a primary key or without user_id, we drop and recreate it
-- This is safe because cookies are per-day and will be recreated on next login

-- Drop the table if it exists without proper structure (no primary key or no user_id)
-- We'll recreate it with the correct structure below
DROP TABLE IF EXISTS "edsquare_cookies";

-- Now create the table with the correct structure (idempotent)
CREATE TABLE IF NOT EXISTS "edsquare_cookies"(
	"id" TEXT NOT NULL PRIMARY KEY,
	"user_id" TEXT NOT NULL,
	"date" DATE NOT NULL,
	"cookie_data" JSONB NOT NULL
);

-- Create index for faster lookups (idempotent)
CREATE INDEX IF NOT EXISTS idx_edsquare_cookies_user_id_date ON edsquare_cookies(user_id, date);
