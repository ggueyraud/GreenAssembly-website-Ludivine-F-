INSERT INTO pages (title, identifier, description) VALUES 
('Accueil', '/', NULL),
('Portfolio', '/portfolio', NULL),
('Motion design', '/motion-design', NULL),
('Mes petits plus', '/mes-petits-plus', NULL),
('Contact', '/contact', NULL),
('Blog', '/blog', NULL),
('Mentions légales', '/mentions-legales', NULL);

INSERT INTO page_chunks (page_id, identifier, content) VALUES
(1, 'profile_picture', '{"path": ""}'),
(3, 'link', '{"link": "https://player.vimeo.com/video/422216336?h=88fda9d15b&title=0&byline=0&portrait=0"}'),
(4, 'link_creations', '{"value": "https://greenassembly.fr/"}'),
(4, 'link_shootings', '{"value": "https://hunimalis.com/"}');

INSERT INTO "user" (username, email, password) VALUES
('Ludivine', 'hello@ludivinefarat.fr', '$argon2id$v=19$m=4096,t=3,p=1$C0s1htjlrBsEMghdgrUUPA$bK3CG5m4O5SSCcSuPEdYykO1UOdtICcYZGkzOQOyRv4');

INSERT INTO blog_categories (name, uri, description, is_visible, is_seo, "order") VALUES
('Print', 'print-1', 'Lorem ipsum dolor sit amet', true, true, 1),
('Motion design', 'motion-design-2', null, false, false, 2);

INSERT INTO project_categories (name, "order") VALUES
('Lorem', 1),
('Ipsum', 2),
('Dolor', 3);

INSERT INTO files (name, path) VALUES
('first file', 'path');

INSERT INTO blog_articles (category_id, cover_id, title, uri, is_published, is_seo) VALUES
(1, 1, 'Les aventures de lulu', 'les-aventures-de-lulu', false, true),
(1, 1, 'Les aventures de lulu partie 2', 'les-aventures-de-lulu-partie-2', true, true),
(2, 1, 'Les aventures de lulu partie 3', 'les-aventures-de-lulu-partie-3', true, true),
(2, 1, 'Les aventures de lulu partie 3', 'les-aventures-de-lulu-partie-4', true, false);

INSERT INTO settings (background_color, title_color, text_color) VALUES
('#ffffff', '#000000', '#000000');