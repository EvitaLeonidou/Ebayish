import React from 'react';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Category } from '@/types/category';
import { Edit, Trash2, Package } from 'lucide-react';

interface CategoryCardProps {
  category: Category;
  onEdit: (category: Category) => void;
  onDelete: (id: number) => void;
}

const CategoryCard: React.FC<CategoryCardProps> = ({ category, onEdit, onDelete }) => {
  const hasItems = (category.item_count ?? 0) > 0;

  return (
    <Card className="flex flex-col">
      <CardHeader>
        <CardTitle>{category.name}</CardTitle>
      </CardHeader>
      <CardContent className="flex-grow space-y-4">
        <p className="text-sm text-gray-600">
          {category.description || 'No description provided.'}
        </p>
        <div className="flex items-center text-sm bg-gray-50 p-3 rounded-md">
          <Package className="h-4 w-4 mr-2 text-gray-500" />
          <span className="font-medium">{category.item_count ?? 0}</span>
          <span className="ml-1 text-gray-600">items in this category</span>
        </div>
      </CardContent>
      <CardFooter className="flex gap-2">
        <Button variant="outline" size="sm" className="flex-1" onClick={() => onEdit(category)}>
          <Edit className="h-4 w-4 mr-1" /> Edit
        </Button>
        <Button
          variant="destructive"
          size="sm"
          className="flex-1"
          onClick={() => onDelete(category.id)}
          disabled={hasItems}
          title={hasItems ? 'Cannot delete categories that contain items' : 'Delete category'}
        >
          <Trash2 className="h-4 w-4 mr-1" /> Delete
        </Button>
      </CardFooter>
    </Card>
  );
};

export default CategoryCard;
