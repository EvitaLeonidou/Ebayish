CREATE TABLE items (
    item_id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    condition VARCHAR(100),
    
    price DECIMAL(10,2) NOT NULL,
    currently DECIMAL(10,2),
    buy_price DECIMAL(10,2),
    shipping_cost DECIMAL(10,2) DEFAULT 0.00 NOT NULL,
    
    listing_type TEXT NOT NULL DEFAULT 'auction' CHECK (listing_type IN ('auction', 'fixed_price')),
    number_of_bids INTEGER DEFAULT 0,
    
    started TIMESTAMP NOT NULL,
    ends TIMESTAMP,

    location VARCHAR(255),
    country VARCHAR(100),
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,
    
    seller_user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    seller_rating DECIMAL(5,2),
    
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'ended', 'sold', 'pending', 'rejected')),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE item_categories (
    item_id VARCHAR(50) REFERENCES items(item_id) ON DELETE CASCADE,
    category_id INTEGER REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, category_id)
);

CREATE TABLE item_images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id VARCHAR(50) REFERENCES items(item_id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    original_name VARCHAR(255) NOT NULL,
    display_order INTEGER NOT NULL DEFAULT 1,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(50) NOT NULL,
    upload_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT unique_item_display_order UNIQUE (item_id, display_order),
    CONSTRAINT max_images_per_item CHECK (display_order >= 1 AND display_order <= 5)
);

CREATE OR REPLACE FUNCTION check_image_limit()
RETURNS TRIGGER AS $$
BEGIN
    IF (SELECT COUNT(*) FROM item_images WHERE item_id = NEW.item_id) >= 5 THEN
        RAISE EXCEPTION 'Item cannot have more than 5 images';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER enforce_image_limit
    BEFORE INSERT ON item_images
    FOR EACH ROW
    EXECUTE FUNCTION check_image_limit();

