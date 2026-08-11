import React, { useState, useEffect } from 'react';
import { toast } from 'sonner';
import { Category } from '@/types/category';
import { Item } from '@/types/item';
import { Button } from '@/components/ui/button';
import { PlusCircle, Loader2, ListTree } from 'lucide-react';
import CategoryCard from '@/components/admin/CategoryCard';
import CategoryForm from '@/components/admin/CategoryForm';
import { authFetch } from '@/utils/auth-fetch';

const CategoryManagement: React.FC = () => {
  const [categories, setCategories] = useState<Category[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [editingCategory, setEditingCategory] = useState<Category | null>(null);

  useEffect(() => {
    fetchCategoriesAndItems();
  }, []);

  const fetchCategoriesAndItems = async () => {
    try {
      // Fetch categories and items in parallel
      const [categoriesResponse, itemsResponse] = await Promise.all([
        fetch('/api/categories'),
        fetch('/api/items'),
      ]);

      if (!categoriesResponse.ok) throw new Error('Failed to fetch categories');
      if (!itemsResponse.ok) throw new Error('Failed to fetch items');

      const categoriesData = await categoriesResponse.json();
      const itemsData: Item[] = await itemsResponse.json();

      // Calculate item count for each category
      const categoriesWithCounts = categoriesData.map((category: any) => ({
        ...category,
        description: category.description || '', // Provide default empty description
        item_count: itemsData.filter(
          (item) => item.categories && item.categories.includes(category.name)
        ).length,
      }));

      setCategories(categoriesWithCounts);
    } catch (error) {
      toast.error('Could not load categories and items.');
    } finally {
      setIsLoading(false);
    }
  };

  const handleFormSubmit = async (formData: Omit<Category, 'id'>) => {
    const isEditing = !!editingCategory;
    const url = isEditing ? `/api/categories/${editingCategory.id}` : '/api/categories';
    const method = isEditing ? 'PUT' : 'POST';

    try {
      // Only send the name field since backend doesn't handle description
      const requestData = { name: formData.name };
      const response = await authFetch(url, {
        method,
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(requestData),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || `Failed to ${isEditing ? 'update' : 'create'} category`);
      }

      toast.success(`Category ${isEditing ? 'updated' : 'created'} successfully!`);
      setIsFormOpen(false);
      fetchCategoriesAndItems(); // Re-fetch to get the latest data
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'An unknown error occurred.');
    }
  };

  const handleEdit = (category: Category) => {
    setEditingCategory(category);
    setIsFormOpen(true);
  };

  const handleDelete = async (categoryId: number) => {
    if (window.confirm('Are you sure you want to delete this category?')) {
      try {
        const response = await authFetch(`/api/categories/${categoryId}`, { method: 'DELETE' });
        if (!response.ok) {
          const errorData = await response.json();
          throw new Error(errorData.error || 'Failed to delete category');
        }
        toast.success('Category deleted successfully.');
        fetchCategoriesAndItems(); // Re-fetch to get updated data
      } catch (error) {
        toast.error(error instanceof Error ? error.message : 'An unknown error occurred.');
      }
    }
  };

  const openCreateForm = () => {
    setEditingCategory(null);
    setIsFormOpen(true);
  };

  if (isLoading) {
    return (
      <div className="flex justify-center items-center h-64">
        <Loader2 className="h-12 w-12 animate-spin text-blue-600" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-bold text-gray-900">Category Management</h1>
        <Button onClick={openCreateForm}>
          <PlusCircle className="h-4 w-4 mr-2" />
          Create Category
        </Button>
      </div>

      {categories.length > 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {categories.map((category) => (
            <CategoryCard
              key={category.id}
              category={category}
              onEdit={handleEdit}
              onDelete={handleDelete}
            />
          ))}
        </div>
      ) : (
        <div className="text-center py-16 border-2 border-dashed rounded-lg">
          <ListTree className="mx-auto h-16 w-16 text-gray-400" />
          <h3 className="mt-4 text-xl font-semibold text-gray-800">No Categories Found</h3>
          <p className="mt-1 text-gray-500">Get started by creating a new category.</p>
        </div>
      )}

      {isFormOpen && (
        <CategoryForm
          initialData={editingCategory}
          onSubmit={handleFormSubmit}
          onClose={() => setIsFormOpen(false)}
        />
      )}
    </div>
  );
};

export default CategoryManagement;
