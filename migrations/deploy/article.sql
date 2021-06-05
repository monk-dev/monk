-- Deploy monk:articles to pg
-- requires: db

BEGIN;

SET client_min_messages TO WARNING;

CREATE TABLE IF NOT EXISTS public.article
(
    id                  uuid        NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    name                text        NOT NULL,
    description         text,
    url                 text,
    created_at          timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

COMMIT;
