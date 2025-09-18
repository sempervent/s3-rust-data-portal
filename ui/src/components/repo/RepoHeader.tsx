// BlackLake Repository Header
// Week 4: Status badges and quick actions

import React, { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent } from '@/components/ui/card';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { 
  Shield, 
  HardDrive, 
  Search, 
  Upload, 
  MoreVertical, 
  Settings,
  GitBranch,
  Tag,
  RefreshCw,
  AlertTriangle,
  CheckCircle,
  Info,
  Database,
  Globe
} from 'lucide-react';
import { useToast } from '@/hooks/use-toast';

export interface RepoStatus {
  branch_protection: 'enabled' | 'disabled';
  quota_status: 'healthy' | 'warning' | 'critical' | 'unlimited';
  quota_usage: number; // percentage
  quota_limit: number; // bytes
  search_provider: 'postgres' | 'opensearch';
  legal_hold: boolean;
  features: {
    auto_rdf: boolean;
    lineage_tracking: boolean;
    schema_validation: boolean;
  };
}

export interface RepoInfo {
  name: string;
  description?: string;
  default_branch: string;
  current_branch: string;
  last_commit: {
    id: string;
    message: string;
    author: string;
    timestamp: string;
  };
  status: RepoStatus;
}

interface RepoHeaderProps {
  repo: RepoInfo;
  onBranchChange?: (branch: string) => void;
  onRefresh?: () => void;
  onUpload?: () => void;
  onSearch?: () => void;
  onSettings?: () => void;
  className?: string;
}

export const RepoHeader: React.FC<RepoHeaderProps> = ({
  repo,
  onBranchChange,
  onRefresh,
  onUpload,
  onSearch,
  onSettings,
  className,
}) => {
  const [branches, setBranches] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const { toast } = useToast();

  // Load branches on mount
  useEffect(() => {
    loadBranches();
  }, [repo.name]);

  const loadBranches = async () => {
    try {
      const response = await fetch(`/api/v1/repos/${repo.name}/refs`, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
      });

      if (response.ok) {
        const data = await response.json();
        setBranches(data.refs?.map((ref: any) => ref.name) || []);
      }
    } catch (error) {
      console.error('Failed to load branches:', error);
    }
  };

  const handleRefresh = async () => {
    setLoading(true);
    try {
      await onRefresh?.();
      await loadBranches();
      toast({
        title: "Repository refreshed",
        description: "Repository information has been updated.",
      });
    } catch (error) {
      toast({
        title: "Refresh failed",
        description: "Could not refresh repository information.",
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const getQuotaColor = (status: RepoStatus['quota_status']) => {
    switch (status) {
      case 'healthy':
        return 'bg-green-100 text-green-800';
      case 'warning':
        return 'bg-yellow-100 text-yellow-800';
      case 'critical':
        return 'bg-red-100 text-red-800';
      case 'unlimited':
        return 'bg-gray-100 text-gray-800';
    }
  };

  const getQuotaIcon = (status: RepoStatus['quota_status']) => {
    switch (status) {
      case 'healthy':
        return <CheckCircle className="h-3 w-3" />;
      case 'warning':
        return <AlertTriangle className="h-3 w-3" />;
      case 'critical':
        return <AlertTriangle className="h-3 w-3" />;
      case 'unlimited':
        return <Info className="h-3 w-3" />;
    }
  };

  const getSearchProviderIcon = (provider: RepoStatus['search_provider']) => {
    switch (provider) {
      case 'postgres':
        return <Database className="h-3 w-3" />;
      case 'opensearch':
        return <Globe className="h-3 w-3" />;
    }
  };

  return (
    <TooltipProvider>
      <Card className={className}>
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            {/* Left side - Repo info and status */}
            <div className="flex items-center gap-4">
              <div>
                <h1 className="text-xl font-bold">{repo.name}</h1>
                {repo.description && (
                  <p className="text-sm text-gray-600">{repo.description}</p>
                )}
              </div>

              {/* Status badges */}
              <div className="flex items-center gap-2">
                {/* Branch protection */}
                <Tooltip>
                  <TooltipTrigger>
                    <Badge 
                      className={
                        repo.status.branch_protection === 'enabled'
                          ? 'bg-blue-100 text-blue-800'
                          : 'bg-gray-100 text-gray-800'
                      }
                    >
                      <Shield className="h-3 w-3 mr-1" />
                      {repo.status.branch_protection === 'enabled' ? 'Protected' : 'Open'}
                    </Badge>
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>
                      {repo.status.branch_protection === 'enabled' 
                        ? 'Branch protection is enabled'
                        : 'Branch protection is disabled'
                      }
                    </p>
                  </TooltipContent>
                </Tooltip>

                {/* Quota status */}
                <Tooltip>
                  <TooltipTrigger>
                    <Badge className={getQuotaColor(repo.status.quota_status)}>
                      {getQuotaIcon(repo.status.quota_status)}
                      <span className="ml-1">
                        {repo.status.quota_status === 'unlimited' 
                          ? 'Unlimited' 
                          : `${repo.status.quota_usage}%`
                        }
                      </span>
                    </Badge>
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>
                      {repo.status.quota_status === 'unlimited' 
                        ? 'No storage quota limit'
                        : `Storage usage: ${repo.status.quota_usage}% (${formatBytes(repo.status.quota_limit)})`
                      }
                    </p>
                  </TooltipContent>
                </Tooltip>

                {/* Legal hold */}
                {repo.status.legal_hold && (
                  <Tooltip>
                    <TooltipTrigger>
                      <Badge className="bg-red-100 text-red-800">
                        <Shield className="h-3 w-3 mr-1" />
                        Legal Hold
                      </Badge>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>Legal hold is active - deletion is prevented</p>
                    </TooltipContent>
                  </Tooltip>
                )}

                {/* Search provider */}
                <Tooltip>
                  <TooltipTrigger>
                    <Badge className="bg-purple-100 text-purple-800">
                      {getSearchProviderIcon(repo.status.search_provider)}
                      <span className="ml-1 capitalize">{repo.status.search_provider}</span>
                    </Badge>
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>Search provider: {repo.status.search_provider}</p>
                  </TooltipContent>
                </Tooltip>

                {/* Feature badges */}
                {repo.status.features.auto_rdf && (
                  <Tooltip>
                    <TooltipTrigger>
                      <Badge className="bg-green-100 text-green-800">
                        RDF
                      </Badge>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>Automatic RDF generation enabled</p>
                    </TooltipContent>
                  </Tooltip>
                )}

                {repo.status.features.lineage_tracking && (
                  <Tooltip>
                    <TooltipTrigger>
                      <Badge className="bg-blue-100 text-blue-800">
                        Lineage
                      </Badge>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>Data lineage tracking enabled</p>
                    </TooltipContent>
                  </Tooltip>
                )}
              </div>
            </div>

            {/* Right side - Actions and branch selector */}
            <div className="flex items-center gap-2">
              {/* Branch selector */}
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="outline" size="sm" className="flex items-center gap-2">
                    <GitBranch className="h-4 w-4" />
                    {repo.current_branch}
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent>
                  {branches.map((branch) => (
                    <DropdownMenuItem
                      key={branch}
                      onClick={() => onBranchChange?.(branch)}
                      className={branch === repo.current_branch ? 'bg-blue-50' : ''}
                    >
                      <GitBranch className="h-4 w-4 mr-2" />
                      {branch}
                      {branch === repo.default_branch && (
                        <Badge variant="secondary" className="ml-2 text-xs">
                          default
                        </Badge>
                      )}
                    </DropdownMenuItem>
                  ))}
                </DropdownMenuContent>
              </DropdownMenu>

              {/* Quick actions */}
              <Button
                variant="outline"
                size="sm"
                onClick={onSearch}
                className="flex items-center gap-2"
              >
                <Search className="h-4 w-4" />
                Search
              </Button>

              <Button
                variant="outline"
                size="sm"
                onClick={onUpload}
                className="flex items-center gap-2"
              >
                <Upload className="h-4 w-4" />
                Upload
              </Button>

              <Button
                variant="outline"
                size="sm"
                onClick={handleRefresh}
                disabled={loading}
                className="flex items-center gap-2"
              >
                <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
                Refresh
              </Button>

              {/* More actions */}
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="outline" size="sm">
                    <MoreVertical className="h-4 w-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent>
                  <DropdownMenuItem onClick={onSettings}>
                    <Settings className="h-4 w-4 mr-2" />
                    Repository Settings
                  </DropdownMenuItem>
                  <DropdownMenuItem>
                    <Tag className="h-4 w-4 mr-2" />
                    Manage Tags
                  </DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem>
                    <HardDrive className="h-4 w-4 mr-2" />
                    Storage Info
                  </DropdownMenuItem>
                  <DropdownMenuItem>
                    <Shield className="h-4 w-4 mr-2" />
                    Security Settings
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          </div>

          {/* Last commit info */}
          {repo.last_commit && (
            <div className="mt-3 pt-3 border-t border-gray-200">
              <div className="flex items-center gap-2 text-sm text-gray-600">
                <span>Last commit:</span>
                <code className="bg-gray-100 px-2 py-1 rounded text-xs">
                  {repo.last_commit.id.substring(0, 8)}
                </code>
                <span>by {repo.last_commit.author}</span>
                <span>•</span>
                <span>{repo.last_commit.message}</span>
                <span>•</span>
                <span>{new Date(repo.last_commit.timestamp).toLocaleString()}</span>
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </TooltipProvider>
  );
};
