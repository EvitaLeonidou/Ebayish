export interface Category {
  id: number;
  name: string;
  description?: string; // Make description optional since backend doesn't provide it
  item_count?: number; // Number of items in this category
}
