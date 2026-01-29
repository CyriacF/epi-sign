-- Fix migration: Drop edsquare_cookies table if it exists without primary key
-- This must run before the create_edsquare_cookies migration
-- Diesel reads the schema before running migrations, so we need to clean up first

DROP TABLE IF EXISTS "edsquare_cookies" CASCADE;
