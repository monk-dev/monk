CREATE TABLE IF NOT EXISTS user (
    id              UUID    PRIMARY KEY DEFAULT (uuid()),
    name            STRING  NOT NULL,
    created_at      INT     NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS article (
    id              UUID    PRIMARY KEY DEFAULT (uuid()),
    user_id         UUID    REFERENCES user(id),
    name            STRING  NOT NULL,
    description     STRING,
    url             URL,
    created_at      INT     NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS article_tag (
    id              UUID    PRIMARY KEY DEFAULT (uuid()),
    article_id      UUID    REFERENCES article(id) ON UPDATE CASCADE,
    tag_id          UUID    REFERENCES tag(id) ON UPDATE CASCADE,
    created_at      INT     NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS tag (
    id              UUID    PRIMARY KEY DEFAULT (uuid()),
    name            STRING  NOT NULL,
    created_at      INT     NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS namespace (
    id              UUID    PRIMARY KEY DEFAULT (uuid()),
    user_id         UUID    NOT NULL REFERENCES user(id),
    name            STRING  NOT NULL,
    created_at      INT     NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS namespace_article (
    id              UUID    PRIMARY KEY DEFAULT (uuid()),
    namespace_id    UUID    NOT NULL REFERENCES namespace(id),
    article_id      UUID    NOT NULL REFERENCES article(id),
    created_at      INT     NOT NULL DEFAULT CURRENT_TIMESTAMP
);