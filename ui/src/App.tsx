import { useEffect } from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { useAuthStore } from './stores/auth';
import Login from './pages/Login';
import Repositories from './pages/Repositories';
import Repository from './pages/Repository';
import { Button } from './components/ui/button';
import { LogOut, User } from 'lucide-react';

function App() {
  const { isAuthenticated, isLoading, user, logout, initialize } = useAuthStore();

  useEffect(() => {
    initialize();
  }, [initialize]);

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-lg">Loading...</div>
      </div>
    );
  }

  if (!isAuthenticated) {
    return <Login />;
  }

  return (
    <div className="min-h-screen bg-background">
      <header className="border-b">
        <div className="container mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <h1 className="text-xl font-bold">Blacklake</h1>
              <nav className="flex space-x-4">
                <a href="/repos" className="text-sm hover:text-primary">
                  Repositories
                </a>
              </nav>
            </div>
            <div className="flex items-center space-x-4">
              <div className="flex items-center space-x-2 text-sm text-muted-foreground">
                <User className="w-4 h-4" />
                <span>{user?.name || user?.email || user?.sub}</span>
              </div>
              <Button variant="outline" size="sm" onClick={logout}>
                <LogOut className="w-4 h-4 mr-2" />
                Sign Out
              </Button>
            </div>
          </div>
        </div>
      </header>

      <main>
        <Routes>
          <Route path="/" element={<Navigate to="/repos" replace />} />
          <Route path="/repos" element={<Repositories />} />
          <Route path="/repos/:repo" element={<Repository />} />
          <Route path="/callback" element={<div>Processing login...</div>} />
        </Routes>
      </main>
    </div>
  );
}

export default App;
