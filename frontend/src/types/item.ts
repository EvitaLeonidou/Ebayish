export interface Item {
  item_id: string;
  listing_type: 'auction' | 'fixed_price';
  name: string;
  description: string;
  price: number;
  currently?: number;
  buy_price?: number;
  number_of_bids?: number;
  ends?: string;
  images: string[];
  seller_user_id: string;
  seller_rating: number;
  categories: string[];
  condition: string;
  location?: string;
  country?: string;
  latitude?: number;
  longitude?: number;
  status?: 'active' | 'ended' | 'sold' | 'pending' | 'rejected';
  winner?: {
    id: number;
    username: string;
  };
}
