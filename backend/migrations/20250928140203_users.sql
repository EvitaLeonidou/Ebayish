CREATE TABLE users (
    id UUID NOT NULL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(50) NOT NULL,
    last_name VARCHAR(50) NOT NULL,
    phone VARCHAR(20) NOT NULL,
    date_of_birth DATE NOT NULL,
    status TEXT NOT NULL DEFAULT 'confirmed',
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    seller_rating DECIMAL(5,2) CHECK (seller_rating >= 0.0 AND seller_rating <= 100.0),
    tax_id VARCHAR(50),
    location VARCHAR(100),
    country VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
