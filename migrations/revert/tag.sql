-- Revert monk:tags from pg

BEGIN;

DROP TABLE IF EXISTS tag CASCADE;

COMMIT;
