INSERT INTO pages (title, identifier, description) VALUES 
('Accueil', 'accueil', NULL),
('Portfolio', 'portfolio', NULL),
('Motion design', 'motion-design', NULL),
('Mes petits plus', 'mes-petits-plus', NULL),
('Contact', 'contact', NULL),
('Blog', 'blog', NULL);

INSERT INTO page_chunks (page_id, identifier, content) VALUES
(1, 'profile_picture', '{"path": ""}'),
(4, 'link_creations', '{"value": ""}'),
(4, 'link_shootings', '{"value": ""}');

INSERT INTO "user" (username, email, password) VALUES
('Ludivine', 'hello@ludivinefarat.fr', '$argon2id$v=19$m=4096,t=3,p=1$Mp3DjJs9YxfrRDu0TUIbcw$+W67zS2FLYB6ruKPI1MHCB4KD+QrEJhN4D2VBzeVuOs');

INSERT INTO blog_categories (name, uri, description, is_visible, is_seo, "order") VALUES
('Print', 'print-1', 'Lorem ipsum dolor sit amet', true, true, 1),
('Motion design', 'motion-design-2', null, false, false, 2);

INSERT INTO files (name, path) VALUES
('first file', 'path');

-- We prepare the rows for the 7 images in the my_little_plus_images page
INSERT INTO my_little_plus_images (file_id) VALUES
(NULL), -- #1
(NULL), -- #2
(NULL), -- #3
(NULL), -- #4
(NULL), -- #5
(NULL), -- #6
(NULL); -- #7

INSERT INTO blog_articles (category_id, cover_id, title, uri, is_published, is_seo) VALUES
(1, 1, 'Les aventures de lulu', 'les-aventures-de-lulu', false, true),
(1, 1, 'Les aventures de lulu partie 2', 'les-aventures-de-lulu-partie-2', true, true),
(2, 1, 'Les aventures de lulu partie 3', 'les-aventures-de-lulu-partie-3', true, true),
(2, 1, 'Les aventures de lulu partie 3', 'les-aventures-de-lulu-partie-4', true, false);

INSERT INTO blog_article_blocks (article_id, title, content, left_column, "order") VALUES
(1, 'Lorem ipsum', 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam id felis feugiat, rhoncus nisi nec, consequat elit. Duis velit nibh, hendrerit id lectus id, dapibus posuere sem. Pellentesque consequat tortor id vestibulum pellentesque. Sed et erat rutrum, congue ligula at, facilisis justo. Morbi felis elit, commodo sed rhoncus et, dignissim.', true, 1),
(1, 'Dolor sit amet', 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam id felis feugiat, rhoncus nisi nec, consequat elit. Duis velit nibh, hendrerit id lectus id, dapibus posuere sem. Pellentesque consequat tortor id vestibulum pellentesque. Sed et erat rutrum, congue ligula at, facilisis justo. Morbi felis elit, commodo sed rhoncus et, dignissim.', true, 2),
(2, 'Lorem ipsum', 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam id felis feugiat, rhoncus nisi nec, consequat elit. Duis velit nibh, hendrerit id lectus id, dapibus posuere sem. Pellentesque consequat tortor id vestibulum pellentesque. Sed et erat rutrum, congue ligula at, facilisis justo. Morbi felis elit, commodo sed rhoncus et, dignissim.', true, 1),
(2, 'Lorem ipsum', 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam id felis feugiat, rhoncus nisi nec, consequat elit. Duis velit nibh, hendrerit id lectus id, dapibus posuere sem. Pellentesque consequat tortor id vestibulum pellentesque. Sed et erat rutrum, congue ligula at, facilisis justo. Morbi felis elit, commodo sed rhoncus et, dignissim.', false, 1),
(2, 'Lorem ipsum', 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam id felis feugiat, rhoncus nisi nec, consequat elit. Duis velit nibh, hendrerit id lectus id, dapibus posuere sem. Pellentesque consequat tortor id vestibulum pellentesque. Sed et erat rutrum, congue ligula at, facilisis justo. Morbi felis elit, commodo sed rhoncus et, dignissim.', true, 2);