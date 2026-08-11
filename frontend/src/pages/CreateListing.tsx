import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
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
  Type,
  MapPin,
  Truck,
  Image as ImageIcon,
  X,
  Loader2,
} from 'lucide-react';
import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from '@/components/ui/select';
import { useAuth } from '@/contexts/AuthContext';
import LocationPickerModal from '@/components/common/LocationPickerModal';
import CountrySelect from '@/components/common/CountrySelect';

interface Category {
  id: number;
  name: string;
}

const conditions = [
  { id: 'new', name: 'New' },
  { id: 'like_new', name: 'Used - Like New' },
  { id: 'very_good', name: 'Used - Very Good' },
  { id: 'good', name: 'Used - Good' },
  { id: 'acceptable', name: 'Used - Acceptable' },
];

const CreateListing: React.FC = () => {
  const navigate = useNavigate();
  const { user, token } = useAuth();
  const [isLoading, setIsLoading] = useState(false);
  const [listingType, setListingType] = useState<'auction' | 'fixed_price'>('fixed_price');
  const [imagePreviews, setImagePreviews] = useState<string[]>([]);
  const [imageFiles, setImageFiles] = useState<File[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [isCategoriesLoading, setIsCategoriesLoading] = useState(true);

  // --- LOCATION STATE ---
  const [isMapModalOpen, setIsMapModalOpen] = useState(false);
  const [location, setLocation] = useState<{ lat: number; lng: number; name: string } | null>(null);
  const [country, setCountry] = useState<string>('United States');
  // --- END LOCATION STATE ---

  useEffect(() => {
    const fetchCategories = async () => {
      try {
        const response = await fetch('/api/categories');
        if (!response.ok) {
          throw new Error('Failed to fetch categories');
        }
        const data: Category[] = await response.json();
        setCategories(data);
      } catch (error) {
        console.error('Error fetching categories:', error);
        toast.error('Could not load categories. Please try again later.');
      } finally {
        setIsCategoriesLoading(false);
      }
    };
    fetchCategories();
  }, []);

  const handleImageChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      const files = Array.from(e.target.files);
      const newImageFiles = [...imageFiles, ...files].slice(0, 5);
      const newImagePreviews = newImageFiles.map((file) => URL.createObjectURL(file));
      setImageFiles(newImageFiles);
      setImagePreviews(newImagePreviews);
    }
  };

  const removeImage = (index: number) => {
    const newImageFiles = [...imageFiles];
    const newImagePreviews = [...imagePreviews];
    newImageFiles.splice(index, 1);
    newImagePreviews.splice(index, 1);
    setImageFiles(newImageFiles);
    setImagePreviews(newImagePreviews);
  };

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setIsLoading(true);

    if (!user || !user.id) {
      toast.error('You must be logged in to create a listing.');
      setIsLoading(false);
      return;
    }

    // VALIDATION
    if (!location) {
      toast.error('Please select an item location on the map.');
      setIsLoading(false);
      return;
    }

    const formData = new FormData(e.currentTarget);
    const data = Object.fromEntries(formData.entries());
    const currentListingType = data.listing_type as 'auction' | 'fixed_price';

    const selectedCategoryId = parseInt(data.category_id as string, 10);
    const selectedCategory = categories.find((c) => c.id === selectedCategoryId);
    if (!selectedCategory) {
      toast.error('Please select a valid category.');
      setIsLoading(false);
      return;
    }

    const now = new Date();
    let payload: any;

    const basePayload = {
      name: data.title as string,
      description: data.description as string,
      categories: [selectedCategory.name],
      condition: data.condition as string,
      location: location.name, // From map
      latitude: location.lat, // From map
      longitude: location.lng, // From map
      country: country, // From dropdown
      seller_user_id: user.id,
      started: now.toISOString(),
      status: 'active',
    };

    if (currentListingType === 'auction') {
      const durationDays = parseInt(data.auction_duration as string, 10) || 7;
      const endDate = new Date(now.getTime() + durationDays * 24 * 60 * 60 * 1000);

      payload = {
        ...basePayload,
        listing_type: 'auction',
        price: parseFloat(data.starting_price as string),
        currently: parseFloat(data.starting_price as string),
        buy_price: data.buy_it_now_price ? parseFloat(data.buy_it_now_price as string) : null,
        number_of_bids: 0,
        ends: endDate.toISOString(),
      };
    } else {
      // 'fixed_price'
      payload = {
        ...basePayload,
        listing_type: 'fixed_price',
        price: parseFloat(data.starting_price as string),
      };
    }

    try {
      const response = await fetch('/api/items', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });

      const responseData = await response.json();
      if (!response.ok) {
        throw new Error(responseData.message || responseData.error || 'Failed to create listing');
      }

      const itemId = responseData.item_id;

      if (imageFiles.length > 0) {
        const imageFormData = new FormData();
        imageFiles.forEach((file) => {
          imageFormData.append('images', file);
        });

        const headers: Record<string, string> = {};
        if (token) {
          headers['Authorization'] = `Bearer ${token}`;
        }

        const imageResponse = await fetch(`/api/items/${itemId}/images`, {
          method: 'POST',
          headers,
          body: imageFormData,
        });

        if (!imageResponse.ok) {
          toast.warning('Listing created, but image upload failed.');
        }
      }

      toast.success('Your listing has been created successfully!');
      navigate(`/item/${itemId}`);
    } catch (error) {
      const msg = error instanceof Error ? error.message : 'An unknown error occurred.';
      toast.error(`Error: ${msg}`);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="max-w-4xl mx-auto">
      <LocationPickerModal
        isOpen={isMapModalOpen}
        onClose={() => setIsMapModalOpen(false)}
        onLocationSelect={setLocation}
      />
      <div className="text-center mb-10">
        <h1 className="text-4xl font-bold text-gray-900">Create a New Listing</h1>
        <p className="mt-2 text-lg text-gray-600">
          Fill out the details below to put your item up for sale.
        </p>
      </div>

      <form onSubmit={handleSubmit}>
        <Card className="mb-6">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Package className="h-5 w-5" /> What are you selling?
            </CardTitle>
            <CardDescription>
              Provide the essential details about your item. A good title and description will
              attract more buyers.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="title">Title</Label>
              <Input
                id="title"
                name="title"
                placeholder="e.g., Apple iPhone 15 Pro, 256GB, Space Black"
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="description">Description</Label>
              <Textarea
                id="description"
                name="description"
                placeholder="Describe the item's condition, features, and any included accessories."
                rows={5}
                required
              />
            </div>
          </CardContent>
        </Card>

        <Card className="mb-6">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <ImageIcon className="h-5 w-5" /> Add Photos
            </CardTitle>
            <CardDescription>
              Upload up to 5 photos. Good photos are key to a successful sale.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-3 sm:grid-cols-5 gap-4">
              {imagePreviews.map((preview, index) => (
                <div key={index} className="relative aspect-square">
                  <img
                    src={preview}
                    alt={`Preview ${index}`}
                    className="w-full h-full object-cover rounded-md"
                  />
                  <Button
                    type="button"
                    variant="destructive"
                    size="sm"
                    className="absolute top-1 right-1 h-6 w-6 p-0"
                    onClick={() => removeImage(index)}
                  >
                    <X className="h-4 w-4" />
                  </Button>
                </div>
              ))}
              {imagePreviews.length < 5 && (
                <Label
                  htmlFor="image-upload"
                  className="flex flex-col items-center justify-center w-full aspect-square border-2 border-dashed border-gray-300 rounded-md cursor-pointer hover:bg-gray-50"
                >
                  <ImageIcon className="h-8 w-8 text-gray-400" />
                  <span className="text-xs text-gray-500 mt-1">Add Photo</span>
                </Label>
              )}
            </div>
            <Input
              id="image-upload"
              type="file"
              multiple
              accept="image/*"
              className="hidden"
              onChange={handleImageChange}
            />
          </CardContent>
        </Card>

        <Card className="mb-6">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Tag className="h-5 w-5" /> Categorization
            </CardTitle>
            <CardDescription>
              Help buyers find your item by categorizing it correctly.
            </CardDescription>
          </CardHeader>
          <CardContent className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="category_id">Category</Label>
              <Select
                name="category_id"
                required
                disabled={isCategoriesLoading || categories.length === 0}
              >
                <SelectTrigger className="w-full h-10 px-3 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500">
                  <SelectValue
                    placeholder={
                      isCategoriesLoading ? 'Loading categories...' : 'Select a category'
                    }
                  />
                </SelectTrigger>
                <SelectContent>
                  {categories.map((cat) => (
                    <SelectItem key={cat.id} value={cat.id.toString()}>
                      {cat.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="condition">Condition</Label>
              <Select name="condition" required defaultValue={conditions[0].id}>
                <SelectTrigger className="w-full h-10 px-3 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500">
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
              <Type className="h-5 w-5" /> Listing Format & Price
            </CardTitle>
            <CardDescription>Choose how you want to sell your item.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex gap-4">
              <Button
                type="button"
                variant={listingType === 'fixed_price' ? 'default' : 'outline'}
                onClick={() => setListingType('fixed_price')}
                className="flex-1"
              >
                Fixed Price
              </Button>
              <Button
                type="button"
                variant={listingType === 'auction' ? 'default' : 'outline'}
                onClick={() => setListingType('auction')}
                className="flex-1"
              >
                Auction
              </Button>
              <input type="hidden" name="listing_type" value={listingType} />
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="starting_price">
                  {listingType === 'auction' ? 'Starting Price' : 'Price'}
                </Label>
                <div className="relative">
                  <DollarSign className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
                  <Input
                    id="starting_price"
                    name="starting_price"
                    type="number"
                    step="0.01"
                    placeholder="e.g., 99.99"
                    className="pl-9"
                    required
                  />
                </div>
              </div>
              {listingType === 'auction' && (
                <div className="space-y-2">
                  <Label htmlFor="buy_it_now_price">Buy It Now Price (Optional)</Label>
                  <div className="relative">
                    <DollarSign className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
                    <Input
                      id="buy_it_now_price"
                      name="buy_it_now_price"
                      type="number"
                      step="0.01"
                      placeholder="e.g., 149.99"
                      className="pl-9"
                    />
                  </div>
                </div>
              )}
              {listingType === 'auction' && (
                <div className="space-y-2">
                  <Label htmlFor="auction_duration">Auction Duration (days)</Label>
                  <Input
                    id="auction_duration"
                    name="auction_duration"
                    type="number"
                    defaultValue={7}
                    required={listingType === 'auction'}
                  />
                </div>
              )}
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
                    name="listing_location"
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
            <div className="space-y-2">
              <Label htmlFor="shipping_cost">Shipping Cost</Label>
              <div className="relative">
                <DollarSign className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
                <Input
                  id="shipping_cost"
                  name="shipping_cost"
                  type="number"
                  step="0.01"
                  placeholder="Enter 0 for free shipping"
                  className="pl-9"
                />
              </div>
            </div>
          </CardContent>
        </Card>

        <div className="flex justify-end gap-4">
          <Button
            type="button"
            variant="outline"
            onClick={() => navigate('/')}
            disabled={isLoading}
          >
            Cancel
          </Button>
          <Button type="submit" className="bg-blue-600 hover:bg-blue-700" disabled={isLoading}>
            {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {isLoading ? 'Submitting...' : 'Create Listing'}
          </Button>
        </div>
      </form>
    </div>
  );
};

export default CreateListing;
