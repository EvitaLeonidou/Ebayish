import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Category } from '@/types/category';

interface CategoryFilterProps {
  categories: Category[];
  selectedCategory: number | null;
  onSelectCategory: (id: number | null) => void;
}

const CategoryFilter: React.FC<CategoryFilterProps> = ({
  categories,
  selectedCategory,
  onSelectCategory,
}) => {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Categories</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="flex flex-col space-y-2">
          <Button
            variant={selectedCategory === null ? 'outline' : 'secondary'}
            onClick={() => onSelectCategory(null)}
            className="justify-start"
          >
            All Categories
          </Button>
          {categories.map((category) => (
            <Button
              key={category.id}
              variant={selectedCategory === category.id ? 'outline' : 'secondary'}
              onClick={() => onSelectCategory(category.id)}
              className="justify-start"
            >
              {category.name}
            </Button>
          ))}
        </div>
      </CardContent>
    </Card>
  );
};

export default CategoryFilter;
