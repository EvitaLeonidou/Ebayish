import axios from 'axios';

export interface CartItem {
  item_id: string;
  name: string;
  currently: number;
  buy_price?: number;
  images?: string[];
  listing_type: 'auction' | 'fixed_price';
}

export interface CartResponse {
  items: CartItem[];
}

class CartService {
  private baseURL = '/api';

  async getCart(): Promise<CartItem[]> {
    try {
      const response = await axios.get<CartItem[]>(`${this.baseURL}/cart`);
      return response.data;
    } catch (error) {
      console.error('Failed to fetch cart:', error);
      throw error;
    }
  }

  async addToCart(itemId: string): Promise<void> {
    try {
      await axios.post(`${this.baseURL}/cart/items/${itemId}`);
    } catch (error) {
      console.error('Failed to add item to cart:', error);
      throw error;
    }
  }

  async removeFromCart(itemId: string): Promise<void> {
    try {
      await axios.delete(`${this.baseURL}/cart/items/${itemId}`);
    } catch (error) {
      console.error('Failed to remove item from cart:', error);
      throw error;
    }
  }

  async clearCart(): Promise<void> {
    try {
      await axios.delete(`${this.baseURL}/cart`);
    } catch (error) {
      console.error('Failed to clear cart:', error);
      throw error;
    }
  }
}

export const cartService = new CartService();
