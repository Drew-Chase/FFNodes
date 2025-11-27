import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { motion } from 'framer-motion';
import { BentoCard, BentoCardHeader, BentoCardContent } from '../layout/BentoGrid';
import { Progress } from '../ui';
import { useConfigStore } from '../../stores/useConfigStore';

interface RemoteJobProgress {
  client_name: string;
  filename: string;
  percentage: number;
  speed: number;
  frame: number;
  total_frames: number;
}

interface RemoteProgressResponse {
  active_jobs: RemoteJobProgress[];
}

export function RemoteUsersBento() {
  const { config } = useConfigStore();
  const [progress, setProgress] = useState<RemoteProgressResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchProgress = async () => {
      if (!config) return;

      try {
        setLoading(true);
        const data: RemoteProgressResponse = await invoke('get_remote_progress', { config });
        setProgress(data);
        setError(null);
      } catch (err) {
        setError(String(err));
        console.error('Failed to fetch remote progress:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchProgress();
    // Refresh every 5 seconds for real-time updates
    const interval = setInterval(fetchProgress, 5000);
    return () => clearInterval(interval);
  }, [config]);

  if (loading) {
    return (
      <BentoCard colSpan={2} rowSpan={2} elevation={3} background="glass">
        <BentoCardHeader
          title="Remote Users"
          icon={<iconify-icon icon="mdi:account-group" class="text-2xl" />}
        />
        <BentoCardContent>
          <div className="flex items-center justify-center py-8">
            <div className="neon-spinner" />
          </div>
        </BentoCardContent>
      </BentoCard>
    );
  }

  if (error) {
    return (
      <BentoCard colSpan={2} rowSpan={2} elevation={3} background="glass">
        <BentoCardHeader
          title="Remote Users"
          icon={<iconify-icon icon="mdi:account-group" class="text-2xl" />}
        />
        <BentoCardContent>
          <div className="text-center text-danger py-4">
            <p>Failed to load remote progress</p>
            <p className="text-sm mt-2">{error}</p>
          </div>
        </BentoCardContent>
      </BentoCard>
    );
  }

  if (!progress || progress.active_jobs.length === 0) {
    return (
      <BentoCard colSpan={2} rowSpan={2} elevation={3} background="glass">
        <BentoCardHeader
          title="Remote Users"
          subtitle="Active encoders"
          icon={<iconify-icon icon="mdi:account-group" class="text-2xl" />}
        />
        <BentoCardContent>
          <div className="text-center text-foreground/60 py-8">
            <iconify-icon icon="mdi:account-off-outline" class="text-6xl mb-4 opacity-50" />
            <p>No remote users encoding</p>
          </div>
        </BentoCardContent>
      </BentoCard>
    );
  }

  return (
    <BentoCard colSpan={2} rowSpan={2} elevation={3} background="glass" className="overflow-hidden">
      <BentoCardHeader
        title="Remote Users"
        subtitle={`${progress.active_jobs.length} active encoders`}
        icon={<iconify-icon icon="mdi:account-group" class="text-2xl" />}
      />

      <BentoCardContent>
        <div className="space-y-4 max-h-[500px] overflow-y-auto pr-2">
          {progress.active_jobs.map((job, idx) => (
            <motion.div
              key={idx}
              className="bg-content2 rounded-md p-4"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3, delay: idx * 0.05 }}
            >
              {/* Client Name and Speed */}
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center gap-2">
                  <iconify-icon icon="mdi:account-circle" class="text-primary text-xl" />
                  <span className="font-medium text-foreground">{job.client_name}</span>
                </div>
                <div className="flex items-center gap-1 text-sm">
                  <iconify-icon icon="mdi:speedometer" class="text-success" />
                  <span className={`font-bold ${job.speed >= 1 ? 'text-success' : 'text-primary'}`}>
                    {job.speed.toFixed(2)}x
                  </span>
                </div>
              </div>

              {/* Filename */}
              <p className="text-sm text-foreground/80 mb-2 truncate" title={job.filename}>
                {job.filename}
              </p>

              {/* Progress Bar */}
              <div className="mb-2">
                <Progress value={job.percentage} height="sm" />
              </div>

              {/* Progress Details */}
              <div className="flex items-center justify-between text-xs text-foreground/60">
                <span>{job.percentage.toFixed(1)}%</span>
                <span>
                  {job.frame.toLocaleString()} / {job.total_frames.toLocaleString()} frames
                </span>
              </div>
            </motion.div>
          ))}
        </div>
      </BentoCardContent>
    </BentoCard>
  );
}
