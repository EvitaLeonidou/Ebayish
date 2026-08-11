CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
--for recommendation algo
CREATE TABLE user_category_views (
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    category_id INTEGER REFERENCES categories(id) ON DELETE CASCADE,
    view_count INTEGER DEFAULT 1,
    last_viewed TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, category_id)
);

INSERT INTO categories (name) VALUES 
    ('Electronics'),
    ('Clothing'),
    ('Books'),
    ('Fishing'),
    ('Collectibles'),
    ('Art'),
    ('Musical Instruments'),
    ('Music');
