import React from 'react';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { X, Mail, Phone, Calendar, Shield, Clock, BadgeCheck } from 'lucide-react';
import { User } from '@/types/user';

interface UserDetailsModalProps {
  user: User;
  onClose: () => void;
}

const UserDetailsModal: React.FC<UserDetailsModalProps> = ({ user, onClose }) => {
  const getStatusBadge = (status: User['status']) => {
    const statusStyles: { [key in User['status']]: string } = {
      confirmed: 'bg-green-100 text-green-800',
      pending: 'bg-yellow-100 text-yellow-800',
      suspended: 'bg-red-100 text-red-800',
    };
    const style = statusStyles[status] || 'bg-gray-100 text-gray-800';
    return (
      <span className={`px-2 py-1 rounded-full text-xs font-medium ${style}`}>
        {status.charAt(0).toUpperCase() + status.slice(1)}
      </span>
    );
  };

  const InfoRow: React.FC<{ icon: React.ReactNode; label: string; value: React.ReactNode }> = ({
    icon,
    label,
    value,
  }) => (
    <div className="flex items-start">
      <div className="flex-shrink-0 w-6 h-6 text-gray-500">{icon}</div>
      <div className="ml-3">
        <p className="text-sm font-medium text-gray-900">{label}</p>
        <p className="text-sm text-gray-600">{value}</p>
      </div>
    </div>
  );

  return (
    <div
      className="fixed inset-0 bg-black/60 flex justify-center items-center z-50 animate-in fade-in"
      onClick={onClose}
    >
      <Card
        className="w-full max-w-lg m-4 animate-in zoom-in-95"
        onClick={(e) => e.stopPropagation()}
      >
        <CardHeader className="flex flex-row items-start justify-between">
          <div>
            <CardTitle className="text-xl">
              {user.first_name} {user.last_name}
            </CardTitle>
            <CardDescription>@{user.username}</CardDescription>
          </div>
          <Button variant="outline" size="sm" onClick={onClose}>
            <X className="h-5 w-5" />
          </Button>
        </CardHeader>
        <CardContent>
          <div className="space-y-6">
            <div className="space-y-4">
              <h3 className="text-sm font-semibold text-gray-500 uppercase">Contact Information</h3>
              <InfoRow
                icon={<Mail />}
                label="Email"
                value={
                  <a href={`mailto:${user.email}`} className="text-blue-600 hover:underline">
                    {user.email}
                  </a>
                }
              />
              <InfoRow icon={<Phone />} label="Phone" value={user.phone || 'Not provided'} />
            </div>
            <div className="space-y-4">
              <h3 className="text-sm font-semibold text-gray-500 uppercase">Account Details</h3>
              <InfoRow icon={<BadgeCheck />} label="Status" value={getStatusBadge(user.status)} />
              <InfoRow
                icon={<Shield />}
                label="Role"
                value={user.role.charAt(0).toUpperCase() + user.role.slice(1)}
              />
              <InfoRow
                icon={<Calendar />}
                label="Date of Birth"
                value={
                  user.date_of_birth
                    ? new Date(user.date_of_birth).toLocaleDateString()
                    : 'Not provided'
                }
              />
              <InfoRow
                icon={<Clock />}
                label="Member Since"
                value={new Date(user.created_at).toLocaleString()}
              />
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default UserDetailsModal;
