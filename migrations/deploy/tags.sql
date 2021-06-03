-- Deploy monk:tags to pg
-- requires: db

BEGIN;

CREATE TABLE IF NOT EXISTS public.tags
(
    id                  uuid        NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    tag                 text        NOT NULL,
    created_at        timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);


COMMIT;
