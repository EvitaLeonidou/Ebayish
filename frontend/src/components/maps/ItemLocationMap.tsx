import React from 'react';
import { MapContainer, TileLayer, Marker, Popup } from 'react-leaflet';
import 'leaflet/dist/leaflet.css';
import L from 'leaflet';

// Fix for default markers not showing in React-Leaflet
delete (L.Icon.Default.prototype as any)._getIconUrl;
L.Icon.Default.mergeOptions({
  iconRetinaUrl: 'https://cdnjs.cloudflare.com/ajax/libs/leaflet/1.7.1/images/marker-icon-2x.png',
  iconUrl: 'https://cdnjs.cloudflare.com/ajax/libs/leaflet/1.7.1/images/marker-icon.png',
  shadowUrl: 'https://cdnjs.cloudflare.com/ajax/libs/leaflet/1.7.1/images/marker-shadow.png',
});

interface ItemLocationMapProps {
  latitude: number;
  longitude: number;
  itemName: string;
  location?: string;
}

const ItemLocationMap: React.FC<ItemLocationMapProps> = ({
  latitude,
  longitude,
  itemName,
  location,
}) => {
  return (
    <div className="bg-white p-6 rounded-lg shadow-md">
      <h2 className="text-xl font-bold text-gray-900 mb-4 border-b pb-2">Item Location</h2>
      {location && (
        <p className="text-gray-600 mb-4">
          <strong>Location:</strong> {location}
        </p>
      )}
      <div className="h-64 w-full rounded-lg overflow-hidden border">
        <MapContainer
          center={[latitude, longitude]}
          zoom={13}
          style={{ height: '100%', width: '100%' }}
        >
          <TileLayer
            url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
            attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
          />
          <Marker position={[latitude, longitude]}>
            <Popup>
              <div className="text-center">
                <strong>{itemName}</strong>
                {location && <p className="text-sm text-gray-600">{location}</p>}
              </div>
            </Popup>
          </Marker>
        </MapContainer>
      </div>
    </div>
  );
};

export default ItemLocationMap;
