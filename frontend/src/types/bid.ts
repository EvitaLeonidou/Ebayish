export interface Bid {
  id: number;
  amount: number;
  created_at: string; // ISO 8601 date string
  user: {
    id: number;
    username: string;
  };
}
