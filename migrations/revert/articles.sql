-- Revert monk:articles from pg

BEGIN;

DROP TABLE IF EXISTS articles;

COMMIT;
