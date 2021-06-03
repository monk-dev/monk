-- Deploy monk:articles_tags to pg
-- requires: articles
-- requires: tags

BEGIN;

CREATE TABLE public.article_tag (
    article_id      uuid                    NOT NULL,
    tag_id          uuid                    NOT NULL,
    added_at        timestamp               NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY     (article_id, tag_id),
    FOREIGN KEY     (article_id)            REFERENCES articles(id) ON UPDATE CASCADE,
    FOREIGN KEY     (tag_id)                REFERENCES tags(id) ON UPDATE CASCADE
);

COMMIT;
