// BlackLake Export Dialog
// Week 5: Multi-select export with job status

import React, { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Checkbox } from '@/components/ui/checkbox';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { 
  Archive, 
  Download, 
  FileText, 
  CheckCircle, 
  AlertCircle, 
  Clock,
  RefreshCw
} from 'lucide-react';
import { useToast } from '@/hooks/use-toast';
import { useDownloadQueue } from '@/components/download/DownloadQueue';

export interface ExportItem {
  path: string;
  ref: string;
  name: string;
  size: number;
  type: string;
}

export interface ExportJob {
  id: string;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  progress: number;
  total_items: number;
  processed_items: number;
  output_size: number;
  download_url?: string;
  error_message?: string;
  created_at: number;
  completed_at?: number;
}

interface ExportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  items: ExportItem[];
  repo: string;
  onExportComplete?: (job: ExportJob) => void;
}

export const ExportDialog: React.FC<ExportDialogProps> = ({
  open,
  onOpenChange,
  items,
  repo,
  onExportComplete,
}) => {
  const [selectedItems, setSelectedItems] = useState<ExportItem[]>([]);
  const [includeMetadata, setIncludeMetadata] = useState(true);
  const [includeRdf, setIncludeRdf] = useState(false);
  const [exportName, setExportName] = useState('');
  const [isExporting, setIsExporting] = useState(false);
  const [exportJob, setExportJob] = useState<ExportJob | null>(null);
  const [pollingInterval, setPollingInterval] = useState<NodeJS.Timeout | null>(null);
  
  const { toast } = useToast();
  const { addExportDownload } = useDownloadQueue();

  // Initialize selected items when dialog opens
  useEffect(() => {
    if (open) {
      setSelectedItems(items);
      setExportName(`${repo}-export-${new Date().toISOString().split('T')[0]}`);
    }
  }, [open, items, repo]);

  // Cleanup polling interval on unmount
  useEffect(() => {
    return () => {
      if (pollingInterval) {
        clearInterval(pollingInterval);
      }
    };
  }, [pollingInterval]);

  const handleItemToggle = (item: ExportItem, checked: boolean) => {
    if (checked) {
      setSelectedItems(prev => [...prev, item]);
    } else {
      setSelectedItems(prev => prev.filter(i => i.path !== item.path));
    }
  };

  const handleSelectAll = (checked: boolean) => {
    if (checked) {
      setSelectedItems(items);
    } else {
      setSelectedItems([]);
    }
  };

  const handleExport = async () => {
    if (selectedItems.length === 0) {
      toast({
        title: "No items selected",
        description: "Please select at least one item to export.",
        variant: "destructive",
      });
      return;
    }

    setIsExporting(true);

    try {
      const manifest = {
        items: selectedItems.map(item => ({
          ref: item.ref,
          path: item.path,
        })),
        include_meta: includeMetadata,
        include_rdf: includeRdf,
      };

      const response = await fetch(`/api/v1/repos/${repo}/export`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
        body: JSON.stringify({ manifest }),
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const job: ExportJob = await response.json();
      setExportJob(job);

      // Start polling for job status
      const interval = setInterval(async () => {
        try {
          const statusResponse = await fetch(`/api/v1/exports/${job.id}`, {
            headers: {
              'Authorization': `Bearer ${localStorage.getItem('token')}`,
            },
          });

          if (statusResponse.ok) {
            const updatedJob: ExportJob = await statusResponse.json();
            setExportJob(updatedJob);

            if (updatedJob.status === 'completed') {
              clearInterval(interval);
              setIsExporting(false);
              
              toast({
                title: "Export completed",
                description: `Export "${exportName}" has been completed successfully.`,
              });

              if (updatedJob.download_url) {
                addExportDownload(
                  updatedJob.download_url,
                  `${exportName}.tar.gz`,
                  updatedJob.output_size,
                  repo
                );
              }

              onExportComplete?.(updatedJob);
              onOpenChange(false);
            } else if (updatedJob.status === 'failed') {
              clearInterval(interval);
              setIsExporting(false);
              
              toast({
                title: "Export failed",
                description: updatedJob.error_message || "Export failed for unknown reason.",
                variant: "destructive",
              });
            }
          }
        } catch (error) {
          console.error('Failed to check export status:', error);
        }
      }, 2000);

      setPollingInterval(interval);

    } catch (error) {
      setIsExporting(false);
      toast({
        title: "Export failed",
        description: error instanceof Error ? error.message : "Failed to start export",
        variant: "destructive",
      });
    }
  };

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const getTotalSize = () => {
    return selectedItems.reduce((total, item) => total + item.size, 0);
  };

  const getStatusIcon = (status: ExportJob['status']) => {
    switch (status) {
      case 'pending':
        return <Clock className="h-4 w-4 text-yellow-500" />;
      case 'processing':
        return <RefreshCw className="h-4 w-4 text-blue-500 animate-spin" />;
      case 'completed':
        return <CheckCircle className="h-4 w-4 text-green-500" />;
      case 'failed':
        return <AlertCircle className="h-4 w-4 text-red-500" />;
    }
  };

  const getStatusColor = (status: ExportJob['status']) => {
    switch (status) {
      case 'pending':
        return 'bg-yellow-100 text-yellow-800';
      case 'processing':
        return 'bg-blue-100 text-blue-800';
      case 'completed':
        return 'bg-green-100 text-green-800';
      case 'failed':
        return 'bg-red-100 text-red-800';
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Archive className="h-5 w-5" />
            Export Items
          </DialogTitle>
          <DialogDescription>
            Export selected items from {repo} as a downloadable archive.
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-y-auto space-y-4">
          {/* Export Configuration */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-sm">Export Configuration</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <Label htmlFor="export-name">Export Name</Label>
                <Input
                  id="export-name"
                  value={exportName}
                  onChange={(e) => setExportName(e.target.value)}
                  placeholder="Enter export name"
                />
              </div>

              <div className="flex items-center space-x-2">
                <Checkbox
                  id="include-metadata"
                  checked={includeMetadata}
                  onCheckedChange={(checked) => setIncludeMetadata(checked as boolean)}
                />
                <Label htmlFor="include-metadata">Include metadata</Label>
              </div>

              <div className="flex items-center space-x-2">
                <Checkbox
                  id="include-rdf"
                  checked={includeRdf}
                  onCheckedChange={(checked) => setIncludeRdf(checked as boolean)}
                />
                <Label htmlFor="include-rdf">Include RDF data</Label>
              </div>
            </CardContent>
          </Card>

          {/* Item Selection */}
          <Card>
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <CardTitle className="text-sm">Select Items</CardTitle>
                <div className="flex items-center space-x-2">
                  <Checkbox
                    id="select-all"
                    checked={selectedItems.length === items.length}
                    onCheckedChange={handleSelectAll}
                  />
                  <Label htmlFor="select-all" className="text-sm">
                    Select All ({selectedItems.length}/{items.length})
                  </Label>
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-2 max-h-48 overflow-y-auto">
                {items.map((item) => (
                  <div key={item.path} className="flex items-center space-x-3 p-2 border rounded">
                    <Checkbox
                      id={`item-${item.path}`}
                      checked={selectedItems.some(i => i.path === item.path)}
                      onCheckedChange={(checked) => handleItemToggle(item, checked as boolean)}
                    />
                    <FileText className="h-4 w-4 text-blue-500 flex-shrink-0" />
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium truncate">{item.name}</p>
                      <p className="text-xs text-gray-500">
                        {formatFileSize(item.size)} • {item.ref} • {item.type}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          {/* Export Summary */}
          {selectedItems.length > 0 && (
            <Card>
              <CardContent className="pt-4">
                <div className="flex justify-between text-sm">
                  <span>Selected items:</span>
                  <span className="font-medium">{selectedItems.length}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span>Total size:</span>
                  <span className="font-medium">{formatFileSize(getTotalSize())}</span>
                </div>
              </CardContent>
            </Card>
          )}

          {/* Export Progress */}
          {exportJob && (
            <Card>
              <CardHeader className="pb-3">
                <div className="flex items-center gap-2">
                  <CardTitle className="text-sm">Export Progress</CardTitle>
                  {getStatusIcon(exportJob.status)}
                  <Badge className={getStatusColor(exportJob.status)}>
                    {exportJob.status}
                  </Badge>
                </div>
              </CardHeader>
              <CardContent>
                {exportJob.status === 'processing' && (
                  <div className="space-y-2">
                    <Progress value={exportJob.progress} className="h-2" />
                    <div className="flex justify-between text-sm text-gray-600">
                      <span>{exportJob.processed_items} of {exportJob.total_items} items</span>
                      <span>{exportJob.progress}%</span>
                    </div>
                  </div>
                )}
                
                {exportJob.status === 'completed' && (
                  <div className="text-sm text-green-600">
                    Export completed successfully! Download will start automatically.
                  </div>
                )}
                
                {exportJob.status === 'failed' && exportJob.error_message && (
                  <div className="text-sm text-red-600">
                    Export failed: {exportJob.error_message}
                  </div>
                )}
              </CardContent>
            </Card>
          )}
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={isExporting}
          >
            Cancel
          </Button>
          <Button
            onClick={handleExport}
            disabled={isExporting || selectedItems.length === 0}
            className="flex items-center gap-2"
          >
            {isExporting ? (
              <>
                <RefreshCw className="h-4 w-4 animate-spin" />
                Exporting...
              </>
            ) : (
              <>
                <Download className="h-4 w-4" />
                Export ({selectedItems.length} items)
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
