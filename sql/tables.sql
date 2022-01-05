DROP TABLE IF EXISTS project_categories CASCADE;
CREATE TABLE project_categories (
    id SMALLINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name VARCHAR(30) NOT NULL,
    "order" SMALLINT NOT NULL DEFAULT 0
);

DROP TABLE IF EXISTS projects CASCADE;
CREATE TABLE projects (
    id SMALLINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name VARCHAR(120) NOT NULL,
    description VARCHAR(320),
    content VARCHAR(1000) NOT NULL, -- autoriser gras, lien, taille titre, liste à puce
    date TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_update_date TIMESTAMP WITH TIME ZONE
);

-- Table d'association d'un projet à une ou plusieurs catégories
DROP TABLE IF EXISTS projects_categories CASCADE;
CREATE TABLE projects_categories (
    project_id SMALLINT
        REFERENCES projects (id)
        ON DELETE CASCADE,
    category_id SMALLINT
        REFERENCES project_categories (id)
        ON DELETE CASCADE,
    PRIMARY KEY (project_id, category_id)
);

DROP TABLE IF EXISTS files CASCADE;
CREATE TABLE files (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name VARCHAR(120),
    path VARCHAR(255) NOT NULL
);

-- TODO : implement a trigger when delete to recalculate order
DROP TABLE IF EXISTS project_assets CASCADE;
CREATE TABLE project_assets (
    id SMALLINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    project_id SMALLINT NOT NULL
        REFERENCES projects (id)
        ON DELETE CASCADE,
    file_id INT NOT NULL
        REFERENCES files (id),
    "order" SMALLINT NOT NULL
    -- UNIQUE (project_id, "order")
);

DROP TABLE IF EXISTS pages CASCADE;
CREATE TABLE pages (
    id SMALLINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    title VARCHAR(120) NOT NULL,
    identifier VARCHAR(120) UNIQUE NOT NULL,
    description VARCHAR(320)
);

DROP TABLE IF EXISTS metric_sessions CASCADE;
CREATE TABLE metric_sessions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    ip VARCHAR(120),
    expiration_date TIMESTAMP WITH TIME ZONE DEFAULT NOW() + interval '30 minutes' NOT NULL
);

DROP TABLE IF EXISTS metrics CASCADE;
CREATE TABLE metrics (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id uuid REFERENCES metric_sessions (id) ON DELETE SET NULL,
    page_id SMALLINT
        REFERENCES pages (id)
        ON DELETE CASCADE,
    project_id SMALLINT
        REFERENCES projects (id)
        ON DELETE SET NULL,
    article_id SMALLINT
        REFERENCES blog_articles (id)
        ON DELETE SET NULL,
    ip VARCHAR(120),
    browser VARCHAR(20),
    os VARCHAR(20),
    device_type VARCHAR(20),
    referer VARCHAR(255),
    "date" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    end_date TIMESTAMP WITH TIME ZONE
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
    logo VARCHAR(255) NOT NULL,
    dark_mode_active BOOLEAN DEFAULT FALSE,
    background_color VARCHAR(6),
    text_color VARCHAR(6),
    title_color VARCHAR(6)
);

DROP TABLE IF EXISTS "user" CASCADE;
CREATE TABLE "user" (
    email VARCHAR(255) NOT NULL,
    username VARCHAR(60) NOT NULL,
    password VARCHAR(255) NOT NULL,
    token VARCHAR(60),
    token_validity_date TIMESTAMP WITH TIME ZONE
);

DROP TABLE IF EXISTS attempts CASCADE;
CREATE TABLE attempts (
    id SMALLINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    email VARCHAR(250) NOT NULL,
    ip VARCHAR(60) NOT NULL,
    is_login BOOLEAN NOT NULL,
    "date" tIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

DROP TABLE IF EXISTS blog_categories CASCADE;
CREATE TABLE blog_categories (
    id SMALLINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name VARCHAR(60) NOT NULL,
    uri VARCHAR(70),
    description VARCHAR(255),
    is_visible BOOLEAN,
    is_seo BOOLEAN,
    "order" SMALLINT NOT NULL
);

DROP TABLE IF EXISTS blog_articles CASCADE;
CREATE TABLE blog_articles (
    id SMALLINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    category_id SMALLINT
        REFERENCES blog_categories (id)
        ON DELETE SET NULL,
    cover_id INT NOT NULL
        REFERENCES files (id)
        ON DELETE SET NULL,
    title VARCHAR(255) NOT NULL,
    description VARCHAR(320),
    content TEXT NOT NULL,
    uri VARCHAR(260),
    "date" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    modified_date TIMESTAMP WITH TIME ZONE,
    is_published BOOLEAN DEFAULT FALSE,
    is_seo BOOLEAN DEFAULT FALSE
);

DROP TABLE IF EXISTS blog_article_images CASCADE;
CREATE TABLE blog_article_images (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    article_id SMALLINT NOT NULL
        REFERENCES blog_articles (id)
        ON DELETE CASCADE,
    file_id INT NOT NULL
        REFERENCES files (id)
);

DROP TABLE IF EXISTS settings CASCADE;
CREATE TABLE settings (
    background_color CHAR(7) NOT NULL,
    title_color CHAR(7) NOT NULL,
    text_color CHAR(7) NOT NULL
);