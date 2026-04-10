INSERT INTO users (username, password_hash, role)
VALUES (
    'admin',
    '$2b$12$ktPoOOZv2X6Nn.Zh2S/2ZeQ7kFWXZYWKCSTsK9jafeLu9BoXK4g/W',
    'admin'
) ON CONFLICT (username) DO NOTHING;
