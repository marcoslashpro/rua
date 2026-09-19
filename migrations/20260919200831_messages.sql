create table if not exists messages
(
    id UUID PRIMARY KEY NOT NULL,
    content MEDIUM_BLOB NOT NULL,
    kind VARCHAR(50) NOT NULL
)