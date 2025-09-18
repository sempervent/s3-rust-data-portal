// BlackLake Download Manager
// Week 5: Batch downloads with progress and retry

import React, { useState, useEffect, useCallback } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { 
  Download, 
  Pause, 
  Play, 
  Trash2, 
  RefreshCw, 
  CheckCircle, 
  XCircle, 
  AlertTriangle,
  FileText,
  Archive,
  MoreVertical
} from 'lucide-react';
import { useToast } from '@/hooks/use-toast';

export interface DownloadItem {
  id: string;
  repo: string;
  path: string;
  filename: string;
  size: number;
  status: 'pending' | 'downloading' | 'completed' | 'failed' | 'paused';
  progress: number;
  error?: string;
  presignedUrl?: string;
  retryCount: number;
  maxRetries: number;
  createdAt: Date;
  completedAt?: Date;
}

export interface DownloadQueue {
  items: DownloadItem[];
  isRunning: boolean;
  concurrentDownloads: number;
  maxConcurrent: number;
}

interface DownloadQueueProps {
  className?: string;
}

export const DownloadQueue: React.FC<DownloadQueueProps> = ({ className }) => {
  const [queue, setQueue] = useState<DownloadQueue>({
    items: [],
    isRunning: false,
    concurrentDownloads: 0,
    maxConcurrent: 3,
  });
  const [showQueue, setShowQueue] = useState(false);
  const [selectedItems, setSelectedItems] = useState<Set<string>>(new Set());
  const { toast } = useToast();

  // Load queue from localStorage on mount
  useEffect(() => {
    const savedQueue = localStorage.getItem('blacklake-download-queue');
    if (savedQueue) {
      try {
        const parsed = JSON.parse(savedQueue);
        setQueue({
          ...parsed,
          items: parsed.items.map((item: any) => ({
            ...item,
            createdAt: new Date(item.createdAt),
            completedAt: item.completedAt ? new Date(item.completedAt) : undefined,
          })),
        });
      } catch (error) {
        console.error('Failed to load download queue:', error);
      }
    }
  }, []);

  // Save queue to localStorage whenever it changes
  useEffect(() => {
    localStorage.setItem('blacklake-download-queue', JSON.stringify(queue));
  }, [queue]);

  // Add items to download queue
  const addToQueue = useCallback((items: Omit<DownloadItem, 'id' | 'status' | 'progress' | 'retryCount' | 'createdAt'>[]) => {
    const newItems = items.map(item => ({
      ...item,
      id: `${item.repo}-${item.path}-${Date.now()}-${Math.random()}`,
      status: 'pending' as const,
      progress: 0,
      retryCount: 0,
      createdAt: new Date(),
    }));

    setQueue(prev => ({
      ...prev,
      items: [...prev.items, ...newItems],
    }));

    toast({
      title: "Added to download queue",
      description: `${items.length} item(s) added to download queue.`,
    });
  }, [toast]);

  // Remove items from queue
  const removeFromQueue = useCallback((itemIds: string[]) => {
    setQueue(prev => ({
      ...prev,
      items: prev.items.filter(item => !itemIds.includes(item.id)),
    }));
  }, []);

  // Update item status
  const updateItemStatus = useCallback((itemId: string, updates: Partial<DownloadItem>) => {
    setQueue(prev => ({
      ...prev,
      items: prev.items.map(item =>
        item.id === itemId ? { ...item, ...updates } : item
      ),
    }));
  }, []);

  // Start downloads
  const startDownloads = useCallback(async () => {
    setQueue(prev => ({ ...prev, isRunning: true }));

    const pendingItems = queue.items.filter(item => item.status === 'pending');
    const activeItems = queue.items.filter(item => item.status === 'downloading');

    // Start new downloads up to max concurrent
    const availableSlots = queue.maxConcurrent - activeItems.length;
    const itemsToStart = pendingItems.slice(0, availableSlots);

    for (const item of itemsToStart) {
      startDownload(item);
    }
  }, [queue.items, queue.maxConcurrent]);

  // Start individual download
  const startDownload = useCallback(async (item: DownloadItem) => {
    updateItemStatus(item.id, { status: 'downloading' });

    try {
      // Get presigned URL
      const response = await fetch(`/api/v1/repos/${item.repo}/download/${encodeURIComponent(item.path)}`, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
      });

      if (!response.ok) {
        throw new Error(`Failed to get download URL: ${response.statusText}`);
      }

      const data = await response.json();
      const presignedUrl = data.presigned_url;

      // Download file
      const downloadResponse = await fetch(presignedUrl);
      if (!downloadResponse.ok) {
        throw new Error(`Download failed: ${downloadResponse.statusText}`);
      }

      const blob = await downloadResponse.blob();
      
      // Create download link and trigger download
      const url = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = item.filename;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      window.URL.revokeObjectURL(url);

      // Mark as completed
      updateItemStatus(item.id, {
        status: 'completed',
        progress: 100,
        completedAt: new Date(),
      });

      // Start next download if queue is running
      if (queue.isRunning) {
        const nextPending = queue.items.find(i => i.status === 'pending');
        if (nextPending) {
          setTimeout(() => startDownload(nextPending), 1000);
        }
      }

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      
      if (item.retryCount < item.maxRetries) {
        // Retry
        updateItemStatus(item.id, {
          status: 'pending',
          retryCount: item.retryCount + 1,
          error: `Retry ${item.retryCount + 1}/${item.maxRetries}: ${errorMessage}`,
        });
      } else {
        // Mark as failed
        updateItemStatus(item.id, {
          status: 'failed',
          error: errorMessage,
        });
      }
    }
  }, [queue.isRunning, queue.items, updateItemStatus]);

  // Pause downloads
  const pauseDownloads = useCallback(() => {
    setQueue(prev => ({ ...prev, isRunning: false }));
    
    // Pause all downloading items
    queue.items.forEach(item => {
      if (item.status === 'downloading') {
        updateItemStatus(item.id, { status: 'paused' });
      }
    });
  }, [queue.items, updateItemStatus]);

  // Resume downloads
  const resumeDownloads = useCallback(() => {
    setQueue(prev => ({ ...prev, isRunning: true }));
    
    // Resume paused items
    const pausedItems = queue.items.filter(item => item.status === 'paused');
    pausedItems.forEach(item => {
      updateItemStatus(item.id, { status: 'pending' });
    });
    
    startDownloads();
  }, [queue.items, updateItemStatus, startDownloads]);

  // Clear completed items
  const clearCompleted = useCallback(() => {
    setQueue(prev => ({
      ...prev,
      items: prev.items.filter(item => item.status !== 'completed'),
    }));
  }, []);

  // Export manifest
  const exportManifest = useCallback(() => {
    const manifest = {
      exported_at: new Date().toISOString(),
      total_items: queue.items.length,
      completed_items: queue.items.filter(item => item.status === 'completed').length,
      failed_items: queue.items.filter(item => item.status === 'failed').length,
      items: queue.items.map(item => ({
        repo: item.repo,
        path: item.path,
        filename: item.filename,
        size: item.size,
        status: item.status,
        created_at: item.createdAt.toISOString(),
        completed_at: item.completedAt?.toISOString(),
      })),
    };

    const blob = new Blob([JSON.stringify(manifest, null, 2)], { type: 'application/json' });
    const url = window.URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `blacklake-download-manifest-${new Date().toISOString().split('T')[0]}.json`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    window.URL.revokeObjectURL(url);
  }, [queue.items]);

  // Get status icon
  const getStatusIcon = (status: DownloadItem['status']) => {
    switch (status) {
      case 'completed':
        return <CheckCircle className="h-4 w-4 text-green-500" />;
      case 'failed':
        return <XCircle className="h-4 w-4 text-red-500" />;
      case 'downloading':
        return <RefreshCw className="h-4 w-4 text-blue-500 animate-spin" />;
      case 'paused':
        return <Pause className="h-4 w-4 text-yellow-500" />;
      default:
        return <Download className="h-4 w-4 text-gray-500" />;
    }
  };

  // Get status badge
  const getStatusBadge = (status: DownloadItem['status']) => {
    const variants = {
      completed: 'bg-green-100 text-green-800',
      failed: 'bg-red-100 text-red-800',
      downloading: 'bg-blue-100 text-blue-800',
      paused: 'bg-yellow-100 text-yellow-800',
      pending: 'bg-gray-100 text-gray-800',
    };

    return (
      <Badge className={variants[status]}>
        {status.charAt(0).toUpperCase() + status.slice(1)}
      </Badge>
    );
  };

  // Format file size
  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const completedCount = queue.items.filter(item => item.status === 'completed').length;
  const failedCount = queue.items.filter(item => item.status === 'failed').length;
  const totalCount = queue.items.length;

  return (
    <>
      {/* Download Queue Button */}
      <Button
        variant="outline"
        size="sm"
        onClick={() => setShowQueue(true)}
        className="relative"
      >
        <Download className="h-4 w-4 mr-2" />
        Downloads
        {totalCount > 0 && (
          <Badge className="ml-2 bg-blue-500 text-white">
            {totalCount}
          </Badge>
        )}
      </Button>

      {/* Download Queue Dialog */}
      <Dialog open={showQueue} onOpenChange={setShowQueue}>
        <DialogContent className="max-w-4xl max-h-[80vh] overflow-hidden flex flex-col">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Download className="h-5 w-5" />
              Download Queue
            </DialogTitle>
            <DialogDescription>
              Manage your file downloads with progress tracking and retry logic.
            </DialogDescription>
          </DialogHeader>

          {/* Queue Controls */}
          <div className="flex items-center justify-between p-4 border-b">
            <div className="flex items-center gap-2">
              <Button
                onClick={queue.isRunning ? pauseDownloads : resumeDownloads}
                variant="outline"
                size="sm"
              >
                {queue.isRunning ? (
                  <>
                    <Pause className="h-4 w-4 mr-2" />
                    Pause
                  </>
                ) : (
                  <>
                    <Play className="h-4 w-4 mr-2" />
                    Resume
                  </>
                )}
              </Button>

              <Button
                onClick={clearCompleted}
                variant="outline"
                size="sm"
                disabled={completedCount === 0}
              >
                <Trash2 className="h-4 w-4 mr-2" />
                Clear Completed
              </Button>

              <Button
                onClick={exportManifest}
                variant="outline"
                size="sm"
              >
                <FileText className="h-4 w-4 mr-2" />
                Export Manifest
              </Button>
            </div>

            <div className="flex items-center gap-4 text-sm text-gray-600">
              <span>Total: {totalCount}</span>
              <span className="text-green-600">Completed: {completedCount}</span>
              <span className="text-red-600">Failed: {failedCount}</span>
            </div>
          </div>

          {/* Queue Table */}
          <div className="flex-1 overflow-y-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Status</TableHead>
                  <TableHead>File</TableHead>
                  <TableHead>Repository</TableHead>
                  <TableHead>Size</TableHead>
                  <TableHead>Progress</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {queue.items.map((item) => (
                  <TableRow key={item.id}>
                    <TableCell>
                      <div className="flex items-center gap-2">
                        {getStatusIcon(item.status)}
                        {getStatusBadge(item.status)}
                      </div>
                    </TableCell>
                    <TableCell>
                      <div>
                        <div className="font-medium">{item.filename}</div>
                        <div className="text-sm text-gray-500">{item.path}</div>
                      </div>
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline">{item.repo}</Badge>
                    </TableCell>
                    <TableCell>{formatFileSize(item.size)}</TableCell>
                    <TableCell>
                      <div className="w-32">
                        <Progress value={item.progress} className="h-2" />
                        <div className="text-xs text-gray-500 mt-1">
                          {item.progress}%
                        </div>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center gap-2">
                        {item.status === 'failed' && (
                          <Button
                            onClick={() => startDownload(item)}
                            variant="outline"
                            size="sm"
                          >
                            <RefreshCw className="h-3 w-3" />
                          </Button>
                        )}
                        <Button
                          onClick={() => removeFromQueue([item.id])}
                          variant="outline"
                          size="sm"
                        >
                          <Trash2 className="h-3 w-3" />
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>

          <DialogFooter>
            <Button onClick={() => setShowQueue(false)}>
              Close
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
};