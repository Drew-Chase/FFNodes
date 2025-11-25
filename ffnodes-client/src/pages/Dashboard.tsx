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
import { BentoGrid, BentoCard, BentoCardHeader, BentoCardContent } from '../components/layout/BentoGrid';

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
      <div className="min-h-screen p-6 md:p-8 relative z-10">
        {/* Header */}
        <motion.header
          className="mb-6"
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3 }}
        >
          <BentoCard elevation={3} background="glass" className="!p-4">
            <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
              <div>
                <h1 className="text-headline-md font-medium text-primary mb-1">
                  FFNodes Client
                </h1>
                <p className="text-body-md text-foreground/70">
                  {config?.display_name || 'Encoding Client'} • {gpuInfo?.vendor || 'Unknown GPU'}
                </p>
              </div>

              <div className="flex gap-2 flex-wrap">
                <Button
                  color={isPaused ? 'success' : 'primary'}
                  onClick={handlePauseResume}
                  className="rounded-md-lg shadow-md-2"
                >
                  <iconify-icon icon={isPaused ? 'mdi:play' : 'mdi:pause'} class="text-lg mr-1" />
                  {isPaused ? 'Resume' : 'Pause'}
                </Button>
                <Button
                  color="secondary"
                  onClick={() => navigate('/settings')}
                  className="rounded-md-lg shadow-md-2"
                >
                  <iconify-icon icon="mdi:cog" class="text-lg mr-1" />
                  Settings
                </Button>
              </div>
            </div>
          </BentoCard>
        </motion.header>

        {/* Bento Grid Layout */}
        <BentoGrid columns={6} gap="md">
          {/* Current Job - Large card spanning 4 columns and 2 rows */}
          <BentoCard
            colSpan={4}
            rowSpan={2}
            elevation={4}
            background="glass"
            className="min-h-[400px]"
          >
            <CurrentJob />
          </BentoCard>

          {/* Video Queue - Tall card on the right */}
          <BentoCard
            colSpan={2}
            rowSpan={2}
            elevation={3}
            background="glass"
            className="min-h-[400px]"
          >
            <VideoList />
          </BentoCard>

          {/* Connection Status Card */}
          <BentoCard
            colSpan={3}
            elevation={2}
            background="gradient"
            hover
          >
            <BentoCardHeader
              title="Connection Status"
              icon={<iconify-icon icon="mdi:lan-connect" class="text-2xl" />}
            />
            <BentoCardContent>
              <div className="flex items-center gap-3 mt-2">
                <div className="w-3 h-3 rounded-full bg-success animate-pulse shadow-md-2 shadow-success" />
                <div>
                  <p className="text-body-lg font-medium text-foreground">
                    Connected
                  </p>
                  <p className="text-body-sm text-foreground/70">
                    {config?.server_url || 'Server'}
                  </p>
                </div>
              </div>
            </BentoCardContent>
          </BentoCard>

          {/* GPU Info Card */}
          <BentoCard
            colSpan={3}
            elevation={2}
            background="gradient"
            hover
          >
            <BentoCardHeader
              title="GPU Information"
              icon={<iconify-icon icon="mdi:memory" class="text-2xl" />}
            />
            <BentoCardContent>
              <div className="mt-2 space-y-1">
                <p className="text-body-lg font-medium text-foreground">
                  {gpuInfo?.name || 'Unknown GPU'}
                </p>
                <p className="text-body-sm text-foreground/70">
                  {gpuInfo?.vendor || 'Unknown Vendor'}
                </p>
                {gpuInfo?.memory && (
                  <p className="text-body-sm text-foreground/60">
                    {gpuInfo.memory} MB VRAM
                  </p>
                )}
              </div>
            </BentoCardContent>
          </BentoCard>

          {/* Quick Actions Card */}
          <BentoCard
            colSpan={2}
            elevation={2}
            background="solid"
            hover
            onClick={() => navigate('/setup')}
          >
            <BentoCardHeader
              title="Quick Setup"
              icon={<iconify-icon icon="mdi:settings-outline" class="text-2xl" />}
            />
            <BentoCardContent>
              <p className="text-body-sm text-foreground/70 mt-2">
                Update server connection and preferences
              </p>
            </BentoCardContent>
          </BentoCard>

          {/* Stats Card - Jobs Processed */}
          <BentoCard
            colSpan={2}
            elevation={2}
            background="gradient"
          >
            <BentoCardHeader
              title="Jobs Today"
              icon={<iconify-icon icon="mdi:chart-line" class="text-2xl" />}
            />
            <BentoCardContent>
              <div className="mt-2">
                <p className="text-display-sm font-normal text-primary">
                  {useJobStore.getState().jobQueue.length}
                </p>
                <p className="text-body-sm text-foreground/70">
                  Active in queue
                </p>
              </div>
            </BentoCardContent>
          </BentoCard>

          {/* Processing Status */}
          <BentoCard
            colSpan={2}
            elevation={2}
            background="gradient"
          >
            <BentoCardHeader
              title="Status"
              icon={<iconify-icon icon="mdi:state-machine" class="text-2xl" />}
            />
            <BentoCardContent>
              <div className="mt-2">
                <p className="text-title-lg font-medium text-foreground">
                  {isPaused ? 'Paused' : 'Processing'}
                </p>
                <p className="text-body-sm text-foreground/70">
                  {isPaused ? 'Resume to continue' : 'Actively encoding'}
                </p>
              </div>
            </BentoCardContent>
          </BentoCard>
        </BentoGrid>
      </div>
    </>
  );
}
