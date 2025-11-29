import { create } from 'zustand';
import type {
  SystemStatus,
  OverallSystemStats,
  EncodingJob,
  ClientStatus,
  LeaderboardResponse,
  LeaderboardCategory,
} from '../types/api';
import {
  fetchSystemStatus,
  fetchOverallStats,
  fetchActiveJobs,
  fetchConnectedClients,
  fetchLeaderboard,
} from '../lib/api';

interface DashboardState {
  // Data
  systemStatus: SystemStatus | null;
  overallStats: OverallSystemStats | null;
  activeJobs: EncodingJob[];
  connectedClients: ClientStatus[];
  leaderboard: LeaderboardResponse | null;
  selectedLeaderboardCategory: LeaderboardCategory;

  // UI State
  loading: boolean;
  error: string | null;
  lastUpdate: number | null;

  // Actions
  fetchSystemStatus: () => Promise<void>;
  fetchOverallStats: () => Promise<void>;
  fetchActiveJobs: () => Promise<void>;
  fetchConnectedClients: () => Promise<void>;
  fetchLeaderboard: (category?: LeaderboardCategory) => Promise<void>;
  fetchAllData: () => Promise<void>;
  setLeaderboardCategory: (category: LeaderboardCategory) => void;
  clearError: () => void;

  // SSE Update Handlers
  updateJobProgress: (jobId: string, percentage: number, speed: string) => void;
  markJobCompleted: (jobId: string) => void;
  markJobFailed: (jobId: string, error: string) => void;
  addClient: (client: ClientStatus) => void;
  removeClient: (clientId: string) => void;
}

export const useDashboardStore = create<DashboardState>((set, get) => ({
  // Initial state
  systemStatus: null,
  overallStats: null,
  activeJobs: [],
  connectedClients: [],
  leaderboard: null,
  selectedLeaderboardCategory: 'most_jobs',
  loading: false,
  error: null,
  lastUpdate: null,

  // Fetch system status
  fetchSystemStatus: async () => {
    try {
      const status = await fetchSystemStatus();
      set({ systemStatus: status, lastUpdate: Date.now(), error: null });
    } catch (error) {
      set({ error: (error as Error).message });
      console.error('Failed to fetch system status:', error);
    }
  },

  // Fetch overall statistics
  fetchOverallStats: async () => {
    try {
      const stats = await fetchOverallStats();
      set({ overallStats: stats, lastUpdate: Date.now(), error: null });
    } catch (error) {
      set({ error: (error as Error).message });
      console.error('Failed to fetch overall stats:', error);
    }
  },

  // Fetch active jobs
  fetchActiveJobs: async () => {
    try {
      const jobs = await fetchActiveJobs();
      set({ activeJobs: jobs, lastUpdate: Date.now(), error: null });
    } catch (error) {
      set({ error: (error as Error).message });
      console.error('Failed to fetch active jobs:', error);
    }
  },

  // Fetch connected clients
  fetchConnectedClients: async () => {
    try {
      const clients = await fetchConnectedClients();
      set({ connectedClients: clients, lastUpdate: Date.now(), error: null });
    } catch (error) {
      set({ error: (error as Error).message });
      console.error('Failed to fetch connected clients:', error);
    }
  },

  // Fetch leaderboard
  fetchLeaderboard: async (category?: LeaderboardCategory) => {
    const cat = category || get().selectedLeaderboardCategory;
    try {
      const leaderboard = await fetchLeaderboard(cat);
      set({ leaderboard, lastUpdate: Date.now(), error: null });
    } catch (error) {
      set({ error: (error as Error).message });
      console.error('Failed to fetch leaderboard:', error);
    }
  },

  // Fetch all data in parallel
  fetchAllData: async () => {
    set({ loading: true, error: null });
    try {
      await Promise.all([
        get().fetchSystemStatus(),
        get().fetchOverallStats(),
        get().fetchActiveJobs(),
        get().fetchConnectedClients(),
        get().fetchLeaderboard(),
      ]);
    } finally {
      set({ loading: false });
    }
  },

  // Set leaderboard category and fetch data
  setLeaderboardCategory: (category: LeaderboardCategory) => {
    set({ selectedLeaderboardCategory: category });
    get().fetchLeaderboard(category);
  },

  // Clear error
  clearError: () => set({ error: null }),

  // SSE Handlers
  updateJobProgress: (jobId: string, _percentage: number, _speed: string) => {
    set((state) => ({
      activeJobs: state.activeJobs.map((job) =>
        job.id === jobId
          ? { ...job, /* Add progress fields if needed */ }
          : job
      ),
    }));
  },

  markJobCompleted: (jobId: string) => {
    set((state) => ({
      activeJobs: state.activeJobs.filter((job) => job.id !== jobId),
    }));
    // Refresh stats after completion
    get().fetchOverallStats();
    get().fetchSystemStatus();
  },

  markJobFailed: (jobId: string, error: string) => {
    set((state) => ({
      activeJobs: state.activeJobs.map((job) =>
        job.id === jobId
          ? { ...job, status: 'failed' as const, error_message: error }
          : job
      ),
    }));
  },

  addClient: (client: ClientStatus) => {
    set((state) => ({
      connectedClients: [...state.connectedClients, client],
    }));
  },

  removeClient: (clientId: string) => {
    set((state) => ({
      connectedClients: state.connectedClients.filter((c) => c.id !== clientId),
    }));
  },
}));
