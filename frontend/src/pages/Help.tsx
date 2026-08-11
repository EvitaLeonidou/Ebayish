import React, { useState } from 'react';
import { Card } from '@/components/ui/card';
// import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { LifeBuoy, MessageSquare, Mail, Phone, ChevronDown } from 'lucide-react';

const Help: React.FC = () => {
  const [openFaq, setOpenFaq] = useState<number | null>(0); // Keep the first FAQ open by default

  const faqItems = [
    {
      question: 'How do I place a bid?',
      answer:
        'To place a bid, navigate to the item page you are interested in. In the bidding interface, enter your maximum bid amount in the input field and click "Place Bid". The system will automatically bid for you up to your maximum amount.',
    },
    {
      question: 'How does shipping work?',
      answer:
        'Shipping costs and options are set by the seller and can be found on the item detail page. After winning an auction or purchasing an item, you will proceed to checkout where the final shipping costs will be calculated based on your address.',
    },
    {
      question: 'Can I cancel a bid?',
      answer:
        'Generally, bids are binding. However, under specific circumstances, you may be able to retract a bid. Please check our official bidding policy or contact customer support for more details.',
    },
    {
      question: 'How do I sell an item?',
      answer:
        'To sell an item, click the "Sell" button in the header. You will be guided through a form to create your listing, including adding photos, a description, setting a price, and choosing shipping options.',
    },
  ];

  const handleFaqClick = (index: number) => {
    setOpenFaq(openFaq === index ? null : index);
  };

  return (
    <div className="container mx-auto p-4 md:p-6 space-y-12">
      {/* Search and Hero Section */}
      <div className="text-center py-8">
        <LifeBuoy className="mx-auto h-16 w-16 text-blue-600" />
        <h1 className="mt-4 text-4xl font-bold text-gray-900">Help & Contact Center</h1>
        <p className="mt-2 text-lg text-gray-600">We're here to help you with any questions.</p>
      </div>

      {/* Main Content Area */}
      <div className="max-w-4xl mx-auto">
        <h2 className="text-2xl font-bold text-gray-900 mb-6 text-center">
          Frequently Asked Questions
        </h2>
        <div className="space-y-4">
          {faqItems.map((item, index) => (
            <Card key={index} className="overflow-hidden transition-all duration-300">
              <div
                onClick={() => handleFaqClick(index)}
                className="flex justify-between items-center p-6 cursor-pointer hover:bg-gray-50"
              >
                <h3 className="font-semibold text-lg text-gray-800">{item.question}</h3>
                <ChevronDown
                  className={`h-5 w-5 text-gray-500 transition-transform duration-200 ${
                    openFaq === index ? 'rotate-180' : ''
                  }`}
                />
              </div>
              {openFaq === index && (
                <div className="px-6 pb-6 pt-0 text-gray-700 animate-in fade-in-0 slide-in-from-top-2 duration-300">
                  <p className="text-base leading-relaxed">{item.answer}</p>
                </div>
              )}
            </Card>
          ))}
        </div>
      </div>

      {/* Redesigned Contact Us Section */}
      <div>
        <h2 className="text-2xl font-bold text-gray-900 text-center mb-6">Still Need Help?</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8 text-center">
          {/* Live Chat Card */}
          <div className="bg-white p-8 rounded-xl shadow-md hover:shadow-xl hover:-translate-y-1 transition-all duration-300">
            <div className="bg-blue-100 rounded-full h-16 w-16 mx-auto flex items-center justify-center">
              <MessageSquare className="h-8 w-8 text-blue-600" />
            </div>
            <h3 className="mt-6 text-xl font-semibold">Live Chat</h3>
            <p className="text-gray-500 mt-2">Get instant answers from our support team.</p>
            <Button variant="secondary" className="mt-6">
              Chat Now
            </Button>
          </div>
          {/* Email Card */}
          <div className="bg-white p-8 rounded-xl shadow-md hover:shadow-xl hover:-translate-y-1 transition-all duration-300">
            <div className="bg-green-100 rounded-full h-16 w-16 mx-auto flex items-center justify-center">
              <Mail className="h-8 w-8 text-green-600" />
            </div>
            <h3 className="mt-6 text-xl font-semibold">Email Us</h3>
            <p className="text-gray-500 mt-2">We'll get back to you within 24 hours.</p>
            <Button variant="secondary" className="mt-6">
              Send Email
            </Button>
          </div>
          {/* Phone Card */}
          <div className="bg-white p-8 rounded-xl shadow-md hover:shadow-xl hover:-translate-y-1 transition-all duration-300">
            <div className="bg-purple-100 rounded-full h-16 w-16 mx-auto flex items-center justify-center">
              <Phone className="h-8 w-8 text-purple-600" />
            </div>
            <h3 className="mt-6 text-xl font-semibold">Call Us</h3>
            <p className="text-gray-500 mt-2">Our lines are open 24/7 for urgent issues.</p>
            <Button variant="secondary" className="mt-6">
              View Number
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Help;
