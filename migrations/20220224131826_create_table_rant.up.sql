CREATE TABLE rants (
    guild_id BIGINT NOT NULL,
    id SERIAL NOT NULL PRIMARY KEY,
    name VARCHAR(30) NOT NULL,
    rant TEXT NOT NULL,
    author BIGINT NOT NULL,
    UNIQUE (guild_id, name)
);