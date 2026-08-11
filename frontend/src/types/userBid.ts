import { Item } from './item';

/**
 * Represents a bid made by a user, including the full details
 * of the item that was bid on.
 */
export interface UserBid {
  id: number;
  amount: number;
  created_at: string;
  item: Item; // The associated item
}
