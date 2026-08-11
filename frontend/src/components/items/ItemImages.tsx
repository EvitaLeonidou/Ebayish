import React, { useState } from 'react';
import { Card } from '@/components/ui/card';
import { Package, ChevronLeft, ChevronRight } from 'lucide-react';

interface ItemImagesProps {
  images: string[];
  name: string;
}

const ItemImages: React.FC<ItemImagesProps> = ({ images, name }) => {
  const [currentImageIndex, setCurrentImageIndex] = useState(0);

  const hasImages = images && images.length > 0;
  const hasMultipleImages = images && images.length > 1;

  // Use relative path for image URLs (Vite proxy will handle routing to backend)
  const getImageUrl = (imagePath: string | null) => {
    if (!imagePath) return null;
    // Return the path as-is for Vite proxy to handle
    return imagePath;
  };

  const nextImage = () => {
    if (images && images.length > 0) {
      setCurrentImageIndex((prev) => (prev + 1) % images.length);
    }
  };

  const prevImage = () => {
    if (images && images.length > 0) {
      setCurrentImageIndex((prev) => (prev - 1 + images.length) % images.length);
    }
  };

  const goToImage = (index: number) => {
    setCurrentImageIndex(index);
  };

  const currentImageUrl = hasImages ? getImageUrl(images[currentImageIndex]) : null;

  return (
    <div className="space-y-4">
      {/* Main Image Display */}
      <Card className="p-4 flex items-center justify-center">
        <div className="aspect-square w-full relative group">
          {currentImageUrl ? (
            <>
              <img
                src={currentImageUrl}
                alt={name}
                className="w-full h-full object-contain rounded-md"
                onError={(e) => {
                  console.warn(`Failed to load image: ${currentImageUrl}`);
                  // Hide the img and show fallback
                  e.currentTarget.style.display = 'none';
                  const fallback = e.currentTarget.nextElementSibling as HTMLElement;
                  if (fallback) fallback.style.display = 'flex';
                }}
              />
              <div
                className="absolute inset-0 bg-gray-100 flex items-center justify-center rounded-md"
                style={{ display: 'none' }}
              >
                <Package className="h-24 w-24 text-gray-400" />
              </div>
            </>
          ) : (
            <div className="w-full h-full bg-gray-100 flex items-center justify-center rounded-md">
              <Package className="h-24 w-24 text-gray-400" />
            </div>
          )}

          {/* Navigation arrows - only show if multiple images */}
          {hasMultipleImages && (
            <>
              <button
                onClick={prevImage}
                className="absolute left-4 top-1/2 transform -translate-y-1/2 bg-black bg-opacity-50 hover:bg-opacity-70 text-white p-2 rounded-full opacity-0 group-hover:opacity-100 transition-opacity duration-200"
              >
                <ChevronLeft className="h-6 w-6" />
              </button>
              <button
                onClick={nextImage}
                className="absolute right-4 top-1/2 transform -translate-y-1/2 bg-black bg-opacity-50 hover:bg-opacity-70 text-white p-2 rounded-full opacity-0 group-hover:opacity-100 transition-opacity duration-200"
              >
                <ChevronRight className="h-6 w-6" />
              </button>
            </>
          )}

          {/* Image counter - only show if multiple images */}
          {hasMultipleImages && (
            <div className="absolute top-4 right-4 bg-black bg-opacity-50 text-white text-sm px-3 py-1 rounded">
              {currentImageIndex + 1}/{images.length}
            </div>
          )}
        </div>
      </Card>

      {/* Thumbnail Gallery - only show if multiple images */}
      {hasMultipleImages && (
        <div className="flex gap-2 overflow-x-auto pb-2">
          {images.map((image, index) => (
            <button
              key={index}
              onClick={() => goToImage(index)}
              className={`flex-shrink-0 w-16 h-16 rounded-md overflow-hidden border-2 transition-colors ${
                index === currentImageIndex
                  ? 'border-blue-500'
                  : 'border-gray-200 hover:border-gray-300'
              }`}
            >
              <img
                src={getImageUrl(image)}
                alt={`${name} - Image ${index + 1}`}
                className="w-full h-full object-cover"
                onError={(e) => {
                  e.currentTarget.style.display = 'none';
                }}
              />
            </button>
          ))}
        </div>
      )}
    </div>
  );
};

export default ItemImages;
