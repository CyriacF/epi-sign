-- Your SQL goes here
-- This migration is idempotent: it only creates the table if it doesn't exist

CREATE TABLE IF NOT EXISTS "edsquare_cookies"(
	"id" TEXT NOT NULL PRIMARY KEY,
	"user_id" TEXT NOT NULL,
	"date" DATE NOT NULL,
	"cookie_data" JSONB NOT NULL
);

-- Create index for faster lookups (only if it doesn't exist)
CREATE INDEX IF NOT EXISTS idx_edsquare_cookies_user_id_date ON edsquare_cookies(user_id, date);
