-- Revert monk:articles from pg

BEGIN;

DROP TABLE IF EXISTS article CASCADE;

COMMIT;
