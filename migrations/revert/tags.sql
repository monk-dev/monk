-- Revert monk:tags from pg

BEGIN;

DROP TABLE IF EXISTS tags;

COMMIT;
