CREATE TABLE IF NOT EXISTS article (
    id          UUID    PRIMARY KEY DEFAULT (uuid()),
    name        STRING  NOT NULL,
    description STRING,
    url         URL,
    created_at  INT     NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS tag (
    id          UUID    PRIMARY KEY DEFAULT (uuid()),
    name        STRING  NOT NULL,
    created_at  INT     NOT NULL DEFAULT CURRENT_TIMESTAMP
) WITHOUT rowid;

CREATE TABLE IF NOT EXISTS article_tag (
    id                      UUID PRIMARY KEY DEFAULT (uuid()),
    article_id              UUID REFERENCES article(id) ON UPDATE CASCADE,
    tag_id                  UUID REFERENCES tag(id) ON UPDATE CASCADE,
    created_at              INT  NOT NULL DEFAULT CURRENT_TIMESTAMP
) WITHOUT rowid;

-- CREATE TABLE IF NOT EXISTS namespace {
--     id          UUID    NOT NULL PRIMARY KEY,
--     name        STRING  NOT NULL,
--     created_at  INT     NOT NULL,
-- };

-- CREATE TABLE IF NOT EXISTS namespace {

-- };

-- CREATE TABLE IF NOT EXISTS user (
--     id          UUID    NOT NULL PRIMARY KEY,
--     username    STRING  NOT NULL,
--     created_at  INT     NOT NULL
-- );

-- CREATE TABLE IF NOT EXISTS user_article {
--     id          UUID NOT NULL PRIMARY KEY,
--     user_id     UUID NOT NULL,
--     article_id  UUID NOT NULL
--     created_at  INT  NOT NULL
-- }