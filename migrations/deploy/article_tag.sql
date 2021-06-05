-- Deploy monk:articles_tags to pg
-- requires: articles
-- requires: tags

BEGIN;

CREATE TABLE public.article_tag (
    article_id      uuid                    NOT NULL,
    tag_id          uuid                    NOT NULL,
    added_at        timestamp               NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY     (article_id, tag_id),
    FOREIGN KEY     (article_id)            REFERENCES article(id) ON UPDATE CASCADE,
    FOREIGN KEY     (tag_id)                REFERENCES tag(id) ON UPDATE CASCADE
);

CREATE FUNCTION public.article_tags(a public.article)
returns setof public.tag as $$
    select tag.*
    from public.tag
    inner join public.article_tag
    on (article_tag.tag_id = tag.id)
    where article_tag.article_id = a.id
$$ language sql stable;

-- comment on function article_tags(articles) is E'@omit';

COMMIT;
