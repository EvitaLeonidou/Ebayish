CREATE TABLE cart (
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    item_id VARCHAR(50) REFERENCES items(item_id) ON DELETE CASCADE,
    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, item_id)
);

CREATE TABLE purchases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id VARCHAR(50) REFERENCES items(item_id) ON DELETE CASCADE,
    buyer_user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    seller_user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    purchase_price DECIMAL(10,2) NOT NULL,
    purchased_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

