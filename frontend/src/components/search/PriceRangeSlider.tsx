import React, { useState, useEffect } from 'react';
import { Slider } from '@/components/ui/slider';

interface PriceRangeSliderProps {
  minPrice: string;
  maxPrice: string;
  onPriceChange: (minPrice: string, maxPrice: string) => void;
  maxRange?: number;
  items?: Array<{ price: number; currently?: number; buy_price?: number }>;
}

const PriceRangeSlider: React.FC<PriceRangeSliderProps> = ({
  minPrice,
  maxPrice,
  onPriceChange,
  maxRange,
  items = [],
}) => {
  // Calculate dynamic max range based on items
  const dynamicMaxRange = React.useMemo(() => {
    if (maxRange) return maxRange;

    if (items.length === 0) return 10000; // Default fallback

    const maxItemPrice = items.reduce((max, item) => {
      // Match the price calculation logic from ItemCard
      let currentPrice;
      if (item.listing_type === 'auction') {
        currentPrice = item.currently ?? item.price;
      } else {
        currentPrice = item.price;
      }
      return Math.max(max, currentPrice);
    }, 0);

    // Round up to next thousand for a nice range
    return Math.ceil(maxItemPrice / 1000) * 1000;
  }, [maxRange, items]);
  const [sliderValues, setSliderValues] = useState<[number, number]>([
    minPrice ? Number(minPrice) : 0,
    maxPrice ? Number(maxPrice) : dynamicMaxRange,
  ]);

  // Update slider values when props change
  useEffect(() => {
    setSliderValues([
      minPrice ? Number(minPrice) : 0,
      maxPrice ? Number(maxPrice) : dynamicMaxRange,
    ]);
  }, [minPrice, maxPrice, dynamicMaxRange]);

  const handleSliderChange = (values: number[]) => {
    const [min, max] = values;
    setSliderValues([min, max]);

    // Call the parent's onChange with string values (empty string only for max default)
    const minStr = min.toString();
    const maxStr = max === dynamicMaxRange ? '' : max.toString();

    onPriceChange(minStr, maxStr);
  };

  const formatPrice = (price: number): string => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0,
    }).format(price);
  };

  return (
    <div className="space-y-4">
      <h3 className="text-lg font-semibold">Price Range</h3>

      <div className="space-y-3">
        <div className="flex justify-between text-sm text-gray-600">
          <span>{formatPrice(sliderValues[0])}</span>
          <span>{formatPrice(sliderValues[1])}</span>
        </div>

        <Slider
          value={sliderValues}
          onValueChange={handleSliderChange}
          max={dynamicMaxRange}
          min={0}
          step={50}
          className="w-full"
        />

        <div className="flex justify-between text-xs text-gray-500">
          <span>$0</span>
          <span>{formatPrice(dynamicMaxRange)}</span>
        </div>
      </div>
    </div>
  );
};

export default PriceRangeSlider;
