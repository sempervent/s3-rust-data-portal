import React from 'react';
import { QueryClientProvider } from '@tanstack/react-query';
import { AuthProvider } from './stores/auth';
import { AppRoutes } from './app/routes';
import { Toaster } from './components/ui/toasts';
import { useAppStore } from './app/store';
import './app/theme.css';

// Import query client
import { queryClient } from './app/query';

function App() {
  const { toasts, removeToast } = useAppStore();

  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <div className="min-h-screen bg-background">
          <AppRoutes />
          <Toaster 
            toasts={toasts} 
            onRemove={removeToast}
            position="top-right"
          />
        </div>
      </AuthProvider>
    </QueryClientProvider>
  );
}

export default App;
