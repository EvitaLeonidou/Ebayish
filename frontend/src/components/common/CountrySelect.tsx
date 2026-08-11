import React, { useState, useEffect } from 'react';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { toast } from 'sonner';

interface Country {
  name: {
    common: string;
  };
}

interface CountrySelectProps {
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
}

const CountrySelect: React.FC<CountrySelectProps> = ({ value, onChange, disabled }) => {
  const [countries, setCountries] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchCountries = async () => {
      try {
        const response = await fetch('https://restcountries.com/v3.1/all?fields=name');
        if (!response.ok) {
          throw new Error('Failed to fetch country data');
        }
        const data: Country[] = await response.json();
        const countryNames = data
          .map((country) => country.name.common)
          .sort((a, b) => a.localeCompare(b)); // Sort countries alphabetically
        setCountries(countryNames);
      } catch (error) {
        console.error('Error fetching countries:', error);
        toast.error('Could not load the list of countries.');
        // Fallback to a basic list in case of API failure
        setCountries(['United States', 'Canada', 'United Kingdom', 'Australia']);
      } finally {
        setIsLoading(false);
      }
    };

    fetchCountries();
  }, []);

  return (
    <Select value={value} onValueChange={onChange} disabled={disabled || isLoading} required>
      <SelectTrigger>
        <SelectValue placeholder={isLoading ? 'Loading countries...' : 'Select a country'} />
      </SelectTrigger>
      <SelectContent>
        {countries.map((countryName) => (
          <SelectItem key={countryName} value={countryName}>
            {countryName}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
};

export default CountrySelect;
