DROP TABLE IF EXISTS project_categories CASCADE;
CREATE TABLE project_categories (
    id SMALLINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name VARCHAR(120) NOT NULL,
    "order" SMALLINT NOT NULL,
    UNIQUE ("order")
);

DROP TABLE IF EXISTS projects CASCADE;
CREATE TABLE projects (
    id SMALLINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    category_id SMALLINT NOT NULL
        REFERENCES project_categories (id)
        NOT NULL,
    name VARCHAR(120) NOT NULL,
    description VARCHAR(320),
    content TEXT NOT NULL, -- autoriser gras, lien, taille titre, liste à puce
    date TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_update_date TIMESTAMP WITH TIME ZONE
);

DROP TABLE IF EXISTS project_assets CASCADE;
CREATE TABLE project_assets (
    id SMALLINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    project_id SMALLINT NOT NULL
        REFERENCES projects (id)
        ON DELETE CASCADE,
    name VARCHAR(255),
    path VARCHAR(255) NOT NULL,
    "order" SMALLINT NOT NULL,
    is_visible BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE (project_id, "order")
);

DROP TABLE IF EXISTS videos CASCADE;
CREATE TABLE videos (
    id SMALLINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    title VARCHAR(120) NOT NULL,
    description VARCHAR(320),
    path VARCHAR(255) NOT NULL,
    "order" SMALLINT NOT NULL,
    UNIQUE ("order")
);
     
DROP TABLE IF EXISTS pages CASCADE;
CREATE TABLE pages (
    id SMALLINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    title VARCHAR(120) NOT NULL,
    identifier VARCHAR(120) UNIQUE NOT NULL,
    description VARCHAR(320)
);

DROP TABLE IF EXISTS metrics CASCADE;
CREATE TABLE metrics (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    page_id SMALLINT
        REFERENCES pages (id)
        ON DELETE SET NULL,
    project_id SMALLINT
        REFERENCES projects (id)
        ON DELETE SET NULL,
    ip VARCHAR(60),
    browser VARCHAR(20),
    os VARCHAR(20),
    device_type VARCHAR(20),
    referer VARCHAR(255),
    "date" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

DROP TABLE IF EXISTS page_chunks CASCADE;
CREATE TABLE page_chunks (
    id SMALLINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    page_id SMALLINT NOT NULL
        REFERENCES pages (id)
        ON DELETE CASCADE,
    identifier VARCHAR(120) NOT NULL,
    content JSONB,
    UNIQUE (page_id, identifier)
);

DROP TABLE IF EXISTS website CASCADE;
CREATE TABLE website (
    name VARCHAR(120) NOT NULL,
    logo VARCHAR(255) NOT NULL,
    dark_mode_active BOOLEAN DEFAULT FALSE,
    background_color VARCHAR(6),
    text_color VARCHAR(6),
    title_color VARCHAR(6)
);

DROP TABLE IF EXISTS "user" CASCADE;
CREATE TABLE "user" (
    email VARCHAR(255) NOT NULL,
    password VARCHAR(255) NOT NULL
);

DROP TABLE IF EXISTS login_attempts CASCADE;
CREATE TABLE login_attempts (
    id SMALLINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    email VARCHAR(250) NOT NULL,
    ip VARCHAR(60) NOT NULL,
    browser VARCHAR(20),
    os VARCHAR(20),
    device_type VARCHAR(20),
    "date" tIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);