import { useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { ArrowLeft, Upload, Search, File, Folder, Download, Eye } from 'lucide-react';
import { useTree, useUploadInit, useCommit, useRdf } from '../hooks/useApi';
import { useAppStore } from '../stores/app';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../components/ui/card';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '../components/ui/dialog';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { TreeEntry } from '../types';

export default function Repository() {
  const { repo } = useParams<{ repo: string }>();
  const { selectedBranch, selectedPath, setSelectedPath } = useAppStore();
  const [isUploadDialogOpen, setIsUploadDialogOpen] = useState(false);
  const [isRdfDialogOpen, setIsRdfDialogOpen] = useState(false);
  const [selectedFile, setSelectedFile] = useState<TreeEntry | null>(null);
  const [uploadPath, setUploadPath] = useState('');
  const [uploadFile, setUploadFile] = useState<File | null>(null);
  const [commitMessage, setCommitMessage] = useState('');

  const { data: tree, isLoading, error } = useTree(repo!, selectedBranch, selectedPath);
  const uploadInitMutation = useUploadInit();
  const commitMutation = useCommit();
  const { data: rdfData } = useRdf(repo!, selectedBranch, selectedFile?.path || '', 'turtle');

  const handleUpload = async () => {
    if (!uploadFile || !uploadPath.trim() || !repo) return;

    try {
      // Initialize upload
      const uploadInit = await uploadInitMutation.mutateAsync({
        repo,
        request: {
          path: uploadPath.trim(),
          size: uploadFile.size,
          media_type: uploadFile.type,
        },
      });

      // Upload file to presigned URL
      const uploadResponse = await fetch(uploadInit.upload_url, {
        method: 'PUT',
        body: uploadFile,
        headers: {
          'Content-Type': uploadFile.type,
        },
      });

      if (!uploadResponse.ok) {
        throw new Error('Upload failed');
      }

      // Create commit
      await commitMutation.mutateAsync({
        repo,
        request: {
          ref: selectedBranch,
          message: commitMessage || `Upload ${uploadPath}`,
          changes: [
            {
              op: 'put',
              path: uploadPath.trim(),
              sha256: uploadInit.sha256,
              meta: {
                name: uploadFile.name,
                size: uploadFile.size,
                type: uploadFile.type,
              },
            },
          ],
        },
      });

      setUploadPath('');
      setUploadFile(null);
      setCommitMessage('');
      setIsUploadDialogOpen(false);
    } catch (error) {
      console.error('Upload failed:', error);
    }
  };

  const handleFileClick = (entry: TreeEntry) => {
    if (entry.is_dir) {
      setSelectedPath(entry.path);
    } else {
      setSelectedFile(entry);
      setIsRdfDialogOpen(true);
    }
  };

  const handleDownload = async (entry: TreeEntry) => {
    // In a real implementation, this would get a presigned download URL
    console.log('Download:', entry.path);
  };

  if (isLoading) {
    return (
      <div className="container mx-auto p-6">
        <div className="flex items-center justify-center h-64">
          <div className="text-lg">Loading repository...</div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="container mx-auto p-6">
        <div className="text-center text-red-600">
          Failed to load repository: {error.message}
        </div>
      </div>
    );
  }

  return (
    <div className="container mx-auto p-6">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center space-x-4">
          <Link to="/repos">
            <Button variant="outline" size="sm">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Repositories
            </Button>
          </Link>
          <div>
            <h1 className="text-3xl font-bold">{repo}</h1>
            <p className="text-muted-foreground">
              Branch: {selectedBranch} • Path: {selectedPath || '/'}
            </p>
          </div>
        </div>
        <div className="flex space-x-2">
          <Button variant="outline">
            <Search className="w-4 h-4 mr-2" />
            Search
          </Button>
          <Dialog open={isUploadDialogOpen} onOpenChange={setIsUploadDialogOpen}>
            <DialogTrigger asChild>
              <Button>
                <Upload className="w-4 h-4 mr-2" />
                Upload
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Upload File</DialogTitle>
                <DialogDescription>
                  Upload a new file to the repository.
                </DialogDescription>
              </DialogHeader>
              <div className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="upload-path">File Path</Label>
                  <Input
                    id="upload-path"
                    value={uploadPath}
                    onChange={(e) => setUploadPath(e.target.value)}
                    placeholder="path/to/file.ext"
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="upload-file">File</Label>
                  <Input
                    id="upload-file"
                    type="file"
                    onChange={(e) => setUploadFile(e.target.files?.[0] || null)}
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="commit-message">Commit Message</Label>
                  <Input
                    id="commit-message"
                    value={commitMessage}
                    onChange={(e) => setCommitMessage(e.target.value)}
                    placeholder="Describe your changes"
                  />
                </div>
              </div>
              <DialogFooter>
                <Button variant="outline" onClick={() => setIsUploadDialogOpen(false)}>
                  Cancel
                </Button>
                <Button 
                  onClick={handleUpload}
                  disabled={!uploadFile || !uploadPath.trim() || uploadInitMutation.isPending || commitMutation.isPending}
                >
                  {(uploadInitMutation.isPending || commitMutation.isPending) ? 'Uploading...' : 'Upload'}
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </div>
      </div>

      {tree && tree.entries.length > 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {tree.entries.map((entry) => (
            <Card key={entry.path} className="hover:shadow-lg transition-shadow">
              <CardHeader className="pb-3">
                <CardTitle className="flex items-center text-lg">
                  {entry.is_dir ? (
                    <Folder className="w-5 h-5 mr-2 text-blue-500" />
                  ) : (
                    <File className="w-5 h-5 mr-2 text-gray-500" />
                  )}
                  {entry.path.split('/').pop()}
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-2 text-sm text-muted-foreground">
                  <div>Size: {entry.size.toLocaleString()} bytes</div>
                  <div>SHA256: {entry.sha256.substring(0, 16)}...</div>
                  {entry.meta && (
                    <div className="text-xs">
                      {Object.entries(entry.meta).slice(0, 2).map(([key, value]) => (
                        <div key={key}>{key}: {String(value)}</div>
                      ))}
                    </div>
                  )}
                </div>
                <div className="flex space-x-2 mt-4">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleFileClick(entry)}
                  >
                    {entry.is_dir ? 'Browse' : 'View'}
                  </Button>
                  {!entry.is_dir && (
                    <>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleDownload(entry)}
                      >
                        <Download className="w-4 h-4" />
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => {
                          setSelectedFile(entry);
                          setIsRdfDialogOpen(true);
                        }}
                      >
                        <Eye className="w-4 h-4" />
                      </Button>
                    </>
                  )}
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <div className="text-center py-12">
          <Folder className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
          <h3 className="text-lg font-semibold mb-2">Empty directory</h3>
          <p className="text-muted-foreground">
            This directory is empty. Upload some files to get started.
          </p>
        </div>
      )}

      {/* RDF Preview Dialog */}
      <Dialog open={isRdfDialogOpen} onOpenChange={setIsRdfDialogOpen}>
        <DialogContent className="max-w-4xl">
          <DialogHeader>
            <DialogTitle>RDF Preview</DialogTitle>
            <DialogDescription>
              Dublin Core metadata for {selectedFile?.path}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            {rdfData ? (
              <pre className="bg-gray-100 p-4 rounded-md text-sm overflow-auto max-h-96">
                {rdfData}
              </pre>
            ) : (
              <div className="text-center py-8 text-muted-foreground">
                Loading RDF data...
              </div>
            )}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setIsRdfDialogOpen(false)}>
              Close
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
