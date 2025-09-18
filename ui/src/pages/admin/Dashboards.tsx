// BlackLake Admin Dashboards
// Week 5: Real-time metrics and system monitoring

import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { 
  Activity, 
  Database, 
  HardDrive, 
  Users, 
  AlertTriangle, 
  CheckCircle, 
  RefreshCw,
  TrendingUp,
  TrendingDown,
  Clock,
  Server,
  Globe,
  Shield
} from 'lucide-react';
import { useToast } from '@/hooks/use-toast';

export interface SystemMetrics {
  api: {
    requests_total: number;
    requests_per_second: number;
    response_time_p95: number;
    response_time_p99: number;
    error_rate: number;
    active_connections: number;
  };
  database: {
    connections: number;
    max_connections: number;
    active_queries: number;
    slow_queries: number;
    cache_hit_ratio: number;
    disk_usage: number;
  };
  storage: {
    total_objects: number;
    total_size: number;
    objects_per_second: number;
    upload_success_rate: number;
    download_success_rate: number;
  };
  jobs: {
    queue_depth: number;
    active_jobs: number;
    completed_jobs: number;
    failed_jobs: number;
    avg_processing_time: number;
  };
  webhooks: {
    total_webhooks: number;
    active_webhooks: number;
    deliveries_today: number;
    failed_deliveries: number;
    avg_delivery_time: number;
  };
  quotas: {
    total_repos: number;
    repos_over_soft_quota: number;
    repos_over_hard_quota: number;
    total_storage_used: number;
    total_storage_quota: number;
  };
}

export interface HealthStatus {
  status: 'healthy' | 'degraded' | 'unhealthy';
  services: {
    api: 'healthy' | 'degraded' | 'unhealthy';
    database: 'healthy' | 'degraded' | 'unhealthy';
    storage: 'healthy' | 'degraded' | 'unhealthy';
    jobs: 'healthy' | 'degraded' | 'unhealthy';
    webhooks: 'healthy' | 'degraded' | 'unhealthy';
  };
  last_check: string;
  uptime: string;
}

interface DashboardsProps {
  className?: string;
}

export const Dashboards: React.FC<DashboardsProps> = ({ className }) => {
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null);
  const [health, setHealth] = useState<HealthStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);
  const { toast } = useToast();

  // Load metrics on mount and set up auto-refresh
  useEffect(() => {
    loadMetrics();
    const interval = setInterval(loadMetrics, 30000); // Refresh every 30 seconds
    return () => clearInterval(interval);
  }, []);

  const loadMetrics = async () => {
    try {
      const response = await fetch('/api/v1/metrics/summary', {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
      });

      if (response.ok) {
        const data = await response.json();
        setMetrics(data.data.metrics);
        setHealth(data.data.health);
        setLastUpdated(new Date());
      } else {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
    } catch (error) {
      console.error('Failed to load metrics:', error);
      toast({
        title: "Failed to load metrics",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'healthy':
        return 'bg-green-100 text-green-800';
      case 'degraded':
        return 'bg-yellow-100 text-yellow-800';
      case 'unhealthy':
        return 'bg-red-100 text-red-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'healthy':
        return <CheckCircle className="h-4 w-4" />;
      case 'degraded':
        return <AlertTriangle className="h-4 w-4" />;
      case 'unhealthy':
        return <AlertTriangle className="h-4 w-4" />;
      default:
        return <Clock className="h-4 w-4" />;
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatNumber = (num: number) => {
    if (num >= 1000000) {
      return (num / 1000000).toFixed(1) + 'M';
    } else if (num >= 1000) {
      return (num / 1000).toFixed(1) + 'K';
    }
    return num.toString();
  };

  const formatPercentage = (value: number) => {
    return (value * 100).toFixed(1) + '%';
  };

  if (loading) {
    return (
      <div className={`space-y-6 ${className}`}>
        <div className="flex items-center justify-between">
          <h2 className="text-2xl font-bold">System Dashboards</h2>
          <Button disabled>
            <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
            Refreshing...
          </Button>
        </div>
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 6 }).map((_, i) => (
            <Card key={i} className="animate-pulse">
              <CardHeader className="pb-2">
                <div className="h-4 bg-gray-200 rounded w-3/4"></div>
              </CardHeader>
              <CardContent>
                <div className="h-8 bg-gray-200 rounded w-1/2 mb-2"></div>
                <div className="h-3 bg-gray-200 rounded w-full"></div>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    );
  }

  if (!metrics || !health) {
    return (
      <div className={`space-y-6 ${className}`}>
        <div className="text-center py-8">
          <AlertTriangle className="h-12 w-12 mx-auto mb-4 text-red-500" />
          <h3 className="text-lg font-medium mb-2">Failed to load metrics</h3>
          <p className="text-gray-500 mb-4">
            Could not load system metrics. Please try again.
          </p>
          <Button onClick={loadMetrics}>
            <RefreshCw className="h-4 w-4 mr-2" />
            Retry
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">System Dashboards</h2>
          <p className="text-gray-600">
            Real-time system metrics and health monitoring
            {lastUpdated && (
              <span className="ml-2 text-sm text-gray-500">
                Last updated: {lastUpdated.toLocaleTimeString()}
              </span>
            )}
          </p>
        </div>
        <Button onClick={loadMetrics} disabled={loading}>
          <RefreshCw className={`h-4 w-4 mr-2 ${loading ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {/* System Health Overview */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            System Health Overview
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            <div className="flex items-center justify-between p-3 border rounded">
              <div className="flex items-center gap-2">
                <Server className="h-4 w-4" />
                <span className="font-medium">API</span>
              </div>
              <Badge className={getStatusColor(health.services.api)}>
                {getStatusIcon(health.services.api)}
                <span className="ml-1 capitalize">{health.services.api}</span>
              </Badge>
            </div>

            <div className="flex items-center justify-between p-3 border rounded">
              <div className="flex items-center gap-2">
                <Database className="h-4 w-4" />
                <span className="font-medium">Database</span>
              </div>
              <Badge className={getStatusColor(health.services.database)}>
                {getStatusIcon(health.services.database)}
                <span className="ml-1 capitalize">{health.services.database}</span>
              </Badge>
            </div>

            <div className="flex items-center justify-between p-3 border rounded">
              <div className="flex items-center gap-2">
                <HardDrive className="h-4 w-4" />
                <span className="font-medium">Storage</span>
              </div>
              <Badge className={getStatusColor(health.services.storage)}>
                {getStatusIcon(health.services.storage)}
                <span className="ml-1 capitalize">{health.services.storage}</span>
              </Badge>
            </div>

            <div className="flex items-center justify-between p-3 border rounded">
              <div className="flex items-center gap-2">
                <Activity className="h-4 w-4" />
                <span className="font-medium">Jobs</span>
              </div>
              <Badge className={getStatusColor(health.services.jobs)}>
                {getStatusIcon(health.services.jobs)}
                <span className="ml-1 capitalize">{health.services.jobs}</span>
              </Badge>
            </div>

            <div className="flex items-center justify-between p-3 border rounded">
              <div className="flex items-center gap-2">
                <Globe className="h-4 w-4" />
                <span className="font-medium">Webhooks</span>
              </div>
              <Badge className={getStatusColor(health.services.webhooks)}>
                {getStatusIcon(health.services.webhooks)}
                <span className="ml-1 capitalize">{health.services.webhooks}</span>
              </Badge>
            </div>

            <div className="flex items-center justify-between p-3 border rounded">
              <div className="flex items-center gap-2">
                <Clock className="h-4 w-4" />
                <span className="font-medium">Uptime</span>
              </div>
              <span className="text-sm text-gray-600">{health.uptime}</span>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* API Metrics */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Server className="h-5 w-5" />
            API Performance
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            <div className="text-center">
              <div className="text-2xl font-bold">{formatNumber(metrics.api.requests_total)}</div>
              <div className="text-sm text-gray-600">Total Requests</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{metrics.api.requests_per_second.toFixed(1)}</div>
              <div className="text-sm text-gray-600">Requests/sec</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{metrics.api.response_time_p95.toFixed(0)}ms</div>
              <div className="text-sm text-gray-600">P95 Response Time</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{formatPercentage(metrics.api.error_rate)}</div>
              <div className="text-sm text-gray-600">Error Rate</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{metrics.api.active_connections}</div>
              <div className="text-sm text-gray-600">Active Connections</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{metrics.api.response_time_p99.toFixed(0)}ms</div>
              <div className="text-sm text-gray-600">P99 Response Time</div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Database Metrics */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Database className="h-5 w-5" />
            Database Performance
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            <div className="text-center">
              <div className="text-2xl font-bold">{metrics.database.connections}</div>
              <div className="text-sm text-gray-600">Active Connections</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{metrics.database.active_queries}</div>
              <div className="text-sm text-gray-600">Active Queries</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{metrics.database.slow_queries}</div>
              <div className="text-sm text-gray-600">Slow Queries</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{formatPercentage(metrics.database.cache_hit_ratio)}</div>
              <div className="text-sm text-gray-600">Cache Hit Ratio</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{formatBytes(metrics.database.disk_usage)}</div>
              <div className="text-sm text-gray-600">Disk Usage</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{metrics.database.max_connections}</div>
              <div className="text-sm text-gray-600">Max Connections</div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Storage Metrics */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <HardDrive className="h-5 w-5" />
            Storage Performance
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            <div className="text-center">
              <div className="text-2xl font-bold">{formatNumber(metrics.storage.total_objects)}</div>
              <div className="text-sm text-gray-600">Total Objects</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{formatBytes(metrics.storage.total_size)}</div>
              <div className="text-sm text-gray-600">Total Size</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{metrics.storage.objects_per_second.toFixed(1)}</div>
              <div className="text-sm text-gray-600">Objects/sec</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{formatPercentage(metrics.storage.upload_success_rate)}</div>
              <div className="text-sm text-gray-600">Upload Success Rate</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{formatPercentage(metrics.storage.download_success_rate)}</div>
              <div className="text-sm text-gray-600">Download Success Rate</div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Job Metrics */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="h-5 w-5" />
            Background Jobs
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            <div className="text-center">
              <div className="text-2xl font-bold">{metrics.jobs.queue_depth}</div>
              <div className="text-sm text-gray-600">Queue Depth</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{metrics.jobs.active_jobs}</div>
              <div className="text-sm text-gray-600">Active Jobs</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{formatNumber(metrics.jobs.completed_jobs)}</div>
              <div className="text-sm text-gray-600">Completed Jobs</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{metrics.jobs.failed_jobs}</div>
              <div className="text-sm text-gray-600">Failed Jobs</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{metrics.jobs.avg_processing_time.toFixed(1)}s</div>
              <div className="text-sm text-gray-600">Avg Processing Time</div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Webhook Metrics */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Globe className="h-5 w-5" />
            Webhook Delivery
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            <div className="text-center">
              <div className="text-2xl font-bold">{metrics.webhooks.total_webhooks}</div>
              <div className="text-sm text-gray-600">Total Webhooks</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{metrics.webhooks.active_webhooks}</div>
              <div className="text-sm text-gray-600">Active Webhooks</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{formatNumber(metrics.webhooks.deliveries_today)}</div>
              <div className="text-sm text-gray-600">Deliveries Today</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{metrics.webhooks.failed_deliveries}</div>
              <div className="text-sm text-gray-600">Failed Deliveries</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{metrics.webhooks.avg_delivery_time.toFixed(1)}ms</div>
              <div className="text-sm text-gray-600">Avg Delivery Time</div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Quota Metrics */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Users className="h-5 w-5" />
            Storage Quotas
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            <div className="text-center">
              <div className="text-2xl font-bold">{metrics.quotas.total_repos}</div>
              <div className="text-sm text-gray-600">Total Repositories</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-yellow-600">{metrics.quotas.repos_over_soft_quota}</div>
              <div className="text-sm text-gray-600">Over Soft Quota</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-red-600">{metrics.quotas.repos_over_hard_quota}</div>
              <div className="text-sm text-gray-600">Over Hard Quota</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{formatBytes(metrics.quotas.total_storage_used)}</div>
              <div className="text-sm text-gray-600">Storage Used</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">{formatBytes(metrics.quotas.total_storage_quota)}</div>
              <div className="text-sm text-gray-600">Total Quota</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold">
                {formatPercentage(metrics.quotas.total_storage_used / metrics.quotas.total_storage_quota)}
              </div>
              <div className="text-sm text-gray-600">Quota Usage</div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};