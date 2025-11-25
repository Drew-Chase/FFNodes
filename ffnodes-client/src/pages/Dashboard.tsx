import { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useNavigate } from 'react-router-dom';
import { addToast } from '@heroui/toast';
import {ClientConfig, useConfigStore} from "../stores/useConfigStore";
import {EncodingJob, useJobStore} from "../stores/useJobStore";
import { VideoBackground } from '../components/VideoBackground';
import { CurrentJob } from '../components/CurrentJob';
import { VideoList } from '../components/VideoList';
import { Button } from '../components/ui';

export function Dashboard() {
  const navigate = useNavigate();
  const { config, setConfig } = useConfigStore();
  const { setJobQueue, setCurrentJob, updateProgress, setProcessing } = useJobStore();
  const [isLoading, setIsLoading] = useState(true);
  const [gpuInfo, setGpuInfo] = useState<any>(null);
  const [isPaused, setIsPaused] = useState(false);

  // Load config and GPU info on mount, start job processing
  useEffect(() => {
    const loadInitialData = async () => {
      try {
        // Load saved config
        const savedConfig: ClientConfig = await invoke('load_config');
        if (!savedConfig) {
          // No config found, redirect to setup
          navigate('/setup');
          return;
        }
        setConfig(savedConfig);

        // Get GPU info
        const gpu = await invoke('get_gpu_info');
        setGpuInfo(gpu);

        // Load active jobs
        const jobs: EncodingJob[] = await invoke('get_active_jobs', { config: savedConfig });
        setJobQueue(jobs || []);

        // Start job processing
        await invoke('start_job_processing', { config: savedConfig, gpu });
        setProcessing(true);

        addToast({
          title: 'Success',
          description: 'Connected to server! Job processing started.',
          color: 'success'
        });
      } catch (error) {
        addToast({
          title: 'Error',
          description: `Failed to load: ${error}`,
          color: 'danger'
        });
      } finally {
        setIsLoading(false);
      }
    };

    loadInitialData();

    // Cleanup: Stop job processing on unmount
    return () => {
      invoke('stop_job_processing').catch(console.error);
    };
  }, [navigate, setConfig, setJobQueue, setProcessing]);

  // Set up event listeners for job updates
  useEffect(() => {
    const unlistenPromises: Promise<() => void>[] = [];

    // Listen for job started
    unlistenPromises.push(
      listen('job-started', (event: any) => {
        console.log('Job started:', event.payload);
        setCurrentJob(event.payload);
        addToast({
          title: 'Info',
          description: `Started encoding: ${event.payload.file_name}`,
          color: 'primary'
        });
      })
    );

    // Listen for frame extracted
    unlistenPromises.push(
      listen('frame-extracted', (event: any) => {
        console.log('Frame extracted');
        updateProgress({ extractedFrame: event.payload });
      })
    );

    // Listen for encoding progress
    unlistenPromises.push(
      listen('encoding-progress', (event: any) => {
        const progress = event.payload;
        updateProgress({
          frame: progress.frame,
          fps: progress.fps,
          bitrate: progress.bitrate,
          speed: progress.speed,
          percentage: progress.percentage,
        });
      })
    );

    // Listen for job completed
    unlistenPromises.push(
      listen('job-completed', (event: any) => {
        console.log('Job completed:', event.payload);
        addToast({
          title: 'Success',
          description: 'Job completed successfully!',
          color: 'success'
        });
        setCurrentJob(null);
        updateProgress(null);
      })
    );

    // Listen for job errors
    unlistenPromises.push(
      listen('job-error', (event: any) => {
        console.error('Job error:', event.payload);
        addToast({
          title: 'Error',
          description: `Error: ${event.payload}`,
          color: 'danger',
          timeout: 6000
        });
      })
    );

    // Cleanup listeners on unmount
    return () => {
      Promise.all(unlistenPromises).then((unlisteners) => {
        unlisteners.forEach((unlisten) => unlisten());
      });
    };
  }, [setCurrentJob, updateProgress]);

  // Handle pause/resume
  const handlePauseResume = async () => {
    try {
      if (isPaused) {
        await invoke('resume_job_processing');
        setIsPaused(false);
        addToast({
          title: 'Success',
          description: 'Job processing resumed',
          color: 'success'
        });
      } else {
        await invoke('pause_job_processing');
        setIsPaused(true);
        addToast({
          title: 'Info',
          description: 'Job processing paused',
          color: 'primary'
        });
      }
    } catch (error) {
      addToast({
        title: 'Error',
        description: `Failed to ${isPaused ? 'resume' : 'pause'}: ${error}`,
        color: 'danger'
      });
    }
  };

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div className="neon-spinner mx-auto mb-4" />
          <p className="text-lg">Loading Dashboard...</p>
        </div>
      </div>
    );
  }

  return (
    <>
      {/* Dynamic Video Background */}
      <VideoBackground />

      {/* Main Content */}
      <div className="min-h-screen p-8 relative z-10">
        {/* Header */}
        <motion.header
          className="mb-8 flex items-center justify-between"
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
        >
          <div>
            <h1 className="text-2xl font-bold text-primary mb-2">
              FFNodes Client
            </h1>
            <p className="text-default-500 text-sm">
              {config?.display_name || 'Encoding Client'} • {gpuInfo?.vendor || 'Unknown GPU'}
            </p>
          </div>

          <div className="flex gap-3">
            <Button
              variant={isPaused ? 'success' : 'accent'}
              onClick={handlePauseResume}
            >
              {isPaused ? 'Resume' : 'Pause'}
            </Button>
            <Button variant="accent" onClick={() => navigate('/settings')}>
              Settings
            </Button>
          </div>
        </motion.header>

        {/* Main Layout */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Current Job - Takes 2 columns */}
          <motion.div
            className="lg:col-span-2"
            initial={{ opacity: 0, x: -50 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.2 }}
          >
            <CurrentJob />
          </motion.div>

          {/* Video List Sidebar - Takes 1 column */}
          <motion.div
            className="lg:col-span-1"
            initial={{ opacity: 0, x: 50 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.3 }}
          >
            <div className="sticky top-8">
              <VideoList />
            </div>
          </motion.div>
        </div>

        {/* Connection Status Footer */}
        <motion.footer
          className="mt-8 text-center"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.5 }}
        >
          <div className="bg-default-100/50 backdrop-blur-md inline-block px-6 py-3 rounded-full">
            <div className="flex items-center gap-3">
              <div className="w-2 h-2 rounded-full bg-success" />
              <span className="text-sm text-default-700">
                Connected to {config?.server_url || 'Server'}
              </span>
            </div>
          </div>
        </motion.footer>
      </div>
    </>
  );
}
