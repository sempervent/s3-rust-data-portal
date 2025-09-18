// BlackLake Saved Views
// Week 5: Server-persisted saved search views

import React, { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
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
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { 
  Bookmark, 
  Plus, 
  Edit, 
  Trash2, 
  Share, 
  Copy, 
  Eye,
  Calendar,
  User,
  Tag,
  Search
} from 'lucide-react';
import { useToast } from '@/hooks/use-toast';

export interface SavedView {
  id: string;
  name: string;
  description?: string;
  query: string;
  filters: Record<string, any>;
  columns: string[];
  sort_by?: string;
  sort_order?: 'asc' | 'desc';
  is_public: boolean;
  tags: string[];
  created_at: string;
  updated_at: string;
  created_by: string;
  usage_count: number;
  last_used?: string;
}

export interface SavedViewForm {
  name: string;
  description: string;
  query: string;
  filters: Record<string, any>;
  columns: string[];
  sort_by: string;
  sort_order: 'asc' | 'desc';
  is_public: boolean;
  tags: string[];
}

interface SavedViewsProps {
  className?: string;
  onLoadView?: (view: SavedView) => void;
}

export const SavedViews: React.FC<SavedViewsProps> = ({ className, onLoadView }) => {
  const [views, setViews] = useState<SavedView[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [showEditDialog, setShowEditDialog] = useState(false);
  const [editingView, setEditingView] = useState<SavedView | null>(null);
  const [form, setForm] = useState<SavedViewForm>({
    name: '',
    description: '',
    query: '',
    filters: {},
    columns: [],
    sort_by: 'created_at',
    sort_order: 'desc',
    is_public: false,
    tags: [],
  });
  const [tagInput, setTagInput] = useState('');
  const { toast } = useToast();

  // Load saved views on mount
  useEffect(() => {
    loadSavedViews();
  }, []);

  const loadSavedViews = async () => {
    try {
      const response = await fetch('/api/v1/users/me/views', {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
      });

      if (response.ok) {
        const data = await response.json();
        setViews(data.data || []);
      } else {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
    } catch (error) {
      console.error('Failed to load saved views:', error);
      toast({
        title: "Failed to load saved views",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };

  const saveView = async (viewData: SavedViewForm) => {
    try {
      const url = editingView 
        ? `/api/v1/users/me/views/${editingView.id}`
        : '/api/v1/users/me/views';
      
      const method = editingView ? 'PUT' : 'POST';
      
      const response = await fetch(url, {
        method,
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
        body: JSON.stringify(viewData),
      });

      if (response.ok) {
        toast({
          title: editingView ? "View updated" : "View saved",
          description: `"${viewData.name}" has been ${editingView ? 'updated' : 'saved'} successfully.`,
        });
        
        await loadSavedViews();
        setShowCreateDialog(false);
        setShowEditDialog(false);
        setEditingView(null);
        resetForm();
      } else {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
    } catch (error) {
      toast({
        title: "Failed to save view",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "destructive",
      });
    }
  };

  const deleteView = async (viewId: string) => {
    try {
      const response = await fetch(`/api/v1/users/me/views/${viewId}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
      });

      if (response.ok) {
        toast({
          title: "View deleted",
          description: "The saved view has been deleted successfully.",
        });
        
        await loadSavedViews();
      } else {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
    } catch (error) {
      toast({
        title: "Failed to delete view",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "destructive",
      });
    }
  };

  const loadView = (view: SavedView) => {
    onLoadView?.(view);
    toast({
      title: "View loaded",
      description: `"${view.name}" has been loaded.`,
    });
  };

  const shareView = (view: SavedView) => {
    const shareUrl = `${window.location.origin}/search?view=${view.id}`;
    navigator.clipboard.writeText(shareUrl);
    toast({
      title: "Share link copied",
      description: "The share link has been copied to your clipboard.",
    });
  };

  const editView = (view: SavedView) => {
    setEditingView(view);
    setForm({
      name: view.name,
      description: view.description || '',
      query: view.query,
      filters: view.filters,
      columns: view.columns,
      sort_by: view.sort_by || 'created_at',
      sort_order: view.sort_order || 'desc',
      is_public: view.is_public,
      tags: view.tags,
    });
    setShowEditDialog(true);
  };

  const resetForm = () => {
    setForm({
      name: '',
      description: '',
      query: '',
      filters: {},
      columns: [],
      sort_by: 'created_at',
      sort_order: 'desc',
      is_public: false,
      tags: [],
    });
    setTagInput('');
  };

  const addTag = () => {
    if (tagInput.trim() && !form.tags.includes(tagInput.trim())) {
      setForm(prev => ({
        ...prev,
        tags: [...prev.tags, tagInput.trim()],
      }));
      setTagInput('');
    }
  };

  const removeTag = (tagToRemove: string) => {
    setForm(prev => ({
      ...prev,
      tags: prev.tags.filter(tag => tag !== tagToRemove),
    }));
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString();
  };

  const formatUsageCount = (count: number) => {
    if (count === 0) return 'Never used';
    if (count === 1) return 'Used once';
    return `Used ${count} times`;
  };

  if (loading) {
    return (
      <div className={`space-y-4 ${className}`}>
        <div className="flex items-center justify-between">
          <h2 className="text-2xl font-bold">Saved Views</h2>
          <Button disabled>
            <Plus className="h-4 w-4 mr-2" />
            Create View
          </Button>
        </div>
        <div className="text-center py-8">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mx-auto"></div>
          <p className="text-gray-500 mt-2">Loading saved views...</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Saved Views</h2>
          <p className="text-gray-600">Manage your saved search views</p>
        </div>
        <Button onClick={() => setShowCreateDialog(true)}>
          <Plus className="h-4 w-4 mr-2" />
          Create View
        </Button>
      </div>

      {/* Views List */}
      {views.length === 0 ? (
        <Card>
          <CardContent className="text-center py-8">
            <Bookmark className="h-12 w-12 mx-auto mb-4 text-gray-400" />
            <h3 className="text-lg font-medium mb-2">No saved views</h3>
            <p className="text-gray-500 mb-4">
              Create your first saved view to quickly access frequently used searches.
            </p>
            <Button onClick={() => setShowCreateDialog(true)}>
              <Plus className="h-4 w-4 mr-2" />
              Create Your First View
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {views.map((view) => (
            <Card key={view.id} className="hover:shadow-md transition-shadow">
              <CardHeader className="pb-3">
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <CardTitle className="text-lg">{view.name}</CardTitle>
                    {view.description && (
                      <p className="text-sm text-gray-600 mt-1">{view.description}</p>
                    )}
                  </div>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant="ghost" size="sm">
                        <MoreVertical className="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                      <DropdownMenuItem onClick={() => loadView(view)}>
                        <Eye className="h-4 w-4 mr-2" />
                        Load View
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={() => editView(view)}>
                        <Edit className="h-4 w-4 mr-2" />
                        Edit
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={() => shareView(view)}>
                        <Share className="h-4 w-4 mr-2" />
                        Share
                      </DropdownMenuItem>
                      <DropdownMenuSeparator />
                      <DropdownMenuItem 
                        onClick={() => deleteView(view.id)}
                        className="text-red-600"
                      >
                        <Trash2 className="h-4 w-4 mr-2" />
                        Delete
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
              </CardHeader>
              <CardContent className="space-y-3">
                {/* Tags */}
                {view.tags.length > 0 && (
                  <div className="flex flex-wrap gap-1">
                    {view.tags.map((tag) => (
                      <Badge key={tag} variant="secondary" className="text-xs">
                        <Tag className="h-3 w-3 mr-1" />
                        {tag}
                      </Badge>
                    ))}
                  </div>
                )}

                {/* Query */}
                <div className="text-sm">
                  <span className="font-medium">Query:</span>
                  <span className="text-gray-600 ml-1">
                    {view.query || 'No query'}
                  </span>
                </div>

                {/* Usage Stats */}
                <div className="flex items-center justify-between text-sm text-gray-500">
                  <div className="flex items-center gap-1">
                    <User className="h-3 w-3" />
                    {view.created_by}
                  </div>
                  <div className="flex items-center gap-1">
                    <Calendar className="h-3 w-3" />
                    {formatDate(view.created_at)}
                  </div>
                </div>

                <div className="text-sm text-gray-500">
                  {formatUsageCount(view.usage_count)}
                </div>

                {/* Actions */}
                <div className="flex gap-2">
                  <Button
                    onClick={() => loadView(view)}
                    size="sm"
                    className="flex-1"
                  >
                    <Search className="h-3 w-3 mr-1" />
                    Load
                  </Button>
                  <Button
                    onClick={() => shareView(view)}
                    variant="outline"
                    size="sm"
                  >
                    <Share className="h-3 w-3" />
                  </Button>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Create View Dialog */}
      <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Create Saved View</DialogTitle>
            <DialogDescription>
              Save your current search configuration for quick access later.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4">
            <div>
              <Label htmlFor="name">Name</Label>
              <Input
                id="name"
                value={form.name}
                onChange={(e) => setForm(prev => ({ ...prev, name: e.target.value }))}
                placeholder="Enter view name"
              />
            </div>

            <div>
              <Label htmlFor="description">Description</Label>
              <Textarea
                id="description"
                value={form.description}
                onChange={(e) => setForm(prev => ({ ...prev, description: e.target.value }))}
                placeholder="Enter view description (optional)"
                rows={3}
              />
            </div>

            <div>
              <Label htmlFor="query">Search Query</Label>
              <Input
                id="query"
                value={form.query}
                onChange={(e) => setForm(prev => ({ ...prev, query: e.target.value }))}
                placeholder="Enter search query"
              />
            </div>

            <div>
              <Label htmlFor="tags">Tags</Label>
              <div className="flex gap-2">
                <Input
                  id="tags"
                  value={tagInput}
                  onChange={(e) => setTagInput(e.target.value)}
                  placeholder="Enter tag"
                  onKeyPress={(e) => e.key === 'Enter' && (e.preventDefault(), addTag())}
                />
                <Button onClick={addTag} variant="outline">
                  Add
                </Button>
              </div>
              {form.tags.length > 0 && (
                <div className="flex flex-wrap gap-1 mt-2">
                  {form.tags.map((tag) => (
                    <Badge key={tag} variant="secondary" className="text-xs">
                      {tag}
                      <button
                        onClick={() => removeTag(tag)}
                        className="ml-1 hover:text-red-500"
                      >
                        ×
                      </button>
                    </Badge>
                  ))}
                </div>
              )}
            </div>

            <div className="flex items-center space-x-2">
              <input
                type="checkbox"
                id="is_public"
                checked={form.is_public}
                onChange={(e) => setForm(prev => ({ ...prev, is_public: e.target.checked }))}
                className="rounded"
              />
              <Label htmlFor="is_public">Make this view public</Label>
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreateDialog(false)}>
              Cancel
            </Button>
            <Button onClick={() => saveView(form)}>
              Save View
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Edit View Dialog */}
      <Dialog open={showEditDialog} onOpenChange={setShowEditDialog}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Edit Saved View</DialogTitle>
            <DialogDescription>
              Update your saved view configuration.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4">
            <div>
              <Label htmlFor="edit-name">Name</Label>
              <Input
                id="edit-name"
                value={form.name}
                onChange={(e) => setForm(prev => ({ ...prev, name: e.target.value }))}
                placeholder="Enter view name"
              />
            </div>

            <div>
              <Label htmlFor="edit-description">Description</Label>
              <Textarea
                id="edit-description"
                value={form.description}
                onChange={(e) => setForm(prev => ({ ...prev, description: e.target.value }))}
                placeholder="Enter view description (optional)"
                rows={3}
              />
            </div>

            <div>
              <Label htmlFor="edit-query">Search Query</Label>
              <Input
                id="edit-query"
                value={form.query}
                onChange={(e) => setForm(prev => ({ ...prev, query: e.target.value }))}
                placeholder="Enter search query"
              />
            </div>

            <div>
              <Label htmlFor="edit-tags">Tags</Label>
              <div className="flex gap-2">
                <Input
                  id="edit-tags"
                  value={tagInput}
                  onChange={(e) => setTagInput(e.target.value)}
                  placeholder="Enter tag"
                  onKeyPress={(e) => e.key === 'Enter' && (e.preventDefault(), addTag())}
                />
                <Button onClick={addTag} variant="outline">
                  Add
                </Button>
              </div>
              {form.tags.length > 0 && (
                <div className="flex flex-wrap gap-1 mt-2">
                  {form.tags.map((tag) => (
                    <Badge key={tag} variant="secondary" className="text-xs">
                      {tag}
                      <button
                        onClick={() => removeTag(tag)}
                        className="ml-1 hover:text-red-500"
                      >
                        ×
                      </button>
                    </Badge>
                  ))}
                </div>
              )}
            </div>

            <div className="flex items-center space-x-2">
              <input
                type="checkbox"
                id="edit-is_public"
                checked={form.is_public}
                onChange={(e) => setForm(prev => ({ ...prev, is_public: e.target.checked }))}
                className="rounded"
              />
              <Label htmlFor="edit-is_public">Make this view public</Label>
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setShowEditDialog(false)}>
              Cancel
            </Button>
            <Button onClick={() => saveView(form)}>
              Update View
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
};