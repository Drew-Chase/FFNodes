import { create } from 'zustand';

export interface EncodingJob {
  id: string;
  media_file_path: string;
  status: string;
  priority: number;
}

export interface CurrentJobProgress {
  frame: number;
  totalFrames: number;
  fps: number;
  bitrate: number;
  speed: number;
  percentage: number;
  extractedFrame: string | null; // Base64 or URL to extracted frame
}

interface JobStore {
  currentJob: EncodingJob | null;
  currentProgress: CurrentJobProgress | null;
  jobQueue: EncodingJob[];
  isProcessing: boolean;

  setCurrentJob: (job: EncodingJob | null) => void;
  updateProgress: (progress: Partial<CurrentJobProgress|null>) => void;
  setJobQueue: (jobs: EncodingJob[]) => void;
  addToQueue: (job: EncodingJob) => void;
  removeFromQueue: (jobId: string) => void;
  setProcessing: (isProcessing: boolean) => void;
  clearCurrentJob: () => void;
}

export const useJobStore = create<JobStore>((set) => ({
  currentJob: null,
  currentProgress: null,
  jobQueue: [],
  isProcessing: false,

  setCurrentJob: (job) => set({ currentJob: job }),

  updateProgress: (progress) =>
    set((state) => ({
      currentProgress: state.currentProgress
        ? { ...state.currentProgress, ...progress }
        : {
            frame: 0,
            totalFrames: 0,
            fps: 0,
            bitrate: 0,
            speed: 0,
            percentage: 0,
            extractedFrame: null,
            ...progress,
          },
    })),

  setJobQueue: (jobs) => set({ jobQueue: jobs }),

  addToQueue: (job) =>
    set((state) => ({
      jobQueue: [...state.jobQueue, job],
    })),

  removeFromQueue: (jobId) =>
    set((state) => ({
      jobQueue: state.jobQueue.filter((j) => j.id !== jobId),
    })),

  setProcessing: (isProcessing) => set({ isProcessing }),

  clearCurrentJob: () => set({ currentJob: null, currentProgress: null }),
}));
