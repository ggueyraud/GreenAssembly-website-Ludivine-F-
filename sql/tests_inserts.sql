INSERT INTO project_categories (name, "order") VALUES
('Lorem', 1),
('ipsum', 2),
('Dolor', 3),
('Sit', 4);

INSERT INTO projects (name, description, content) VALUES
('Lorem ipsum dolor sit amet, consectetur adipiscing elit. Pellentesque ut.', null, 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Suspendisse ullamcorper, lacus in porttitor porta, sem ex sollicitudin urna, ut mattis felis dolor vel enim. In vitae bibendum tellus. Curabitur ut lectus quis quam sollicitudin dictum eu quis tellus. Nulla id justo sit amet ligula hendrerit sollicitudin vel non elit. Nullam pulvinar fringilla orci in volutpat. Nullam mauris ex, sodales non vehicula ut, vulputate ac neque. Sed odio dui, mattis sit amet tincidunt ac, convallis vitae turpis. Vestibulum vel eleifend enim. In id condimentum ante. Sed fringilla leo molestie dignissim mollis. Donec in sodales odio. Ut ullamcorper viverra nibh, nec mattis.'),
('Lorem ipsum dolor sit amet, consectetur adipiscing elit.', 'Lorem ipsum dolor sit amet', 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Ut rhoncus mi augue, eget posuere felis posuere imperdiet. Nunc aliquam tempor eros, in pulvinar tortor volutpat congue. Donec porta dui commodo neque euismod venenatis. Quisque in augue dictum, finibus lorem eu, laoreet turpis. Nam sagittis purus eget tincidunt pretium. Sed tincidunt ligula non fringilla sagittis. Sed lobortis pulvinar cursus. Cras ullamcorper, massa nec ultrices vehicula, elit ex mattis mi, in ultrices odio nulla at turpis. Nullam congue urna non lectus egestas sagittis. Aenean non ultricies dui.Donec eget faucibus libero. Nulla sed quam id augue tempus eleifend. Mauris eleifend cursus enim et imperdiet. Vestibulum et cursus sem, non tincidunt diam. Nulla eget odio lobortis, placerat felis tempus, dictum ipsum. Donec a lorem auctor ligula fermentum semper eu sit amet dolor. Mauris mattis, neque consequat varius commodo, mi libero pellentesque urna, gravida eleifend mauris enim id ipsum. Duis eget sem nec nulla.');

INSERT INTO projects_categories (project_id, category_id) VALUES
(1, 1),
(2, 2),
(2, 4);

INSERT INTO files (name, path) VALUES
('Lorem', '1.png'),
('Lorem', '2.png'),
('Lorem', '3.png'),
('Lorem', '4.png'),
('Lorem', '5.png');

INSERT INTO project_assets (project_id, file_id, "order") VALUES
(1, 1, 1),
(1, 2, 2),
(1, 3, 3),
(2, 1, 1),
(2, 2, 2),
(2, 3, 3),
(2, 4, 4),
(2, 5, 5);