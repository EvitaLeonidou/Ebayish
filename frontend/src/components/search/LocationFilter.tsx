import React, { useState, useMemo } from 'react';
import { Input } from '@/components/ui/input';
import { MapPin, X } from 'lucide-react';
import { Item } from '@/types/item';

interface LocationFilterProps {
  selectedLocation: string | null;
  onLocationChange: (location: string | null) => void;
  items: Item[];
}

const LocationFilter: React.FC<LocationFilterProps> = ({
  selectedLocation,
  onLocationChange,
  items,
}) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [showSuggestions, setShowSuggestions] = useState(false);

  // Extract unique locations from items (city part only)
  const availableLocations = useMemo(() => {
    const locationSet = new Set<string>();

    items.forEach((item) => {
      if (item.location) {
        // Extract city (part before first comma)
        const city = item.location.split(',')[0].trim();
        if (city) {
          locationSet.add(city);
        }
      }
    });

    return Array.from(locationSet).sort();
  }, [items]);

  // Filter locations based on search term
  const filteredLocations = useMemo(() => {
    if (!searchTerm.trim()) return availableLocations;

    return availableLocations.filter((location) =>
      location.toLowerCase().includes(searchTerm.toLowerCase())
    );
  }, [availableLocations, searchTerm]);

  const handleLocationSelect = (location: string) => {
    onLocationChange(location);
    setSearchTerm('');
    setShowSuggestions(false);
  };

  const handleClearLocation = () => {
    onLocationChange(null);
    setSearchTerm('');
  };

  const handleInputChange = (value: string) => {
    setSearchTerm(value);
    setShowSuggestions(true);
  };

  const handleInputFocus = () => {
    setShowSuggestions(true);
  };

  const handleInputBlur = () => {
    // Delay hiding suggestions to allow for click events
    setTimeout(() => setShowSuggestions(false), 200);
  };

  return (
    <div className="space-y-4">
      <h3 className="text-lg font-semibold flex items-center">
        <MapPin className="h-5 w-5 mr-2" />
        Location
      </h3>

      <div className="space-y-3">
        {selectedLocation && (
          <div className="flex items-center gap-2">
            <div className="inline-flex items-center gap-1 px-2 py-1 bg-gray-100 text-gray-800 text-sm rounded-md">
              <MapPin className="h-3 w-3" />
              {selectedLocation}
              <button
                onClick={handleClearLocation}
                className="ml-1 hover:bg-gray-300 rounded-full p-0.5 transition-colors"
              >
                <X className="h-3 w-3" />
              </button>
            </div>
          </div>
        )}

        <div className="relative">
          <Input
            type="text"
            placeholder="Search by city..."
            value={searchTerm}
            onChange={(e) => handleInputChange(e.target.value)}
            onFocus={handleInputFocus}
            onBlur={handleInputBlur}
            className="w-full"
          />

          {showSuggestions && filteredLocations.length > 0 && (
            <div className="absolute z-10 w-full mt-1 bg-white border border-gray-200 rounded-md shadow-lg max-h-48 overflow-y-auto">
              {filteredLocations.slice(0, 10).map((location) => (
                <button
                  key={location}
                  onClick={() => handleLocationSelect(location)}
                  className="w-full px-3 py-2 text-left hover:bg-gray-100 transition-colors duration-150"
                >
                  <div className="flex items-center">
                    <MapPin className="h-4 w-4 mr-2 text-gray-400" />
                    {location}
                  </div>
                </button>
              ))}
              {filteredLocations.length > 10 && (
                <div className="px-3 py-2 text-sm text-gray-500 border-t">
                  + {filteredLocations.length - 10} more locations
                </div>
              )}
            </div>
          )}
        </div>

        {selectedLocation && (
          <div className="text-sm text-gray-600">
            Showing items from <strong>{selectedLocation}</strong>
          </div>
        )}

        {!selectedLocation && availableLocations.length > 0 && (
          <div className="text-sm text-gray-500">
            {availableLocations.length} locations available
          </div>
        )}
      </div>
    </div>
  );
};

export default LocationFilter;
