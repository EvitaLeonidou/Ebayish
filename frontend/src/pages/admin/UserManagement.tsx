import React, { useState, useEffect, useMemo } from 'react';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Search,
  UserCheck,
  UserX,
  CheckCircle,
  XCircle,
  Loader2,
  MessageCircle,
} from 'lucide-react';
import { User } from '@/types/user';
import { toast } from 'sonner';
import UserDetailsModal from '@/components/admin/UserDetailsModal';
import { authFetch } from '@/utils/auth-fetch';
import { useNavigate } from 'react-router-dom';

type FilterStatus = 'all' | 'pending' | 'confirmed' | 'suspended';

const UserManagement: React.FC = () => {
  const navigate = useNavigate();
  const [users, setUsers] = useState<User[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterStatus, setFilterStatus] = useState<FilterStatus>('all');
  const [selectedUser, setSelectedUser] = useState<User | null>(null);

  useEffect(() => {
    const fetchUsers = async () => {
      try {
        const response = await authFetch('/api/admin/users');
        if (!response.ok) {
          throw new Error('Failed to fetch users.');
        }
        const data = await response.json();
        setUsers(data);
      } catch (error) {
        toast.error('Could not load user data.');
      } finally {
        setIsLoading(false);
      }
    };
    fetchUsers();
  }, []);

  const handleVerify = async (userId: string) => {
    try {
      const response = await authFetch(`/api/admin/users/${userId}/verify`, { method: 'PUT' });
      if (!response.ok) throw new Error('Verification failed.');
      setUsers(users.map((u) => (u.id === userId ? { ...u, status: 'confirmed' } : u)));
      toast.success('User has been verified.');
    } catch (error) {
      toast.error('Failed to verify user.');
    }
  };

  const handleSuspend = async (userId: string) => {
    if (window.confirm('Are you sure you want to suspend this user?')) {
      try {
        const response = await authFetch(`/api/admin/users/${userId}/suspend`, { method: 'PUT' });
        if (!response.ok) throw new Error('Suspension failed.');
        setUsers(users.map((u) => (u.id === userId ? { ...u, status: 'suspended' } : u)));
        toast.success('User has been suspended.');
      } catch (error) {
        toast.error('Failed to suspend user.');
      }
    }
  };

  const handleActivate = async (userId: string) => {
    if (window.confirm('Are you sure you want to reactivate this user?')) {
      try {
        const response = await authFetch(`/api/admin/users/${userId}/activate`, { method: 'PUT' });
        if (!response.ok) throw new Error('Activation failed.');
        setUsers(users.map((u) => (u.id === userId ? { ...u, status: 'confirmed' } : u)));
        toast.success('User has been reactivated.');
      } catch (error) {
        toast.error('Failed to reactivate user.');
      }
    }
  };

  const handleReject = async (userId: string) => {
    if (window.confirm('Are you sure you want to reject and delete this user?')) {
      try {
        const response = await fetch(`/api/users/${userId}`, { method: 'DELETE' });
        if (!response.ok) throw new Error('Rejection failed.');
        setUsers(users.filter((u) => u.id !== userId));
        toast.success('User has been rejected and removed.');
      } catch (error) {
        toast.error('Failed to reject user.');
      }
    }
  };

  const handleMessage = (user: User) => {
    navigate(`/messaging/${user.id}`);
  };

  const filteredUsers = useMemo(() => {
    return users
      .filter((user) => {
        if (filterStatus === 'all') return true;
        return user.status === filterStatus;
      })
      .filter((user) => {
        const search = searchTerm.toLowerCase();
        return (
          user.username.toLowerCase().includes(search) ||
          user.email.toLowerCase().includes(search) ||
          `${user.first_name} ${user.last_name}`.toLowerCase().includes(search)
        );
      });
  }, [users, filterStatus, searchTerm]);

  const getStatusBadge = (status: User['status'] | undefined) => {
    if (!status) {
      return (
        <span className="px-2 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
          Unknown
        </span>
      );
    }

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

  const stats = useMemo(
    () => ({
      total: users.length,
      pending: users.filter((u) => u.status === 'pending').length,
      confirmed: users.filter((u) => u.status === 'confirmed').length,
      suspended: users.filter((u) => u.status === 'suspended').length,
    }),
    [users]
  );

  if (isLoading) {
    return (
      <div className="flex justify-center items-center h-64">
        <Loader2 className="h-12 w-12 animate-spin" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold text-gray-900">User Management</h1>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardContent className="p-4 text-center">
            <p className="text-2xl font-bold">{stats.total}</p>
            <p className="text-sm text-gray-600">Total Users</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4 text-center">
            <p className="text-2xl font-bold text-yellow-600">{stats.pending}</p>
            <p className="text-sm text-gray-600">Pending</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4 text-center">
            <p className="text-2xl font-bold text-green-600">{stats.confirmed}</p>
            <p className="text-sm text-gray-600">Confirmed</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4 text-center">
            <p className="text-2xl font-bold text-red-600">{stats.suspended}</p>
            <p className="text-sm text-gray-600">Suspended</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader className="p-4">
          <div className="flex flex-col sm:flex-row justify-between items-center gap-4">
            <div className="flex flex-wrap items-center gap-2">
              {(['all', 'pending', 'confirmed', 'suspended'] as const).map((status) => {
                const isActive = filterStatus === status;

                const inactiveClasses = 'border-[#2a2a2a] text-[#2a2a2a] bg-white hover:bg-gray-50';

                let activeClasses = '';
                if (isActive) {
                  switch (status) {
                    case 'pending':
                      activeClasses =
                        'border-yellow-500 text-yellow-600 bg-yellow-50 hover:bg-yellow-100';
                      break;
                    case 'confirmed':
                      activeClasses =
                        'border-green-500 text-green-600 bg-green-50 hover:bg-green-100';
                      break;
                    case 'suspended':
                      activeClasses = 'border-red-500 text-red-600 bg-red-50 hover:bg-red-100';
                      break;
                    case 'all':
                      activeClasses =
                        'border-[#2a2a2a] text-[#2a2a2a] bg-gray-100 hover:bg-gray-200';
                      break;
                  }
                }

                return (
                  <Button
                    key={status}
                    onClick={() => setFilterStatus(status)}
                    variant="outline"
                    className={isActive ? activeClasses : inactiveClasses}
                  >
                    {status.charAt(0).toUpperCase() + status.slice(1)}
                  </Button>
                );
              })}
            </div>

            <div className="relative w-full sm:w-auto sm:max-w-xs">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
              <Input
                placeholder="Search users..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="pl-10 h-10"
              />
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b">
                  <th className="text-left py-3 px-4">User</th>
                  <th className="text-left py-3 px-4">Contact</th>
                  <th className="text-left py-3 px-4">Status</th>
                  <th className="text-left py-3 px-4">Joined</th>
                  <th className="text-left py-3 px-4">Actions</th>
                </tr>
              </thead>
              <tbody>
                {filteredUsers.map((user) => (
                  <tr key={user.id} className="border-b hover:bg-gray-50">
                    <td className="py-4 px-4">
                      <p
                        className="font-medium text-blue-600 hover:underline cursor-pointer"
                        onClick={() => setSelectedUser(user)}
                      >
                        {user.first_name} {user.last_name}
                      </p>
                      <p className="text-gray-600 text-xs">@{user.username}</p>
                    </td>
                    <td className="py-4 px-4">
                      <p>{user.email}</p>
                      <p className="text-gray-600 text-xs">{user.phone}</p>
                    </td>
                    <td className="py-4 px-4">{getStatusBadge(user.status)}</td>
                    <td className="py-4 px-4">{new Date(user.created_at).toLocaleDateString()}</td>
                    <td className="py-4 px-4">
                      {user.status === 'pending' ? (
                        <div className="flex items-center gap-2">
                          <Button
                            size="sm"
                            className="bg-green-500 hover:bg-green-600"
                            onClick={() => handleVerify(user.id)}
                          >
                            <CheckCircle className="h-4 w-4 mr-1" /> Verify
                          </Button>
                          <Button
                            size="sm"
                            variant="destructive"
                            onClick={() => handleReject(user.id)}
                          >
                            <XCircle className="h-4 w-4 mr-1" /> Reject
                          </Button>
                        </div>
                      ) : (
                        <div className="flex items-center gap-2">
                          {user.status === 'confirmed' && (
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => handleMessage(user)}
                              className="text-blue-600 hover:text-blue-700 hover:bg-blue-50"
                            >
                              <MessageCircle className="h-4 w-4" />
                            </Button>
                          )}
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => {
                              if (user.status === 'suspended') {
                                handleActivate(user.id);
                              } else {
                                handleSuspend(user.id);
                              }
                            }}
                          >
                            {user.status === 'suspended' ? (
                              <UserCheck className="h-4 w-4" />
                            ) : (
                              <UserX className="h-4 w-4" />
                            )}
                          </Button>
                        </div>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          {filteredUsers.length === 0 && (
            <div className="text-center py-8">
              <p className="text-gray-500">No users found.</p>
            </div>
          )}
        </CardContent>
      </Card>

      {selectedUser && (
        <UserDetailsModal user={selectedUser} onClose={() => setSelectedUser(null)} />
      )}
    </div>
  );
};

export default UserManagement;
