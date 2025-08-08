-- Schema for utility functions and collations, to keep the "public" schema clean
CREATE SCHEMA kitsune;

-- Unicode collation that effectively ignores all accent and case differences
-- We use this on our username columns to achieve case insensitivity and prevent impersonation through accent characters
CREATE COLLATION kitsune.ignore_accent_case (
    provider = icu,
    deterministic = false,
    locale = 'und-u-ks-level1'
);

CREATE TABLE users (
    id UUID PRIMARY KEY,
    username TEXT NOT NULL COLLATE kitsune.ignore_accent_case,
    email TEXT NOT NULL,
    hashed_password TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE users
    ADD CONSTRAINT "uk-users-username"
        UNIQUE (username);

ALTER TABLE users
    ADD CONSTRAINT "uk-users-email"
        UNIQUE (email);

SELECT diesel_manage_updated_at('users');
