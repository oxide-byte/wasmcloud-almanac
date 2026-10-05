CREATE TABLE IF NOT EXISTS users (
                                     id SERIAL PRIMARY KEY,
                                     name VARCHAR(100) NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL
    );

INSERT INTO users (name, email) VALUES
                                    ('Alice', 'alice@example.com'),
                                    ('Bob', 'bob@example.com'),
                                    ('Charlie', 'charlie@example.com')
    ON CONFLICT (email) DO NOTHING;

-- Read-only role used by PostgREST for anonymous requests
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'web_anon') THEN
        CREATE ROLE web_anon NOLOGIN;
    END IF;
END
$$;
GRANT USAGE ON SCHEMA public TO web_anon;
GRANT SELECT ON users TO web_anon;
