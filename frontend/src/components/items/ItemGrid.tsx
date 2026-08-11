import React from 'react';
import ItemCard from './ItemCard';
import { Item } from '@/types/item';
import { SearchX } from 'lucide-react';

interface ItemGridProps {
  items: Item[];
}

const ItemGrid: React.FC<ItemGridProps> = ({ items }) => {
  if (items.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center text-center h-64 border-2 border-dashed border-gray-300 rounded-lg">
        <SearchX className="h-16 w-16 text-gray-400 mb-4" />
        <h3 className="text-xl font-semibold text-gray-800">No Items Found</h3>
        <p className="text-gray-500">Try adjusting your search or filter criteria.</p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
      {items.map((item) => (
        <ItemCard key={item.item_id} item={item} />
      ))}
    </div>
  );
};

export default ItemGrid;
