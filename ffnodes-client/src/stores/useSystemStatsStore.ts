import { create } from 'zustand';

export interface OverallSystemStats {
  total_media_files: number;
  processed_files: number;
  pending_files: number;
  total_storage_bytes: number;
  total_saved_bytes: number;
  total_processing_time_seconds: number;
  average_encoding_speed: number;
  total_jobs_completed: number;
  total_jobs_failed: number;
}

export interface SystemStatus {
  total_media_files: number;
  pending_jobs: number;
  active_jobs: number;
  connected_clients: number;
}

export interface ScanProgress {
  total_files: number;
  completed_files: number;
  current_file: string | null;
  operation: string;
}

export interface ScanFileLog {
  file: string;
  timestamp: number;
}

interface SystemStatsState {
  // Data
  overallStats: OverallSystemStats | null;
  systemStatus: SystemStatus | null;
  scanProgress: ScanProgress | null;
  scanFileHistory: ScanFileLog[];
  isScanActive: boolean;

  // UI State
  loading: boolean;
  error: string | null;
  lastUpdate: number | null;

  // Actions
  setOverallStats: (stats: OverallSystemStats | null) => void;
  setSystemStatus: (status: SystemStatus | null) => void;
  updateScanProgress: (progress: ScanProgress) => void;
  addScanFile: (file: string) => void;
  clearScanProgress: () => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  clearError: () => void;
}

export const useSystemStatsStore = create<SystemStatsState>((set, get) => ({
  // Initial state
  overallStats: null,
  systemStatus: null,
  scanProgress: null,
  scanFileHistory: [],
  isScanActive: false,
  loading: false,
  error: null,
  lastUpdate: null,

  // Actions
  setOverallStats: (stats) => set({ overallStats: stats, lastUpdate: Date.now(), error: null }),

  setSystemStatus: (status) => set({ systemStatus: status, lastUpdate: Date.now(), error: null }),

  updateScanProgress: (progress) => {
    const isComplete = progress.operation === 'Complete';
    set({
      scanProgress: progress,
      isScanActive: !isComplete,
    });

    // Auto-clear after 5 seconds when scan completes
    if (isComplete) {
      setTimeout(() => {
        get().clearScanProgress();
      }, 5000);
    }
  },

  addScanFile: (file) => {
    set((state) => {
      const newLog: ScanFileLog = {
        file,
        timestamp: Date.now(),
      };

      // Keep only last 10 files in history (performance limit)
      const updatedHistory = [newLog, ...state.scanFileHistory].slice(0, 10);

      return { scanFileHistory: updatedHistory };
    });
  },

  clearScanProgress: () => {
    set({
      scanProgress: null,
      scanFileHistory: [],
      isScanActive: false,
    });
  },

  setLoading: (loading) => set({ loading }),

  setError: (error) => set({ error }),

  clearError: () => set({ error: null }),
}));
