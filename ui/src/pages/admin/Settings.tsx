// BlackLake Admin Settings
// Week 4: Comprehensive admin console with settings tabs

import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Checkbox } from '@/components/ui/checkbox';
import { Textarea } from '@/components/ui/textarea';
import { Switch } from '@/components/ui/switch';
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
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { 
  Shield, 
  HardDrive, 
  Webhook, 
  Users, 
  Settings as SettingsIcon,
  Save,
  RefreshCw,
  AlertTriangle,
  CheckCircle,
  Info,
  HelpCircle
} from 'lucide-react';
import { useToast } from '@/hooks/use-toast';

export interface BranchProtection {
  require_admin: boolean;
  allow_fast_forward: boolean;
  allow_delete: boolean;
  required_checks: string[];
  required_reviewers: number;
  require_schema_pass: boolean;
}

export interface QuotaSettings {
  bytes_soft: number;
  bytes_hard: number;
  retention_days: number;
  legal_hold: boolean;
}

export interface WebhookConfig {
  url: string;
  secret: string;
  events: string[];
  active: boolean;
}

export interface RepoSettings {
  id: string;
  name: string;
  branch_protection: BranchProtection;
  quota: QuotaSettings;
  webhooks: WebhookConfig[];
  features: Record<string, boolean>;
}

interface SettingsProps {
  repo: string;
  className?: string;
}

export const Settings: React.FC<SettingsProps> = ({ repo, className }) => {
  const [settings, setSettings] = useState<RepoSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [activeTab, setActiveTab] = useState('protection');
  const [showHelp, setShowHelp] = useState(false);
  const { toast } = useToast();

  // Load settings on mount
  useEffect(() => {
    loadSettings();
  }, [repo]);

  const loadSettings = async () => {
    try {
      const response = await fetch(`/api/v1/repos/${repo}/settings`, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
      });

      if (response.ok) {
        const data = await response.json();
        setSettings(data);
      } else {
        // Load default settings if none exist
        setSettings({
          id: '',
          name: repo,
          branch_protection: {
            require_admin: false,
            allow_fast_forward: true,
            allow_delete: true,
            required_checks: [],
            required_reviewers: 0,
            require_schema_pass: false,
          },
          quota: {
            bytes_soft: 1024 * 1024 * 1024, // 1GB
            bytes_hard: 10 * 1024 * 1024 * 1024, // 10GB
            retention_days: 30,
            legal_hold: false,
          },
          webhooks: [],
          features: {
            auto_rdf: false,
            lineage_tracking: false,
            schema_validation: true,
          },
        });
      }
    } catch (error) {
      console.error('Failed to load settings:', error);
      toast({
        title: "Failed to load settings",
        description: "Could not load repository settings. Please try again.",
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };

  const saveSettings = async () => {
    if (!settings) return;

    setSaving(true);
    try {
      const response = await fetch(`/api/v1/repos/${repo}/settings`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
        body: JSON.stringify(settings),
      });

      if (response.ok) {
        toast({
          title: "Settings saved",
          description: "Repository settings have been updated successfully.",
        });
      } else {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
    } catch (error) {
      toast({
        title: "Failed to save settings",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "destructive",
      });
    } finally {
      setSaving(false);
    }
  };

  const updateBranchProtection = (updates: Partial<BranchProtection>) => {
    if (!settings) return;
    setSettings({
      ...settings,
      branch_protection: { ...settings.branch_protection, ...updates },
    });
  };

  const updateQuota = (updates: Partial<QuotaSettings>) => {
    if (!settings) return;
    setSettings({
      ...settings,
      quota: { ...settings.quota, ...updates },
    });
  };

  const updateFeatures = (key: string, value: boolean) => {
    if (!settings) return;
    setSettings({
      ...settings,
      features: { ...settings.features, [key]: value },
    });
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const getProtectionStatus = () => {
    if (!settings) return 'disabled';
    const bp = settings.branch_protection;
    if (bp.require_admin || bp.required_checks.length > 0 || bp.required_reviewers > 0) {
      return 'enabled';
    }
    return 'disabled';
  };

  const getQuotaStatus = () => {
    if (!settings) return 'unlimited';
    if (settings.quota.bytes_hard > 0) return 'limited';
    return 'unlimited';
  };

  if (loading) {
    return (
      <div className={`space-y-6 ${className}`}>
        <div className="flex items-center justify-between">
          <h2 className="text-2xl font-bold">Repository Settings</h2>
          <Button variant="outline" size="sm" disabled>
            <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
            Loading...
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

  if (!settings) {
    return (
      <div className={`space-y-6 ${className}`}>
        <div className="text-center py-8">
          <AlertTriangle className="h-12 w-12 mx-auto mb-4 text-red-500" />
          <h3 className="text-lg font-medium mb-2">Failed to load settings</h3>
          <p className="text-gray-500 mb-4">
            Could not load repository settings. Please try again.
          </p>
          <Button onClick={loadSettings}>
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
          <h2 className="text-2xl font-bold">Repository Settings</h2>
          <p className="text-gray-600">Configure {repo} repository settings</p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowHelp(true)}
          >
            <HelpCircle className="h-4 w-4 mr-2" />
            Help
          </Button>
          <Button
            onClick={saveSettings}
            disabled={saving}
            className="flex items-center gap-2"
          >
            {saving ? (
              <>
                <RefreshCw className="h-4 w-4 animate-spin" />
                Saving...
              </>
            ) : (
              <>
                <Save className="h-4 w-4" />
                Save Settings
              </>
            )}
          </Button>
        </div>
      </div>

      {/* Status Overview */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center gap-2">
              <Shield className="h-4 w-4" />
              Branch Protection
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-2">
              <Badge className={
                getProtectionStatus() === 'enabled' 
                  ? 'bg-green-100 text-green-800' 
                  : 'bg-gray-100 text-gray-800'
              }>
                {getProtectionStatus() === 'enabled' ? 'Enabled' : 'Disabled'}
              </Badge>
              {settings.branch_protection.required_checks.length > 0 && (
                <span className="text-xs text-gray-500">
                  {settings.branch_protection.required_checks.length} checks
                </span>
              )}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center gap-2">
              <HardDrive className="h-4 w-4" />
              Storage Quota
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-2">
              <Badge className={
                getQuotaStatus() === 'limited' 
                  ? 'bg-blue-100 text-blue-800' 
                  : 'bg-gray-100 text-gray-800'
              }>
                {getQuotaStatus() === 'limited' ? 'Limited' : 'Unlimited'}
              </Badge>
              {getQuotaStatus() === 'limited' && (
                <span className="text-xs text-gray-500">
                  {formatBytes(settings.quota.bytes_hard)}
                </span>
              )}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center gap-2">
              <Webhook className="h-4 w-4" />
              Webhooks
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-2">
              <Badge className={
                settings.webhooks.length > 0 
                  ? 'bg-purple-100 text-purple-800' 
                  : 'bg-gray-100 text-gray-800'
              }>
                {settings.webhooks.length} configured
              </Badge>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Settings Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-4">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="protection" className="flex items-center gap-2">
            <Shield className="h-4 w-4" />
            Protection
          </TabsTrigger>
          <TabsTrigger value="storage" className="flex items-center gap-2">
            <HardDrive className="h-4 w-4" />
            Storage
          </TabsTrigger>
          <TabsTrigger value="webhooks" className="flex items-center gap-2">
            <Webhook className="h-4 w-4" />
            Webhooks
          </TabsTrigger>
          <TabsTrigger value="features" className="flex items-center gap-2">
            <SettingsIcon className="h-4 w-4" />
            Features
          </TabsTrigger>
        </TabsList>

        {/* Branch Protection Tab */}
        <TabsContent value="protection" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Shield className="h-5 w-5" />
                Branch Protection Rules
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="flex items-center space-x-2">
                <Switch
                  id="require-admin"
                  checked={settings.branch_protection.require_admin}
                  onCheckedChange={(checked) => updateBranchProtection({ require_admin: checked })}
                />
                <Label htmlFor="require-admin">Require admin approval for commits</Label>
              </div>

              <div className="flex items-center space-x-2">
                <Switch
                  id="allow-fast-forward"
                  checked={settings.branch_protection.allow_fast_forward}
                  onCheckedChange={(checked) => updateBranchProtection({ allow_fast_forward: checked })}
                />
                <Label htmlFor="allow-fast-forward">Allow fast-forward merges</Label>
              </div>

              <div className="flex items-center space-x-2">
                <Switch
                  id="allow-delete"
                  checked={settings.branch_protection.allow_delete}
                  onCheckedChange={(checked) => updateBranchProtection({ allow_delete: checked })}
                />
                <Label htmlFor="allow-delete">Allow branch deletion</Label>
              </div>

              <div className="flex items-center space-x-2">
                <Switch
                  id="require-schema-pass"
                  checked={settings.branch_protection.require_schema_pass}
                  onCheckedChange={(checked) => updateBranchProtection({ require_schema_pass: checked })}
                />
                <Label htmlFor="require-schema-pass">Require schema validation to pass</Label>
              </div>

              <div>
                <Label htmlFor="required-reviewers">Required reviewers</Label>
                <Input
                  id="required-reviewers"
                  type="number"
                  min="0"
                  value={settings.branch_protection.required_reviewers}
                  onChange={(e) => updateBranchProtection({ 
                    required_reviewers: parseInt(e.target.value) || 0 
                  })}
                  className="mt-1"
                />
              </div>

              <div>
                <Label htmlFor="required-checks">Required checks (comma-separated)</Label>
                <Input
                  id="required-checks"
                  value={settings.branch_protection.required_checks.join(', ')}
                  onChange={(e) => updateBranchProtection({ 
                    required_checks: e.target.value.split(',').map(s => s.trim()).filter(Boolean)
                  })}
                  placeholder="e.g., lint, test, security-scan"
                  className="mt-1"
                />
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Storage Tab */}
        <TabsContent value="storage" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <HardDrive className="h-5 w-5" />
                Storage & Retention
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              <div>
                <Label htmlFor="soft-quota">Soft quota (warning threshold)</Label>
                <Input
                  id="soft-quota"
                  type="number"
                  value={settings.quota.bytes_soft}
                  onChange={(e) => updateQuota({ 
                    bytes_soft: parseInt(e.target.value) || 0 
                  })}
                  className="mt-1"
                />
                <p className="text-xs text-gray-500 mt-1">
                  Current: {formatBytes(settings.quota.bytes_soft)}
                </p>
              </div>

              <div>
                <Label htmlFor="hard-quota">Hard quota (enforcement limit)</Label>
                <Input
                  id="hard-quota"
                  type="number"
                  value={settings.quota.bytes_hard}
                  onChange={(e) => updateQuota({ 
                    bytes_hard: parseInt(e.target.value) || 0 
                  })}
                  className="mt-1"
                />
                <p className="text-xs text-gray-500 mt-1">
                  Current: {formatBytes(settings.quota.bytes_hard)}
                </p>
              </div>

              <div>
                <Label htmlFor="retention-days">Retention period (days)</Label>
                <Input
                  id="retention-days"
                  type="number"
                  min="0"
                  value={settings.quota.retention_days}
                  onChange={(e) => updateQuota({ 
                    retention_days: parseInt(e.target.value) || 0 
                  })}
                  className="mt-1"
                />
                <p className="text-xs text-gray-500 mt-1">
                  Artifacts will be automatically deleted after this period
                </p>
              </div>

              <div className="flex items-center space-x-2">
                <Switch
                  id="legal-hold"
                  checked={settings.quota.legal_hold}
                  onCheckedChange={(checked) => updateQuota({ legal_hold: checked })}
                />
                <Label htmlFor="legal-hold">Legal hold (prevent deletion)</Label>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Webhooks Tab */}
        <TabsContent value="webhooks" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Webhook className="h-5 w-5" />
                Webhook Configuration
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-center py-8">
                <Webhook className="h-12 w-12 mx-auto mb-4 text-gray-400" />
                <h3 className="text-lg font-medium mb-2">Webhook Management</h3>
                <p className="text-gray-500 mb-4">
                  Configure webhooks to receive notifications about repository events.
                </p>
                <Button>
                  <Webhook className="h-4 w-4 mr-2" />
                  Add Webhook
                </Button>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Features Tab */}
        <TabsContent value="features" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <SettingsIcon className="h-5 w-5" />
                Repository Features
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="flex items-center space-x-2">
                <Switch
                  id="auto-rdf"
                  checked={settings.features.auto_rdf}
                  onCheckedChange={(checked) => updateFeatures('auto_rdf', checked)}
                />
                <Label htmlFor="auto-rdf">Automatic RDF generation</Label>
              </div>

              <div className="flex items-center space-x-2">
                <Switch
                  id="lineage-tracking"
                  checked={settings.features.lineage_tracking}
                  onCheckedChange={(checked) => updateFeatures('lineage_tracking', checked)}
                />
                <Label htmlFor="lineage-tracking">Data lineage tracking</Label>
              </div>

              <div className="flex items-center space-x-2">
                <Switch
                  id="schema-validation"
                  checked={settings.features.schema_validation}
                  onCheckedChange={(checked) => updateFeatures('schema_validation', checked)}
                />
                <Label htmlFor="schema-validation">Schema validation</Label>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* Help Dialog */}
      <Dialog open={showHelp} onOpenChange={setShowHelp}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Repository Settings Help</DialogTitle>
            <DialogDescription>
              Learn about the different settings and their effects on your repository.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <h4 className="font-medium mb-2">Branch Protection</h4>
              <p className="text-sm text-gray-600">
                Configure rules to protect branches from unauthorized changes. 
                Enable admin approval, require checks to pass, and set reviewer requirements.
              </p>
            </div>
            <div>
              <h4 className="font-medium mb-2">Storage & Retention</h4>
              <p className="text-sm text-gray-600">
                Set storage quotas and retention policies. Soft quotas trigger warnings, 
                hard quotas prevent new uploads. Legal hold prevents automatic deletion.
              </p>
            </div>
            <div>
              <h4 className="font-medium mb-2">Webhooks</h4>
              <p className="text-sm text-gray-600">
                Configure webhooks to receive real-time notifications about repository events 
                like commits, artifact uploads, and policy violations.
              </p>
            </div>
            <div>
              <h4 className="font-medium mb-2">Features</h4>
              <p className="text-sm text-gray-600">
                Enable or disable repository features like automatic RDF generation, 
                data lineage tracking, and schema validation.
              </p>
            </div>
          </div>
          <DialogFooter>
            <Button onClick={() => setShowHelp(false)}>Close</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
};
