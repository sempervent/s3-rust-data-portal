import { create } from 'zustand';
import { AppState } from '../types';

interface AppStore extends AppState {
  setSelectedRepo: (repo: string | null) => void;
  setSelectedBranch: (branch: string) => void;
  setSelectedPath: (path: string) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  selectedRepo: null,
  selectedBranch: 'main',
  selectedPath: '',

  setSelectedRepo: (repo) => set({ selectedRepo: repo }),
  setSelectedBranch: (branch) => set({ selectedBranch: branch }),
  setSelectedPath: (path) => set({ selectedPath: path }),
}));
