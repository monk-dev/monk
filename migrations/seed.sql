INSERT INTO tags (id, tag) VALUES 
    ('6400203e-c41d-11eb-8887-5f942429a808', 'rust'), 
    ('6b6a7b58-c41d-11eb-a5c2-379cf2feed72', 'linux'), 
    ('6f620f14-c41d-11eb-8302-e347c88a909c', 'cool');

INSERT INTO articles (id, name, description, url) VALUES
    ('77fb3d58-c41d-11eb-a4b0-37f7c8707735', 'Accelerating networking with AF_XDP', 'Cool article on network capturing', 'https://lwn.net/Articles/750845/'),
    ('9246b3b8-c41d-11eb-884d-87d7cb7249ed', 'Making our own executable packer', 'Deep dive into ELF', 'https://fasterthanli.me/series/making-our-own-executable-packer');

INSERT INTO article_tag (article_id, tag_id) VALUES
    -- AF_XDP -> Linux, Cool
    ('77fb3d58-c41d-11eb-a4b0-37f7c8707735', '6b6a7b58-c41d-11eb-a5c2-379cf2feed72'),
    ('77fb3d58-c41d-11eb-a4b0-37f7c8707735', '6f620f14-c41d-11eb-8302-e347c88a909c'),
    -- AF_XDP -> Rust, Cool
    ('9246b3b8-c41d-11eb-884d-87d7cb7249ed', '6400203e-c41d-11eb-8887-5f942429a808'),
    ('9246b3b8-c41d-11eb-884d-87d7cb7249ed', '6f620f14-c41d-11eb-8302-e347c88a909c');