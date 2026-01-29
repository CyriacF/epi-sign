-- Rollback: Recreate table without user_id (original structure)
-- Drop the table and recreate it without user_id

DROP TABLE IF EXISTS "edsquare_cookies";

-- Recreate table without user_id (original structure from before this migration)
CREATE TABLE "edsquare_cookies"(
	"id" TEXT NOT NULL PRIMARY KEY,
	"date" DATE NOT NULL,
	"cookie_data" JSONB NOT NULL
);
