import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import EbayLogo from '@/components/common/EBayLogo';
import { Fingerprint, Lock, User as UserIcon } from 'lucide-react';
import React, { useState, useEffect } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { useAuth } from '@/contexts/AuthContext';
import Microsoft from '@/components/common/Microsoft';
import Google from '@/components/common/Google';
import GitHub from '@/components/common/Github';
import Facebook from '@/components/common/Fabecook';
import Instagram from '@/components/common/Instagram';
import Twitter from '@/components/common/Twitter';
import Tiktok from '@/components/common/Tiktok';
import Reddit from '@/components/common/Reddit';
import Dropbox from '@/components/common/Dropbox';
import Apple from '@/components/common/Apple';
import Spotify from '@/components/common/Spotify';
import LinkedIn from '@/components/common/LinkedIn';
import Amazon from '@/components/common/Amazon';
import Twitch from '@/components/common/Twitch';
import Adobe from '@/components/common/Adobe';
import Telegram from '@/components/common/Telegram';
import Discord from '@/components/common/Discord';
import Binance from '@/components/common/Binance';
import Notion from '@/components/common/Notion';
import Steam from '@/components/common/Steam';
import Ronin from '@/components/common/Ronin';
import VSCode from '@/components/common/VSCode';
import PayPal from '@/components/common/PayPal';
import StackOverflow from '@/components/common/StackOverflow';
import Youtube from '@/components/common/Youtube';
import Pinterest from '@/components/common/Pinterest';
import Caution from '@/components/common/Caution';
import OnlyFans from '@/components/common/OnlyFans';
import Roblox from '@/components/common/Roblox';
import Shopee from '@/components/common/Shopee';
import Potato from '@/components/common/Potato';
import Mom from '@/components/common/Mom';
import Age from '@/components/common/Age';
import Settings from '@/components/common/Settings';
import Pdf from '@/components/common/Pdf';
import Calculator from '@/components/common/Calculator';
import Address from '@/components/common/Address';
import Chicken from '@/components/common/Chicken';
import Visa from '@/components/common/Visa';
import Id from '@/components/common/Id';
import Means from '@/components/common/Means';

const Login: React.FC = () => {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [showSpecialLogins, setShowSpecialLogins] = useState(false);
  const navigate = useNavigate();
  const location = useLocation();
  const { login } = useAuth();
  const message = location.state?.message;

  const [keySequence, setKeySequence] = useState('');
  const secretCode = 'secret';

  // Easter egg inspo: https://www.reddit.com/r/webdev/comments/1nilgyn/i_just_implemented_oauth_in_my_app_is_this_enough/

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const newSequence = (keySequence + e.key.toLowerCase()).slice(-secretCode.length);
      setKeySequence(newSequence);

      if (newSequence === secretCode) {
        setShowSpecialLogins(true);
      }
    };

    window.addEventListener('keydown', handleKeyDown);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [keySequence]);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setIsLoading(true);
    try {
      await login(username, password);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'An unknown error occurred';
      if (errorMessage.includes('401') || errorMessage.includes('Invalid')) {
        setError('Invalid username or password.');
      } else {
        setError('An unexpected error occurred. Please try again.');
      }
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen w-full flex-col items-center bg-gray-50 px-4 py-12">
      <div className={`w-full ${showSpecialLogins ? 'max-w-lg' : 'max-w-md'}`}>
        <div className="mb-8 flex justify-center">
          <EbayLogo />
        </div>

        <div className="rounded-lg border bg-white p-8 shadow-md">
          <div className="text-center mb-8">
            <h2 className="text-3xl font-bold text-gray-900 mb-2">Welcome back!</h2>
            <p className="text-gray-600">
              Sign in to your eBayish account to continue shopping and selling
            </p>
          </div>

          <form onSubmit={handleLogin} className="space-y-6">
            {message && !error && (
              <div className="rounded-md bg-green-50 border border-green-200 p-4">
                <div className="flex">
                  <div className="text-sm text-green-800">{message}</div>
                </div>
              </div>
            )}
            {error && (
              <div className="rounded-md bg-red-50 border border-red-200 p-4">
                <div className="flex">
                  <div className="text-sm text-red-800">{error}</div>
                </div>
              </div>
            )}

            <div className="space-y-2">
              <Label htmlFor="username" className="text-gray-700 font-medium">
                Username
              </Label>
              <div className="relative">
                <UserIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-gray-400" />
                <Input
                  id="username"
                  type="text"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  required
                  placeholder="Enter your username"
                  className="pl-10 h-12 border-gray-300 focus:border-blue-500 focus:ring-blue-500"
                />
              </div>
            </div>

            <div className="space-y-2">
              <Label htmlFor="password" className="text-gray-700 font-medium">
                Password
              </Label>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-gray-400" />
                <Input
                  id="password"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                  placeholder="Enter your password"
                  className="pl-10 h-12 border-gray-300 focus:border-blue-500 focus:ring-blue-500"
                />
              </div>
            </div>

            <div className="flex justify-end">
              <a href="#" className="text-sm text-blue-600 hover:text-blue-700">
                Forgot password?
              </a>
            </div>

            <Button
              type="submit"
              className="w-full h-12 bg-blue-600 hover:bg-blue-700 text-white font-medium text-lg"
              disabled={isLoading}
            >
              {isLoading ? 'Signing in...' : 'Sign In'}
            </Button>
          </form>

          {showSpecialLogins && (
            <div className="mt-6 animate-in fade-in duration-500">
              <div className="grid grid-cols-2 gap-4">
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Microsoft className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Microsoft</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Google className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Google</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <GitHub className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with GitHub</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Facebook className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Facebook</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Instagram className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Instagram</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Twitter className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with X</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Tiktok className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Tiktok</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Reddit className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Reddit</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Dropbox className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Dropbox</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Apple className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Apple</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Spotify className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Spotify</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <LinkedIn className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with LinkedIn</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Amazon className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Amazon</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Twitch className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Twitch</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Adobe className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Adobe</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Telegram className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Telegram</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Discord className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Discord</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Binance className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Binance</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Notion className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Notion</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Steam className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Steam</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Ronin className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Ronin</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <VSCode className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with VS Code</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <PayPal className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with PayPal</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <StackOverflow className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Stack Overflow</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Youtube className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Youtube</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Pinterest className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Pinterest</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Caution className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Caution</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <OnlyFans className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with OnlyFans</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Roblox className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Roblox</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Shopee className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Shopee</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Fingerprint className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Fingerprint</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Potato className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with a Potato</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Chicken className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with a Chicken</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Settings className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Settings</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Age className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with your age</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Mom className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with your mom</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Pdf className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with PDF</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Calculator className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Calculator</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <img src="/gkiz.webp" alt="Gkiz" className="h-10 w-8" />
                  <span className="text-sm font-medium">Login with Gkiz</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <img src="/prondo.webp" alt="Gkiz" className="h-10 w-10" />
                  <span className="text-sm font-medium">Login with Rondo</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Address className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with your address</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Visa className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with Credit Card</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Id className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with National ID</span>
                </Button>
                <Button
                  variant="outline"
                  className="w-full h-12 flex items-center justify-center gap-2"
                >
                  <Means className="h-6 w-6" />
                  <span className="text-sm font-medium">Login with other means</span>
                </Button>
              </div>
            </div>
          )}

          <div className="mt-8 mb-6">
            <div className="relative">
              <div className="absolute inset-0 flex items-center">
                <div className="w-full border-t border-gray-300" />
              </div>
              <div className="relative flex justify-center text-sm">
                <span className="px-2 bg-white text-gray-500">New to eBayish?</span>
              </div>
            </div>
          </div>

          <div className="text-center">
            <Button
              type="button"
              variant="outline"
              className="w-full h-12 border-2 border-orange-500 text-orange-500 hover:bg-orange-50 font-medium"
              onClick={() => navigate('/signup')}
            >
              Create Account
            </Button>
          </div>
        </div>
        <p className="mt-6 text-center text-sm text-gray-600">
          <a href="/" className="font-medium text-blue-600 hover:text-blue-700">
            &larr; Back to homepage
          </a>
        </p>
      </div>
    </div>
  );
};

export default Login;
