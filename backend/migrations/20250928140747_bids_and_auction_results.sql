CREATE TABLE bids (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id VARCHAR(50) REFERENCES items(item_id) ON DELETE CASCADE,
    bidder_user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    bidder_rating INTEGER,
    time TIMESTAMP NOT NULL,
    amount DECIMAL(10,2) NOT NULL,
    bidder_location VARCHAR(255),
    bidder_country VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE auction_results (
    item_id VARCHAR(50) PRIMARY KEY REFERENCES items(item_id) ON DELETE CASCADE,
    seller_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    winner_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    winning_amount DECIMAL(10,2),
    ended_at TIMESTAMP NOT NULL DEFAULT NOW(),
    total_bids INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
