INSERT INTO pages (title, identifier, description) VALUES 
('Accueil', 'accueil', NULL),
('Portfolio', 'portfolio', NULL),
('Motion design', 'motion-design', NULL),
('Mes petits plus', 'mes-petits-plus', NULL),
('Contact', 'contact', NULL);

INSERT INTO page_chunks (page_id, identifier, content) VALUES
(1, 'profile_picture', '{"path": ""}'),
(4, 'link_creations', '{"value": ""}'),
(4, 'link_shootings', '{"value": ""}');

INSERT INTO "user" (email, password) VALUES
('contact@ludivinefarat.fr', '$argon2id$v=19$m=4096,t=3,p=1$Mp3DjJs9YxfrRDu0TUIbcw$+W67zS2FLYB6ruKPI1MHCB4KD+QrEJhN4D2VBzeVuOs');