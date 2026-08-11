CREATE TABLE seeding_status (
    id SERIAL PRIMARY KEY,
    operation_name VARCHAR(255) UNIQUE NOT NULL,
    completed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE OR REPLACE FUNCTION get_current_timestamp_tz()
RETURNS TIMESTAMPTZ AS $$
BEGIN
    RETURN NOW();
END;
$$ LANGUAGE plpgsql;

CREATE VIEW active_auctions AS
SELECT 
    i.*,
    COUNT(b.id) as current_bid_count,
    MAX(b.amount) as highest_bid
FROM items i
LEFT JOIN bids b ON i.item_id = b.item_id
WHERE i.listing_type = 'auction' 
  AND i.status = 'active'
  AND i.ends > NOW()
GROUP BY i.item_id;

CREATE VIEW ended_auctions_to_process AS
SELECT 
    i.*,
    COUNT(b.id) as total_bids,
    MAX(b.amount) as winning_bid,
    (SELECT bidder_user_id FROM bids WHERE item_id = i.item_id ORDER BY amount DESC, time ASC LIMIT 1) as winner_user_id
FROM items i
LEFT JOIN bids b ON i.item_id = b.item_id
WHERE i.listing_type = 'auction'
  AND i.status = 'active'
  AND i.ends <= NOW()
  AND i.item_id NOT IN (SELECT item_id FROM auction_results)
GROUP BY i.item_id;

CREATE OR REPLACE FUNCTION process_ended_auctions()
RETURNS INTEGER AS $$
DECLARE
    processed_count INTEGER := 0;
    auction_record RECORD;
BEGIN
    FOR auction_record IN 
        SELECT * FROM ended_auctions_to_process
    LOOP
        INSERT INTO auction_results (
            item_id,
            seller_user_id,
            winner_user_id,
            winning_amount,
            ended_at,
            total_bids
        ) VALUES (
            auction_record.item_id,
            auction_record.seller_user_id,
            auction_record.winner_user_id,
            auction_record.winning_bid,
            auction_record.ends,
            auction_record.total_bids
        );
        
        UPDATE items 
        SET status = 'ended' 
        WHERE item_id = auction_record.item_id;
        
        processed_count := processed_count + 1;
    END LOOP;
    
    RETURN processed_count;
END;
$$ LANGUAGE plpgsql;
