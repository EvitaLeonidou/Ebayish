/**
 * Represents an item within the user's shopping cart,
 * matching the backend cart API response structure.
 */
export interface CartItem {
  item_id: string;
  name: string;
  currently: number;
  buy_price?: number;
  images?: string[];
  listing_type: 'auction' | 'fixed_price';
}
