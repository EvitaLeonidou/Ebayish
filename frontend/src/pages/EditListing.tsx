import React, { useState, useEffect } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { toast } from 'sonner';
import {
  DollarSign,
  Package,
  Tag,
  Truck,
  Loader2,
  X,
  Image as ImageIcon,
  Upload,
  MapPin,
} from 'lucide-react';
import { authFetch } from '@/utils/auth-fetch';
import { useAuth } from '@/contexts/AuthContext';
import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from '@/components/ui/select';
import { Item } from '@/types/item';
import { Category } from '@/types/category';
import LocationPickerModal from '@/components/common/LocationPickerModal';
import CountrySelect from '@/components/common/CountrySelect';

interface Condition {
  id: string;
  name: string;
}

const EditListing: React.FC = () => {
  const navigate = useNavigate();
  const { itemId } = useParams<{ itemId: string }>();
  const { token } = useAuth();
  const [item, setItem] = useState<Partial<Item>>({});
  const [isLoading, setIsLoading] = useState(false);
  const [isFetching, setIsFetching] = useState(true);
  const [categories, setCategories] = useState<Category[]>([]);
  const [conditions, setConditions] = useState<Condition[]>([]);

  // --- LOCATION STATE ---
  const [isMapModalOpen, setIsMapModalOpen] = useState(false);
  const [location, setLocation] = useState<{ lat: number; lng: number; name: string } | null>(null);
  const [country, setCountry] = useState<string>('');
  // --- END LOCATION STATE ---

  const [existingImages, setExistingImages] = useState<{ id: string; url: string }[]>([]);
  const [newImageFiles, setNewImageFiles] = useState<File[]>([]);
  const [newImagePreviews, setNewImagePreviews] = useState<string[]>([]);
  const [imagesToDelete, setImagesToDelete] = useState<string[]>([]);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [itemResponse, categoriesResponse] = await Promise.all([
          authFetch(`/api/items/${itemId}`),
          fetch('/api/categories'),
        ]);

        if (!itemResponse.ok) throw new Error('Failed to fetch item data');
        const itemData = await itemResponse.json();

        const transformedItem = {
          ...itemData,
          images: itemData.images?.map((img: any) => img.url) || [],
        };
        setItem(transformedItem);

        // --- Set initial location and country state ---
        if (itemData.latitude && itemData.longitude) {
          setLocation({
            lat: itemData.latitude,
            lng: itemData.longitude,
            name: itemData.location || 'Unknown Location',
          });
        } else if (itemData.location) {
          setLocation({ lat: 0, lng: 0, name: itemData.location }); // Fallback if no coords
        }
        setCountry(itemData.country || 'United States');
        // --- End set location ---

        if (itemData.images && itemData.images.length > 0) {
          const existingImgs = itemData.images.map((img: any) => ({
            id: img.id || img.filename || `img-${Date.now()}`,
            url: img.url,
          }));
          setExistingImages(existingImgs);
        }

        if (categoriesResponse.ok) {
          const categoriesData = await categoriesResponse.json();
          setCategories(categoriesData);
        } else {
          toast.warn('Could not load categories.');
        }

        setConditions([
          { id: 'new', name: 'New' },
          { id: 'like_new', name: 'Used - Like New' },
          { id: 'very_good', name: 'Used - Very Good' },
          { id: 'good', name: 'Used - Good' },
          { id: 'acceptable', name: 'Used - Acceptable' },
        ]);
      } catch (error) {
        toast.error('Failed to load listing data for editing.');
        navigate('/user/profile');
      } finally {
        setIsFetching(false);
      }
    };
    fetchData();
  }, [itemId, navigate]);

  useEffect(() => {
    return () => {
      newImagePreviews.forEach((url) => URL.revokeObjectURL(url));
    };
  }, [newImagePreviews]);

  const handleNewImageChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      const files = Array.from(e.target.files);
      const totalImages = existingImages.length + newImageFiles.length + files.length;

      if (totalImages > 5) {
        toast.error('Maximum 5 images allowed per listing');
        return;
      }

      const newFiles = [...newImageFiles, ...files];
      const newPreviews = [...newImagePreviews];

      files.forEach((file) => {
        newPreviews.push(URL.createObjectURL(file));
      });

      setNewImageFiles(newFiles);
      setNewImagePreviews(newPreviews);
    }
  };

  const removeExistingImage = (imageId: string) => {
    setExistingImages((prev) => prev.filter((img) => img.id !== imageId));
    setImagesToDelete((prev) => [...prev, imageId]);
  };

  const removeNewImage = (index: number) => {
    const newFiles = [...newImageFiles];
    const newPreviews = [...newImagePreviews];
    URL.revokeObjectURL(newPreviews[index]);
    newFiles.splice(index, 1);
    newPreviews.splice(index, 1);
    setNewImageFiles(newFiles);
    setNewImagePreviews(newPreviews);
  };

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    const { name, value } = e.target;
    setItem((prev) => ({ ...prev, [name]: value }));
  };

  const handleSelectChange = (name: string, value: string) => {
    setItem((prev) => ({ ...prev, [name]: value }));
  };

  const handleLocationSelected = (selectedLocation: { lat: number; lng: number; name: string }) => {
    setLocation(selectedLocation);
    setItem((prev) => ({
      ...prev,
      location: selectedLocation.name,
      latitude: selectedLocation.lat,
      longitude: selectedLocation.lng,
    }));
  };

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setIsLoading(true);

    if (!location) {
      toast.error('Please select an item location on the map.');
      setIsLoading(false);
      return;
    }

    const payload = {
      name: item.name,
      description: item.description,
      price: item.price ? parseFloat(String(item.price)) : undefined,
      condition: item.condition,
      location: location.name,
      latitude: location.lat,
      longitude: location.lng,
      country: country,
      // NOTE: We don't send all fields, only those that can be updated.
      // The backend should handle partial updates.
    };

    try {
      const response = await authFetch(`/api/items/${itemId}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText || 'Failed to update listing.');
      }

      if (imagesToDelete.length > 0) {
        await Promise.all(
          imagesToDelete.map((imageId) =>
            authFetch(`/api/items/${itemId}/images/${imageId}`, { method: 'DELETE' })
          )
        );
      }

      if (newImageFiles.length > 0) {
        const imageFormData = new FormData();
        newImageFiles.forEach((file) => imageFormData.append('images', file));

        const headers: Record<string, string> = {};
        if (token) {
          headers['Authorization'] = `Bearer ${token}`;
        }

        await fetch(`/api/items/${itemId}/images`, {
          method: 'POST',
          headers,
          body: imageFormData,
        });
      }

      toast.success('Listing updated successfully!');
      navigate('/user/profile');
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'An unknown error occurred.';
      toast.error(`Error: ${errorMessage}`);
    } finally {
      setIsLoading(false);
    }
  };

  if (isFetching) {
    return (
      <div className="flex justify-center items-center h-64">
        <Loader2 className="h-12 w-12 animate-spin text-blue-600" />
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto">
      <LocationPickerModal
        isOpen={isMapModalOpen}
        onClose={() => setIsMapModalOpen(false)}
        onLocationSelect={handleLocationSelected}
        initialPosition={location ? { lat: location.lat, lng: location.lng } : undefined}
      />
      <div className="text-center mb-10">
        <h1 className="text-4xl font-bold text-gray-900">Edit Listing</h1>
        <p className="mt-2 text-lg text-gray-600">Update the details for your item.</p>
      </div>

      <form onSubmit={handleSubmit}>
        <Card className="mb-6">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Package className="h-5 w-5" /> Item Details
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="title">Title</Label>
              <Input
                id="title"
                name="name"
                value={item.name || ''}
                onChange={handleInputChange}
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="description">Description</Label>
              <Textarea
                id="description"
                name="description"
                value={item.description || ''}
                onChange={handleInputChange}
                rows={5}
                required
              />
            </div>
          </CardContent>
        </Card>

        <Card className="mb-6">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Tag className="h-5 w-5" /> Categorization
            </CardTitle>
          </CardHeader>
          <CardContent className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="category_id">Category</Label>
              <Select
                name="category_id"
                value={(item.categories?.[0] || '').toString()}
                onValueChange={(value) => handleSelectChange('categories', value)}
                required
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select a category" />
                </SelectTrigger>
                <SelectContent>
                  {categories.map((cat) => (
                    <SelectItem key={cat.id} value={cat.name}>
                      {cat.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="condition">Condition</Label>
              <Select
                name="condition"
                value={item.condition}
                onValueChange={(value) => handleSelectChange('condition', value)}
                required
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select condition" />
                </SelectTrigger>
                <SelectContent>
                  {conditions.map((cond) => (
                    <SelectItem key={cond.id} value={cond.id}>
                      {cond.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </CardContent>
        </Card>

        <Card className="mb-6">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <DollarSign className="h-5 w-5" /> Price
            </CardTitle>
          </CardHeader>
          <CardContent className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="price">Price</Label>
              <div className="relative">
                <DollarSign className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
                <Input
                  id="price"
                  name="price"
                  type="number"
                  step="0.01"
                  value={String(item.price || '')}
                  onChange={handleInputChange}
                  className="pl-9"
                  required
                />
              </div>
            </div>
          </CardContent>
        </Card>

        <Card className="mb-6">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Truck className="h-5 w-5" /> Shipping Details
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="listing_location">Item Location</Label>
                <div className="flex items-center gap-2">
                  <Input
                    id="listing_location"
                    name="location"
                    placeholder="Select location on map"
                    value={location?.name || ''}
                    readOnly
                    required
                    className="flex-grow bg-gray-100"
                  />
                  <Button type="button" variant="outline" onClick={() => setIsMapModalOpen(true)}>
                    <MapPin className="h-4 w-4 mr-2" />
                    Select
                  </Button>
                </div>
              </div>
              <div className="space-y-2">
                <Label htmlFor="country">Country</Label>
                <CountrySelect value={country} onChange={setCountry} />
              </div>
            </div>
          </CardContent>
        </Card>

        <Card className="mb-6">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <ImageIcon className="h-5 w-5" /> Images
            </CardTitle>
            <CardDescription>Add or remove images for your listing.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {existingImages.length > 0 && (
              <div>
                <Label className="text-sm font-medium">Current Images</Label>
                <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mt-2">
                  {existingImages.map((image) => (
                    <div key={image.id} className="relative group">
                      <img
                        src={image.url}
                        alt="Current item"
                        className="w-full h-24 object-cover rounded-md border"
                      />
                      <button
                        type="button"
                        onClick={() => removeExistingImage(image.id)}
                        className="absolute top-1 right-1 bg-red-500 text-white rounded-full p-1 opacity-0 group-hover:opacity-100 transition-opacity"
                      >
                        <X className="h-3 w-3" />
                      </button>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {newImagePreviews.length > 0 && (
              <div>
                <Label className="text-sm font-medium">New Images to Add</Label>
                <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mt-2">
                  {newImagePreviews.map((preview, index) => (
                    <div key={`new-${index}`} className="relative group">
                      <img
                        src={preview}
                        alt="New item"
                        className="w-full h-24 object-cover rounded-md border border-green-200"
                      />
                      <button
                        type="button"
                        onClick={() => removeNewImage(index)}
                        className="absolute top-1 right-1 bg-red-500 text-white rounded-full p-1 opacity-0 group-hover:opacity-100 transition-opacity"
                      >
                        <X className="h-3 w-3" />
                      </button>
                    </div>
                  ))}
                </div>
              </div>
            )}

            <div>
              <Label htmlFor="new-images" className="text-sm font-medium">
                Add New Images
              </Label>
              <div className="mt-2">
                <label
                  htmlFor="new-images"
                  className="flex flex-col items-center justify-center w-full h-32 border-2 border-gray-300 border-dashed rounded-lg cursor-pointer bg-gray-50 hover:bg-gray-100"
                >
                  <div className="flex flex-col items-center justify-center pt-5 pb-6">
                    <Upload className="w-8 h-8 mb-2 text-gray-500" />
                    <p className="mb-2 text-sm text-gray-500">
                      <span className="font-semibold">Click to upload</span>
                    </p>
                  </div>
                  <input
                    id="new-images"
                    type="file"
                    className="hidden"
                    multiple
                    accept="image/*"
                    onChange={handleNewImageChange}
                  />
                </label>
              </div>
            </div>
          </CardContent>
        </Card>

        <div className="flex justify-end gap-4">
          <Button
            type="button"
            variant="outline"
            onClick={() => navigate('/user/profile')}
            disabled={isLoading}
          >
            Cancel
          </Button>
          <Button type="submit" className="bg-blue-600 hover:bg-blue-700" disabled={isLoading}>
            {isLoading ? 'Updating...' : 'Save Changes'}
          </Button>
        </div>
      </form>
    </div>
  );
};

export default EditListing;
