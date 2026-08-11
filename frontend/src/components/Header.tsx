import { Button } from '@/components/ui/button';
import { Search, ShoppingCart, LayoutGrid, ChevronDown, LogOut } from 'lucide-react';
import React, { useState, useRef, useEffect } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { useAuth } from '@/contexts/AuthContext';
import { useCart } from '@/contexts/CartContext';
import EbayLogo from './common/EBayLogo';
import { Category } from '@/types/category';
import { toast } from 'sonner';
import NotificationBell from './notifications/NotificationBell';

const Header: React.FC = () => {
  const navigate = useNavigate();
  const { user, isAuthenticated, logout } = useAuth();
  const { itemCount } = useCart();
  const [watchOpen, setWatchOpen] = useState(false);
  const [categoryMenuOpen, setCategoryMenuOpen] = useState(false);
  const watchRef = useRef<HTMLDivElement | null>(null);
  const categoryRef = useRef<HTMLDivElement | null>(null);
  const [categories, setCategories] = useState<Category[]>([]);
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('all');

  useEffect(() => {
    const fetchCategories = async () => {
      try {
        const response = await fetch('/api/categories');
        if (!response.ok) throw new Error('Failed to fetch categories');
        const data = await response.json();
        setCategories(data);
      } catch (error) {
        toast.error('Could not load categories for header.');
      }
    };
    fetchCategories();
  }, []);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (watchRef.current && !watchRef.current.contains(e.target as Node)) {
        setWatchOpen(false);
      }
      if (categoryRef.current && !categoryRef.current.contains(e.target as Node)) {
        setCategoryMenuOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const mainCategories = categories.slice(0, 8);
  // const mainCategoriesWithMarketplace = [{ id: 0, name: 'Marketplace' }, ...mainCategories];

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    const params = new URLSearchParams();

    if (searchTerm.trim()) {
      params.set('q', searchTerm.trim());
    }

    if (selectedCategory !== 'all' && selectedCategory !== '') {
      params.set('category', selectedCategory);
    }

    const queryString = params.toString();
    navigate(`/marketplace${queryString ? `?${queryString}` : ''}`);
  };

  return (
    <header className="bg-white w-full border-b text-sm">
      {/* 1. Top Utility Bar */}
      <nav className="h-8 border-b bg-gray-50 text-xs text-gray-600">
        <div className="max-w-screen-xl mx-auto px-4 sm:px-6 lg:px-8 flex justify-between items-center h-full">
          <div className="flex items-center space-x-6">
            {!isAuthenticated ? (
              <p>
                Hi!{' '}
                <a href="/login" className="text-ebay-blue underline">
                  Sign in
                </a>{' '}
                or{' '}
                <a href="/signup" className="text-ebay-blue underline">
                  register
                </a>
              </p>
            ) : (
              <p>Hi, {user?.first_name || user?.username}!</p>
            )}
            <Link to="/help" className="text-black sm:inline hover:text-black hover:transform-none">
              Help & Contact
            </Link>
          </div>
          <div className="flex items-center space-x-2">
            {user?.role === 'admin' && (
              <Link
                to="/admin"
                className="flex items-center gap-1 text-ebay-red hover:text-ebay-red hover:underline text-xs px-1 py-1"
                title="Admin Panel"
              >
                <LayoutGrid className="h-4 w-4" />
                <span>Admin</span>
              </Link>
            )}
            <Link
              to="/sell"
              className="text-black hover:text-gray-600 text-xs transition-colors px-1 py-1"
            >
              Sell
            </Link>
            <div className="relative" ref={watchRef}>
              {!isAuthenticated && watchOpen && (
                <div className="absolute right-0 mt-2 w-72 bg-white border shadow-lg rounded p-4 z-50">
                  <p className="text-base">
                    Please <a href="/login">sign in</a> to view items you are watching.
                  </p>
                </div>
              )}
            </div>
            <Link
              to="/user/profile"
              className="text-black hover:text-gray-600 text-xs transition-colors px-1 py-1"
            >
              My eBay
            </Link>
            {isAuthenticated && (
              <Link
                to="/messaging"
                className="text-black hover:text-gray-600 text-xs transition-colors px-1 py-1"
              >
                Messages
              </Link>
            )}
            <NotificationBell />
            <Link
              to="/cart"
              className="text-black hover:text-gray-600 relative px-1 py-1 transition-colors"
            >
              <ShoppingCart className="h-4 w-4" />
              {itemCount > 0 && (
                <span className="absolute -top-1 -right-1 bg-red-500 text-white text-xs rounded-full h-5 w-5 flex items-center justify-center font-medium">
                  {itemCount > 99 ? '99+' : itemCount}
                </span>
              )}
            </Link>
            {isAuthenticated && (
              <button
                onClick={logout}
                className="text-red-600 hover:text-red-700 bg-transparent border-none cursor-pointer text-xs flex items-center gap-1 px-1 py-1 transition-colors"
              >
                <LogOut className="h-4 w-4" />
                <span>Logout</span>
              </button>
            )}
          </div>
        </div>
      </nav>

      {/* 2. Main Header with Logo and Search */}
      <div className="py-3">
        <div className="max-w-screen-xl mx-auto px-4 sm:px-6 lg:px-8 flex items-center justify-between gap-6">
          <EbayLogo />

          <div className="hidden lg:block relative" ref={categoryRef}>
            <button
              className="text-sm flex items-center bg-white text-gray-700 font-semibold px-3 py-2 rounded-full border hover:border-gray-600 active:border-gray-600 border-gray-600"
              onClick={() => setCategoryMenuOpen(!categoryMenuOpen)}
            >
              Shop by category
              <ChevronDown
                className={`w-4 h-4 ml-1 transition-transform duration-200 ${
                  categoryMenuOpen ? 'rotate-180' : ''
                }`}
              />
            </button>

            {categoryMenuOpen && (
              <div className="absolute top-full left-0 mt-2 w-80 bg-white border shadow-xl rounded-lg p-4 z-50">
                <h3 className="font-bold text-gray-900 text-base pb-2 mb-2 border-b">
                  Top Categories
                </h3>
                <ul className="space-y-2">
                  {categories.map((category) => (
                    <li key={category.id}>
                      <Link
                        to={`/marketplace?category=${category.id}`}
                        className="text-sm text-gray-600 hover:text-ebay-blue hover:underline block"
                        onClick={() => setCategoryMenuOpen(false)}
                      >
                        {category.name}
                      </Link>
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>

          <div className="flex-1 max-w-4xl">
            <form className="flex items-center w-full gap-3" onSubmit={handleSearch}>
              <div className="flex-grow flex items-center border border-gray-300 rounded-full focus-within:border-ebay-blue focus-within:ring-2 focus-within:ring-ebay-blue/20 h-11 bg-white shadow-sm">
                <div className="pl-4 pr-3">
                  <Search className="h-5 w-5 text-gray-500" />
                </div>
                <input
                  type="text"
                  placeholder="Search for anything..."
                  className="flex-1 h-full text-base bg-transparent border-none focus:outline-none focus:ring-0 placeholder:text-gray-500"
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                />
                <div className="border-l border-gray-300 h-8 mx-3"></div>
                <select
                  className="text-sm text-gray-600 pr-4 pl-3 bg-transparent hover:bg-transparent focus:outline-none cursor-pointer appearance-none min-w-[120px]"
                  value={selectedCategory}
                  onChange={(e) => setSelectedCategory(e.target.value)}
                >
                  <option value="all">All Categories</option>
                  {categories.map((cat) => (
                    <option key={cat.id} value={cat.id.toString()}>
                      {cat.name}
                    </option>
                  ))}
                </select>
              </div>
              <Button
                type="submit"
                className="h-11 px-8 bg-ebay-blue hover:bg-blue-700 rounded-full text-base font-semibold shadow-sm"
              >
                Search
              </Button>
            </form>
          </div>
        </div>
      </div>
    </header>
  );
};

export default Header;
