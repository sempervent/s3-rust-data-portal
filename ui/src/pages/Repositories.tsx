import React, { useState } from 'react'
import { Link } from 'react-router-dom'
import { useApi } from '@/hooks/useApi'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
import { useAppStore } from '@/stores/app'

interface Repository {
  id: string
  name: string
  description?: string
  created_at: string
  updated_at: string
}

const Repositories: React.FC = () => {
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [newRepoName, setNewRepoName] = useState('')
  const [newRepoDescription, setNewRepoDescription] = useState('')
  const { addToast } = useAppStore()

  const { data: repos, isLoading, refetch } = useApi<Repository[]>('/v1/repos')
  const { mutate: createRepo } = useApi('/v1/repos', {
    method: 'POST',
    onSuccess: () => {
      addToast('Repository created successfully', 'success')
      setIsCreateDialogOpen(false)
      setNewRepoName('')
      setNewRepoDescription('')
      refetch()
    },
    onError: (error) => {
      addToast(`Failed to create repository: ${error.message}`, 'error')
    }
  })

  const handleCreateRepo = () => {
    if (!newRepoName.trim()) {
      addToast('Repository name is required', 'error')
      return
    }

    createRepo({
      name: newRepoName,
      description: newRepoDescription || undefined
    })
  }

  if (isLoading) {
    return (
      <div className="container mx-auto px-4 py-8">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
          <p>Loading repositories...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex items-center justify-between mb-8">
        <h1 className="text-3xl font-bold">Repositories</h1>
        
        <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
          <DialogTrigger asChild>
            <Button>Create Repository</Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Create New Repository</DialogTitle>
            </DialogHeader>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-2">
                  Repository Name
                </label>
                <input
                  type="text"
                  value={newRepoName}
                  onChange={(e) => setNewRepoName(e.target.value)}
                  className="w-full px-3 py-2 border border-input rounded-md"
                  placeholder="my-data-repo"
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-2">
                  Description (optional)
                </label>
                <textarea
                  value={newRepoDescription}
                  onChange={(e) => setNewRepoDescription(e.target.value)}
                  className="w-full px-3 py-2 border border-input rounded-md"
                  rows={3}
                  placeholder="Description of your repository"
                />
              </div>
              <div className="flex justify-end space-x-2">
                <Button
                  variant="outline"
                  onClick={() => setIsCreateDialogOpen(false)}
                >
                  Cancel
                </Button>
                <Button onClick={handleCreateRepo}>
                  Create Repository
                </Button>
              </div>
            </div>
          </DialogContent>
        </Dialog>
      </div>

      {repos && repos.length > 0 ? (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {repos.map((repo) => (
            <Link
              key={repo.id}
              to={`/repos/${repo.name}`}
              className="block p-6 border border-border rounded-lg hover:bg-accent transition-colors"
            >
              <h3 className="text-lg font-semibold mb-2">{repo.name}</h3>
              {repo.description && (
                <p className="text-muted-foreground text-sm mb-4">
                  {repo.description}
                </p>
              )}
              <div className="text-xs text-muted-foreground">
                Created {new Date(repo.created_at).toLocaleDateString()}
              </div>
            </Link>
          ))}
        </div>
      ) : (
        <div className="text-center py-12">
          <p className="text-muted-foreground mb-4">
            No repositories found. Create your first repository to get started.
          </p>
          <Button onClick={() => setIsCreateDialogOpen(true)}>
            Create Repository
          </Button>
        </div>
      )}
    </div>
  )
}

export default Repositories