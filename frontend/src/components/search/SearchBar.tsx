import React, { useState, useEffect } from 'react';
import { Input } from '@/components/ui/input';
import { Search } from 'lucide-react';
import { useDebounce } from '@/hooks/useDebounce';

interface SearchBarProps {
  initialValue: string;
  onSearch: (term: string) => void;
}

const SearchBar: React.FC<SearchBarProps> = ({ initialValue, onSearch }) => {
  const [searchTerm, setSearchTerm] = useState(initialValue);
  const debouncedSearchTerm = useDebounce(searchTerm, 500); // 500ms delay

  useEffect(() => {
    // Only call onSearch if the debounced term is different from the initial prop
    // to avoid triggering a search on component mount.
    if (debouncedSearchTerm !== initialValue) {
      onSearch(debouncedSearchTerm);
    }
  }, [debouncedSearchTerm, onSearch]);

  // Update local state when initialValue from URL changes
  useEffect(() => {
    setSearchTerm(initialValue);
  }, [initialValue]);

  return (
    <div className="relative mb-6">
      <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-gray-400" />
      <Input
        type="text"
        placeholder="Search for anything..."
        className="pl-10 h-12 w-full"
        value={searchTerm}
        onChange={(e) => setSearchTerm(e.target.value)}
      />
    </div>
  );
};

export default SearchBar;
