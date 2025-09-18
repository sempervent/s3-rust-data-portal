// BlackLake Help Modal
// Week 4: Keyboard shortcuts and help system

import React, { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@/components/ui/tabs';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { 
  HelpCircle, 
  Keyboard, 
  Settings, 
  Search,
  Upload,
  RefreshCw,
  Plus,
  Palette,
  Download,
  CheckSquare,
  X,
  Escape,
  Command
} from 'lucide-react';
import { useKeyboardShortcuts, KeyboardShortcut } from '@/hooks/useKeyboardShortcuts';

interface HelpModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export const HelpModal: React.FC<HelpModalProps> = ({ open, onOpenChange }) => {
  const [activeTab, setActiveTab] = useState('shortcuts');
  const [searchQuery, setSearchQuery] = useState('');
  const {
    shortcuts,
    enabled,
    getShortcutsByCategory,
    updateShortcut,
    resetShortcuts,
    toggleShortcuts,
    formatShortcut,
  } = useKeyboardShortcuts();

  const categories = getShortcutsByCategory();
  const filteredShortcuts = Object.values(shortcuts).filter(shortcut =>
    shortcut.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
    shortcut.key.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const getShortcutIcon = (shortcut: KeyboardShortcut) => {
    const key = shortcut.key.toLowerCase();
    if (key === 'search' || key === '/') return <Search className="h-4 w-4" />;
    if (key === 'upload' || key === 'u') return <Upload className="h-4 w-4" />;
    if (key === 'refresh' || key === '.') return <RefreshCw className="h-4 w-4" />;
    if (key === 'new' || key === 'n') return <Plus className="h-4 w-4" />;
    if (key === 'theme' || key === 't') return <Palette className="h-4 w-4" />;
    if (key === 'export' || key === 'e') return <Download className="h-4 w-4" />;
    if (key === 'select' || key === 'a') return <CheckSquare className="h-4 w-4" />;
    if (key === 'escape') return <X className="h-4 w-4" />;
    if (key === 'settings' || key === ',') return <Settings className="h-4 w-4" />;
    return <Keyboard className="h-4 w-4" />;
  };

  const handleKeyChange = (shortcutId: string, newKey: string) => {
    updateShortcut(shortcutId, { key: newKey });
  };

  const handleModifierChange = (shortcutId: string, modifier: string, value: boolean) => {
    const updates: Partial<KeyboardShortcut> = {};
    switch (modifier) {
      case 'ctrl':
        updates.ctrl = value;
        break;
      case 'shift':
        updates.shift = value;
        break;
      case 'alt':
        updates.alt = value;
        break;
      case 'meta':
        updates.meta = value;
        break;
    }
    updateShortcut(shortcutId, updates);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[80vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <HelpCircle className="h-5 w-5" />
            Help & Keyboard Shortcuts
          </DialogTitle>
          <DialogDescription>
            Learn about BlackLake features and customize your keyboard shortcuts.
          </DialogDescription>
        </DialogHeader>

        <Tabs value={activeTab} onValueChange={setActiveTab} className="flex-1 overflow-hidden flex flex-col">
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="shortcuts" className="flex items-center gap-2">
              <Keyboard className="h-4 w-4" />
              Shortcuts
            </TabsTrigger>
            <TabsTrigger value="features" className="flex items-center gap-2">
              <HelpCircle className="h-4 w-4" />
              Features
            </TabsTrigger>
            <TabsTrigger value="customize" className="flex items-center gap-2">
              <Settings className="h-4 w-4" />
              Customize
            </TabsTrigger>
          </TabsList>

          {/* Shortcuts Tab */}
          <TabsContent value="shortcuts" className="flex-1 overflow-hidden flex flex-col">
            <div className="space-y-4">
              {/* Search and toggle */}
              <div className="flex items-center gap-4">
                <div className="flex-1">
                  <Input
                    placeholder="Search shortcuts..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                  />
                </div>
                <div className="flex items-center space-x-2">
                  <Switch
                    id="shortcuts-enabled"
                    checked={enabled}
                    onCheckedChange={toggleShortcuts}
                  />
                  <Label htmlFor="shortcuts-enabled">Enable shortcuts</Label>
                </div>
              </div>

              {/* Shortcuts by category */}
              <div className="flex-1 overflow-y-auto space-y-4">
                {Object.entries(categories).map(([category, categoryShortcuts]) => (
                  <Card key={category}>
                    <CardHeader className="pb-3">
                      <CardTitle className="text-sm">{category}</CardTitle>
                    </CardHeader>
                    <CardContent>
                      <div className="grid gap-2">
                        {categoryShortcuts
                          .filter(shortcut => 
                            searchQuery === '' || 
                            shortcut.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
                            shortcut.key.toLowerCase().includes(searchQuery.toLowerCase())
                          )
                          .map((shortcut) => (
                            <div key={shortcut.key} className="flex items-center justify-between p-2 border rounded">
                              <div className="flex items-center gap-3">
                                {getShortcutIcon(shortcut)}
                                <span className="text-sm">{shortcut.description}</span>
                              </div>
                              <Badge variant="outline" className="font-mono text-xs">
                                {formatShortcut(shortcut)}
                              </Badge>
                            </div>
                          ))}
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </div>
          </TabsContent>

          {/* Features Tab */}
          <TabsContent value="features" className="flex-1 overflow-y-auto">
            <div className="space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Upload className="h-5 w-5" />
                    Upload & Management
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-3">
                  <div>
                    <h4 className="font-medium">Bulk Upload</h4>
                    <p className="text-sm text-gray-600">
                      Upload multiple files at once with drag-and-drop support. 
                      Metadata can be applied to all files or customized per file.
                    </p>
                  </div>
                  <div>
                    <h4 className="font-medium">Schema Validation</h4>
                    <p className="text-sm text-gray-600">
                      Automatic validation of metadata against JSON schemas. 
                      Invalid metadata is rejected with detailed error messages.
                    </p>
                  </div>
                  <div>
                    <h4 className="font-medium">Content Addressing</h4>
                    <p className="text-sm text-gray-600">
                      Files are stored using SHA256 hashes for deduplication 
                      and integrity verification.
                    </p>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Search className="h-5 w-5" />
                    Search & Discovery
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-3">
                  <div>
                    <h4 className="font-medium">Advanced Search</h4>
                    <p className="text-sm text-gray-600">
                      Search by file type, organization, tags, size, and date ranges. 
                      Full-text search across metadata and file names.
                    </p>
                  </div>
                  <div>
                    <h4 className="font-medium">Saved Views</h4>
                    <p className="text-sm text-gray-600">
                      Save frequently used search filters and column preferences 
                      for quick access.
                    </p>
                  </div>
                  <div>
                    <h4 className="font-medium">RDF Support</h4>
                    <p className="text-sm text-gray-600">
                      Automatic generation of Dublin Core RDF metadata 
                      for semantic web integration.
                    </p>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Settings className="h-5 w-5" />
                    Governance & Security
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-3">
                  <div>
                    <h4 className="font-medium">Branch Protection</h4>
                    <p className="text-sm text-gray-600">
                      Configure rules to protect branches from unauthorized changes. 
                      Require admin approval, checks, and reviewers.
                    </p>
                  </div>
                  <div>
                    <h4 className="font-medium">Storage Quotas</h4>
                    <p className="text-sm text-gray-600">
                      Set soft and hard storage limits with automatic enforcement 
                      and warning notifications.
                    </p>
                  </div>
                  <div>
                    <h4 className="font-medium">Webhooks</h4>
                    <p className="text-sm text-gray-600">
                      Real-time notifications for repository events with 
                      retry logic and dead letter handling.
                    </p>
                  </div>
                </CardContent>
              </Card>
            </div>
          </TabsContent>

          {/* Customize Tab */}
          <TabsContent value="customize" className="flex-1 overflow-y-auto">
            <div className="space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle>Customize Shortcuts</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  {Object.entries(shortcuts).map(([id, shortcut]) => (
                    <div key={id} className="flex items-center gap-4 p-3 border rounded">
                      <div className="flex-1">
                        <Label className="text-sm font-medium">{shortcut.description}</Label>
                        <p className="text-xs text-gray-500">{shortcut.category}</p>
                      </div>
                      
                      <div className="flex items-center gap-2">
                        {/* Modifiers */}
                        <div className="flex items-center gap-1">
                          <Switch
                            checked={shortcut.ctrl || false}
                            onCheckedChange={(checked) => handleModifierChange(id, 'ctrl', checked)}
                            size="sm"
                          />
                          <span className="text-xs">Ctrl</span>
                        </div>
                        
                        <div className="flex items-center gap-1">
                          <Switch
                            checked={shortcut.shift || false}
                            onCheckedChange={(checked) => handleModifierChange(id, 'shift', checked)}
                            size="sm"
                          />
                          <span className="text-xs">Shift</span>
                        </div>
                        
                        <div className="flex items-center gap-1">
                          <Switch
                            checked={shortcut.alt || false}
                            onCheckedChange={(checked) => handleModifierChange(id, 'alt', checked)}
                            size="sm"
                          />
                          <span className="text-xs">Alt</span>
                        </div>
                        
                        <div className="flex items-center gap-1">
                          <Switch
                            checked={shortcut.meta || false}
                            onCheckedChange={(checked) => handleModifierChange(id, 'meta', checked)}
                            size="sm"
                          />
                          <span className="text-xs">Cmd</span>
                        </div>
                        
                        {/* Key */}
                        <Select
                          value={shortcut.key}
                          onValueChange={(value) => handleKeyChange(id, value)}
                        >
                          <SelectTrigger className="w-20">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="a">A</SelectItem>
                            <SelectItem value="b">B</SelectItem>
                            <SelectItem value="c">C</SelectItem>
                            <SelectItem value="d">D</SelectItem>
                            <SelectItem value="e">E</SelectItem>
                            <SelectItem value="f">F</SelectItem>
                            <SelectItem value="g">G</SelectItem>
                            <SelectItem value="h">H</SelectItem>
                            <SelectItem value="i">I</SelectItem>
                            <SelectItem value="j">J</SelectItem>
                            <SelectItem value="k">K</SelectItem>
                            <SelectItem value="l">L</SelectItem>
                            <SelectItem value="m">M</SelectItem>
                            <SelectItem value="n">N</SelectItem>
                            <SelectItem value="o">O</SelectItem>
                            <SelectItem value="p">P</SelectItem>
                            <SelectItem value="q">Q</SelectItem>
                            <SelectItem value="r">R</SelectItem>
                            <SelectItem value="s">S</SelectItem>
                            <SelectItem value="t">T</SelectItem>
                            <SelectItem value="u">U</SelectItem>
                            <SelectItem value="v">V</SelectItem>
                            <SelectItem value="w">W</SelectItem>
                            <SelectItem value="x">X</SelectItem>
                            <SelectItem value="y">Y</SelectItem>
                            <SelectItem value="z">Z</SelectItem>
                            <SelectItem value="/">/</SelectItem>
                            <SelectItem value=".">.</SelectItem>
                            <SelectItem value=",">,</SelectItem>
                            <SelectItem value="?">?</SelectItem>
                            <SelectItem value="Escape">Esc</SelectItem>
                            <SelectItem value="Enter">Enter</SelectItem>
                            <SelectItem value="Space">Space</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                    </div>
                  ))}
                  
                  <div className="flex justify-end">
                    <Button variant="outline" onClick={resetShortcuts}>
                      Reset to Defaults
                    </Button>
                  </div>
                </CardContent>
              </Card>
            </div>
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
};
