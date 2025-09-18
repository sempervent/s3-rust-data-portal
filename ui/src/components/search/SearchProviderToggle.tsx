// BlackLake Search Provider Toggle
// Week 4: Switch between Postgres and OpenSearch backends

import React, { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { 
  Database, 
  Globe, 
  Settings, 
  RefreshCw, 
  CheckCircle, 
  AlertTriangle,
  Info,
  Activity
} from 'lucide-react';
import { useToast } from '@/hooks/use-toast';

export interface SearchProvider {
  name: 'postgres' | 'opensearch';
  display_name: string;
  description: string;
  icon: React.ReactNode;
  status: 'healthy' | 'degraded' | 'unhealthy';
  response_time_ms: number;
  last_check: string;
}

export interface SearchMetrics {
  provider: string;
  total_queries: number;
  avg_response_time_ms: number;
  error_rate: number;
  indexed_documents: number;
  index_size_bytes: number;
  last_updated: string;
}

interface SearchProviderToggleProps {
  className?: string;
}

export const SearchProviderToggle: React.FC<SearchProviderToggleProps> = ({ className }) => {
  const [currentProvider, setCurrentProvider] = useState<SearchProvider | null>(null);
  const [availableProviders, setAvailableProviders] = useState<SearchProvider[]>([]);
  const [metrics, setMetrics] = useState<SearchMetrics | null>(null);
  const [loading, setLoading] = useState(true);
  const [showConfig, setShowConfig] = useState(false);
  const [saving, setSaving] = useState(false);
  const { toast } = useToast();

  // Load search configuration on mount
  useEffect(() => {
    loadSearchConfig();
    loadSearchMetrics();
  }, []);

  const loadSearchConfig = async () => {
    try {
      const response = await fetch('/api/v1/search/config', {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
      });

      if (response.ok) {
        const data = await response.json();
        const config = data.data;
        
        setCurrentProvider({
          name: config.provider,
          display_name: config.provider === 'postgres' ? 'PostgreSQL' : 'OpenSearch',
          description: config.provider === 'postgres' 
            ? 'Full-text search using PostgreSQL trigram indexes'
            : 'Advanced search using OpenSearch/Elasticsearch',
          icon: config.provider === 'postgres' ? <Database className="h-4 w-4" /> : <Globe className="h-4 w-4" />,
          status: 'healthy', // Would be loaded from health check
          response_time_ms: 0,
          last_check: new Date().toISOString(),
        });

        setAvailableProviders([
          {
            name: 'postgres',
            display_name: 'PostgreSQL',
            description: 'Full-text search using PostgreSQL trigram indexes',
            icon: <Database className="h-4 w-4" />,
            status: 'healthy',
            response_time_ms: 0,
            last_check: new Date().toISOString(),
          },
          {
            name: 'opensearch',
            display_name: 'OpenSearch',
            description: 'Advanced search using OpenSearch/Elasticsearch',
            icon: <Globe className="h-4 w-4" />,
            status: 'healthy',
            response_time_ms: 0,
            last_check: new Date().toISOString(),
          },
        ]);
      }
    } catch (error) {
      console.error('Failed to load search config:', error);
      toast({
        title: "Failed to load search configuration",
        description: "Could not load search provider configuration.",
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };

  const loadSearchMetrics = async () => {
    try {
      const response = await fetch('/api/v1/search/metrics', {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
      });

      if (response.ok) {
        const data = await response.json();
        setMetrics(data.data);
      }
    } catch (error) {
      console.error('Failed to load search metrics:', error);
    }
  };

  const handleProviderChange = async (newProvider: string) => {
    if (newProvider === currentProvider?.name) return;

    setSaving(true);
    try {
      const response = await fetch('/api/v1/search/config', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
        body: JSON.stringify({
          provider: newProvider,
        }),
      });

      if (response.ok) {
        toast({
          title: "Search provider updated",
          description: `Switched to ${newProvider} search backend.`,
        });
        
        // Reload configuration
        await loadSearchConfig();
        await loadSearchMetrics();
      } else {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
    } catch (error) {
      toast({
        title: "Failed to update search provider",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "destructive",
      });
    } finally {
      setSaving(false);
    }
  };

  const getStatusColor = (status: SearchProvider['status']) => {
    switch (status) {
      case 'healthy':
        return 'bg-green-100 text-green-800';
      case 'degraded':
        return 'bg-yellow-100 text-yellow-800';
      case 'unhealthy':
        return 'bg-red-100 text-red-800';
    }
  };

  const getStatusIcon = (status: SearchProvider['status']) => {
    switch (status) {
      case 'healthy':
        return <CheckCircle className="h-3 w-3" />;
      case 'degraded':
        return <AlertTriangle className="h-3 w-3" />;
      case 'unhealthy':
        return <AlertTriangle className="h-3 w-3" />;
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  if (loading) {
    return (
      <Card className={className}>
        <CardContent className="p-4">
          <div className="flex items-center gap-2">
            <RefreshCw className="h-4 w-4 animate-spin" />
            <span className="text-sm">Loading search configuration...</span>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (!currentProvider) {
    return (
      <Card className={className}>
        <CardContent className="p-4">
          <div className="text-center text-gray-500">
            <AlertTriangle className="h-8 w-8 mx-auto mb-2" />
            <p className="text-sm">Failed to load search configuration</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <>
      <Card className={className}>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm flex items-center gap-2">
            <Settings className="h-4 w-4" />
            Search Provider
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Current Provider */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              {currentProvider.icon}
              <div>
                <p className="text-sm font-medium">{currentProvider.display_name}</p>
                <p className="text-xs text-gray-500">{currentProvider.description}</p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <Badge className={getStatusColor(currentProvider.status)}>
                {getStatusIcon(currentProvider.status)}
                <span className="ml-1 capitalize">{currentProvider.status}</span>
              </Badge>
            </div>
          </div>

          {/* Provider Selector */}
          <div>
            <label className="text-xs font-medium text-gray-700 mb-2 block">
              Switch Provider
            </label>
            <Select
              value={currentProvider.name}
              onValueChange={handleProviderChange}
              disabled={saving}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {availableProviders.map((provider) => (
                  <SelectItem key={provider.name} value={provider.name}>
                    <div className="flex items-center gap-2">
                      {provider.icon}
                      <span>{provider.display_name}</span>
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Metrics */}
          {metrics && (
            <div className="space-y-2">
              <div className="flex items-center justify-between text-xs">
                <span className="text-gray-500">Indexed Documents</span>
                <span className="font-medium">{metrics.indexed_documents.toLocaleString()}</span>
              </div>
              <div className="flex items-center justify-between text-xs">
                <span className="text-gray-500">Avg Response Time</span>
                <span className="font-medium">{metrics.avg_response_time_ms.toFixed(1)}ms</span>
              </div>
              <div className="flex items-center justify-between text-xs">
                <span className="text-gray-500">Error Rate</span>
                <span className="font-medium">{(metrics.error_rate * 100).toFixed(1)}%</span>
              </div>
              <div className="flex items-center justify-between text-xs">
                <span className="text-gray-500">Index Size</span>
                <span className="font-medium">{formatBytes(metrics.index_size_bytes)}</span>
              </div>
            </div>
          )}

          {/* Actions */}
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setShowConfig(true)}
              className="flex-1"
            >
              <Settings className="h-3 w-3 mr-1" />
              Configure
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={loadSearchMetrics}
              disabled={saving}
            >
              <RefreshCw className={`h-3 w-3 ${saving ? 'animate-spin' : ''}`} />
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Configuration Dialog */}
      <Dialog open={showConfig} onOpenChange={setShowConfig}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Search Provider Configuration</DialogTitle>
            <DialogDescription>
              Configure search backend settings and monitor performance.
            </DialogDescription>
          </DialogHeader>
          
          <div className="space-y-6">
            {/* Provider Comparison */}
            <div className="grid gap-4 md:grid-cols-2">
              <Card>
                <CardHeader className="pb-3">
                  <CardTitle className="text-sm flex items-center gap-2">
                    <Database className="h-4 w-4" />
                    PostgreSQL
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-2">
                  <div className="text-xs text-gray-600">
                    <p>• Built-in trigram search</p>
                    <p>• No additional infrastructure</p>
                    <p>• Good for basic full-text search</p>
                    <p>• Limited advanced features</p>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader className="pb-3">
                  <CardTitle className="text-sm flex items-center gap-2">
                    <Globe className="h-4 w-4" />
                    OpenSearch
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-2">
                  <div className="text-xs text-gray-600">
                    <p>• Advanced search capabilities</p>
                    <p>• Faceted search and aggregations</p>
                    <p>• Better performance at scale</p>
                    <p>• Requires additional infrastructure</p>
                  </div>
                </CardContent>
              </Card>
            </div>

            {/* Performance Metrics */}
            {metrics && (
              <Card>
                <CardHeader className="pb-3">
                  <CardTitle className="text-sm flex items-center gap-2">
                    <Activity className="h-4 w-4" />
                    Performance Metrics
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="grid gap-3 md:grid-cols-2">
                    <div>
                      <p className="text-xs text-gray-500">Total Queries</p>
                      <p className="text-lg font-semibold">{metrics.total_queries.toLocaleString()}</p>
                    </div>
                    <div>
                      <p className="text-xs text-gray-500">Average Response Time</p>
                      <p className="text-lg font-semibold">{metrics.avg_response_time_ms.toFixed(1)}ms</p>
                    </div>
                    <div>
                      <p className="text-xs text-gray-500">Error Rate</p>
                      <p className="text-lg font-semibold">{(metrics.error_rate * 100).toFixed(1)}%</p>
                    </div>
                    <div>
                      <p className="text-xs text-gray-500">Index Size</p>
                      <p className="text-lg font-semibold">{formatBytes(metrics.index_size_bytes)}</p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            )}

            {/* Health Status */}
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm flex items-center gap-2">
                  <Info className="h-4 w-4" />
                  Health Status
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="flex items-center gap-2">
                  <Badge className={getStatusColor(currentProvider.status)}>
                    {getStatusIcon(currentProvider.status)}
                    <span className="ml-1 capitalize">{currentProvider.status}</span>
                  </Badge>
                  <span className="text-xs text-gray-500">
                    Last checked: {new Date(currentProvider.last_check).toLocaleString()}
                  </span>
                </div>
              </CardContent>
            </Card>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setShowConfig(false)}>
              Close
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
};
