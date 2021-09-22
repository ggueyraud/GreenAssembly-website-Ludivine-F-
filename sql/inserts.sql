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

INSERT INTO "user" (email, password) VALUES
('contact@ludivinefarat.fr', '$argon2id$v=19$m=4096,t=3,p=1$Mp3DjJs9YxfrRDu0TUIbcw$+W67zS2FLYB6ruKPI1MHCB4KD+QrEJhN4D2VBzeVuOs');

INSERT INTO blog_categories (name, description, is_visible, is_seo, "order") VALUES
('Print', 'Lorem ipsum dolor sit amet', true, true, 1),
('Motion design', null, false, false, 2);

INSERT INTO blog_articles (category_id, cover_id, name, content, is_published) VALUES
(1, null, 'Les aventures de lulu', 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam id felis feugiat, rhoncus nisi nec, consequat elit. Duis velit nibh, hendrerit id lectus id, dapibus posuere sem. Pellentesque consequat tortor id vestibulum pellentesque. Sed et erat rutrum, congue ligula at, facilisis justo. Morbi felis elit, commodo sed rhoncus et, dignissim.', false),
(1, null, 'Les aventures de lulu partie 2', 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam id felis feugiat, rhoncus nisi nec, consequat elit. Duis velit nibh, hendrerit id lectus id, dapibus posuere sem. Pellentesque consequat tortor id vestibulum pellentesque. Sed et erat rutrum, congue ligula at, facilisis justo. Morbi felis elit, commodo sed rhoncus et, dignissim.', true),
(2, null, 'Les aventures de lulu partie 3', 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam id felis feugiat, rhoncus nisi nec, consequat elit. Duis velit nibh, hendrerit id lectus id, dapibus posuere sem. Pellentesque consequat tortor id vestibulum pebi felis elit, commodo sed rhoncus et, dignissim.', true);