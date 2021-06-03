-- Revert monk:articles_tags from pg

BEGIN;

DROP TABLE IF EXISTS article_tag;

COMMIT;
