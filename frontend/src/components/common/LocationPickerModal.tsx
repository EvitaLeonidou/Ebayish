import React, { useState, useEffect } from 'react';
import { MapContainer, TileLayer, Marker, useMapEvents } from 'react-leaflet';
import 'leaflet/dist/leaflet.css';
import L from 'leaflet';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { X, MapPin, Loader2, Check } from 'lucide-react';

// Fix for default icon issue with Leaflet and Webpack
delete (L.Icon.Default.prototype as any)._getIconUrl;
L.Icon.Default.mergeOptions({
  iconRetinaUrl: 'https://unpkg.com/leaflet@1.7.1/dist/images/marker-icon-2x.png',
  iconUrl: 'https://unpkg.com/leaflet@1.7.1/dist/images/marker-icon.png',
  shadowUrl: 'https://unpkg.com/leaflet@1.7.1/dist/images/marker-shadow.png',
});

interface LocationPickerModalProps {
  isOpen: boolean;
  onClose: () => void;
  onLocationSelect: (location: { lat: number; lng: number; name: string }) => void;
  initialPosition?: { lat: number; lng: number };
}

const LocationPicker: React.FC<{
  setPosition: (pos: L.LatLng) => void;
  position: L.LatLng;
}> = ({ setPosition, position }) => {
  const map = useMapEvents({
    click(e) {
      setPosition(e.latlng);
      map.flyTo(e.latlng, map.getZoom());
    },
  });

  return position ? <Marker position={position}></Marker> : null;
};

const LocationPickerModal: React.FC<LocationPickerModalProps> = ({
  isOpen,
  onClose,
  onLocationSelect,
  initialPosition,
}) => {
  const defaultPosition = new L.LatLng(
    initialPosition?.lat || 51.505,
    initialPosition?.lng || -0.09
  );
  const [position, setPosition] = useState<L.LatLng>(defaultPosition);
  const [address, setAddress] = useState('');
  const [isGeocoding, setIsGeocoding] = useState(false);

  useEffect(() => {
    if (position) {
      const fetchAddress = async () => {
        setIsGeocoding(true);
        try {
          const response = await fetch(
            `https://nominatim.openstreetmap.org/reverse?format=json&lat=${position.lat}&lon=${position.lng}`
          );
          if (!response.ok) throw new Error('Failed to fetch address');
          const data = await response.json();
          setAddress(data.display_name || 'Location not found');
        } catch (error) {
          console.error('Geocoding error:', error);
          setAddress('Could not retrieve address');
        } finally {
          setIsGeocoding(false);
        }
      };
      fetchAddress();
    }
  }, [position]);

  const handleConfirm = () => {
    if (position) {
      onLocationSelect({
        lat: position.lat,
        lng: position.lng,
        name: address,
      });
      onClose();
    }
  };

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 bg-black/60 flex justify-center items-center z-50 animate-in fade-in"
      onClick={onClose}
    >
      <Card className="w-full max-w-2xl m-4" onClick={(e) => e.stopPropagation()}>
        <CardHeader className="flex flex-row items-start justify-between">
          <div>
            <CardTitle>Select Item Location</CardTitle>
            <p className="text-sm text-gray-500">Click on the map to place a marker.</p>
          </div>
          <Button variant="outline" size="sm" onClick={onClose}>
            <X className="h-5 w-5" />
          </Button>
        </CardHeader>
        <CardContent>
          <div className="h-80 w-full rounded-md overflow-hidden mb-4">
            <MapContainer
              center={position}
              zoom={13}
              style={{ height: '100%', width: '100%' }}
              scrollWheelZoom={true}
            >
              <TileLayer
                url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
                attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
              />
              <LocationPicker setPosition={setPosition} position={position} />
            </MapContainer>
          </div>
          <div className="bg-gray-50 p-3 rounded-md">
            <div className="flex items-start">
              <MapPin className="h-5 w-5 mr-3 mt-1 text-gray-500 flex-shrink-0" />
              <div>
                <p className="text-sm font-medium text-gray-900">Selected Address</p>
                {isGeocoding ? (
                  <div className="flex items-center text-sm text-gray-600">
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                    Fetching address...
                  </div>
                ) : (
                  <p className="text-sm text-gray-600">{address}</p>
                )}
              </div>
            </div>
          </div>
          <div className="flex justify-end gap-2 mt-6">
            <Button type="button" variant="outline" onClick={onClose}>
              Cancel
            </Button>
            <Button
              type="button"
              onClick={handleConfirm}
              disabled={isGeocoding || !address || address.includes('Could not')}
            >
              <Check className="h-4 w-4 mr-2" />
              Confirm Location
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default LocationPickerModal;
