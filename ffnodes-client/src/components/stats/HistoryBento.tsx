import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { motion } from 'framer-motion';
import { BentoCard, BentoCardHeader, BentoCardContent } from '../layout/BentoGrid';
import { useConfigStore } from '../../stores/useConfigStore';

interface JobHistoryEntry {
  filename: string;
  average_speed: number;
  duration_seconds: number;
  size_before: number;
  size_after: number;
  size_saved: number;
  size_reduction_percent: number;
  completed_at: number;
}

interface OverallStats {
  average_speed: number;
  average_duration_seconds: number;
  average_size_reduction_percent: number;
  total_jobs: number;
  total_size_saved: number;
}

interface ClientHistoryResponse {
  jobs: JobHistoryEntry[];
  overall: OverallStats;
}

export function HistoryBento() {
  const { config } = useConfigStore();
  const [history, setHistory] = useState<ClientHistoryResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchHistory = async () => {
      if (!config) return;

      try {
        setLoading(true);
        const data: ClientHistoryResponse = await invoke('get_client_history', { config });
        setHistory(data);
        setError(null);
      } catch (err) {
        setError(String(err));
        console.error('Failed to fetch history:', err);
      } finally {
        setLoading(false);
      }
    };

    // Initial fetch
    fetchHistory();

    // Listen for job completion and refresh immediately
    const unlistenPromise = listen('job-completed', () => {
      console.log('Job completed, refreshing history...');
      fetchHistory();
    });

    // Fallback: Refresh every 60 seconds (in case events are missed)
    const interval = setInterval(fetchHistory, 60000);

    // Cleanup
    return () => {
      clearInterval(interval);
      unlistenPromise.then(unlisten => unlisten());
    };
  }, [config]);

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';

    const isNegative = bytes < 0;
    const absoluteBytes = Math.abs(bytes);

    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(absoluteBytes) / Math.log(k));
    const formatted = `${(absoluteBytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;

    return isNegative ? `+${formatted}` : formatted;
  };

  const formatDuration = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;

    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    } else if (minutes > 0) {
      return `${minutes}m ${secs}s`;
    } else {
      return `${secs}s`;
    }
  };

  if (loading) {
    return (
      <BentoCard colSpan={2} rowSpan={2} elevation={3} background="glass">
        <BentoCardHeader
          title="Job History"
          icon={<iconify-icon icon="mdi:history" class="text-2xl" />}
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
          title="Job History"
          icon={<iconify-icon icon="mdi:history" class="text-2xl" />}
        />
        <BentoCardContent>
          <div className="text-center text-danger py-4">
            <p>Failed to load history</p>
            <p className="text-sm mt-2">{error}</p>
          </div>
        </BentoCardContent>
      </BentoCard>
    );
  }

  if (!history || history.overall.total_jobs === 0) {
    return (
      <BentoCard colSpan={2} rowSpan={2} elevation={3} background="glass">
        <BentoCardHeader
          title="Job History"
          icon={<iconify-icon icon="mdi:history" class="text-2xl" />}
        />
        <BentoCardContent>
          <div className="text-center text-foreground/60 py-8">
            <iconify-icon icon="mdi:clipboard-text-off-outline" class="text-6xl mb-4 opacity-50" />
            <p>No completed jobs yet</p>
          </div>
        </BentoCardContent>
      </BentoCard>
    );
  }

  return (
    <BentoCard colSpan={2} rowSpan={2} elevation={3} background="glass" className="overflow-hidden">
      <BentoCardHeader
        title="Job History"
        subtitle={`${history.overall.total_jobs} completed jobs`}
        icon={<iconify-icon icon="mdi:history" class="text-2xl" />}
      />

      {/* Overall Stats */}
      <div className="grid grid-cols-2 gap-3 mb-4">
        <motion.div
          className="bg-content2 rounded-md p-3"
          whileHover={{ scale: 1.02 }}
          transition={{ duration: 0.2 }}
        >
          <div className="flex items-center gap-2 mb-1">
            <iconify-icon icon="mdi:speedometer" class="text-primary text-lg" />
            <span className="text-xs text-foreground/70">Avg Speed</span>
          </div>
          <p className="text-2xl font-bold text-primary">
            {history.overall.average_speed.toFixed(2)}x
          </p>
        </motion.div>

        <motion.div
          className="bg-content2 rounded-md p-3"
          whileHover={{ scale: 1.02 }}
          transition={{ duration: 0.2 }}
        >
          <div className="flex items-center gap-2 mb-1">
            <iconify-icon icon="mdi:content-save-outline" class={`text-lg ${history.overall.total_size_saved < 0 ? 'text-danger' : 'text-success'}`} />
            <span className="text-xs text-foreground/70">Total Saved</span>
          </div>
          <p className={`text-2xl font-bold ${history.overall.total_size_saved < 0 ? 'text-danger' : 'text-success'}`}>
            {formatBytes(history.overall.total_size_saved)}
          </p>
        </motion.div>

        <motion.div
          className="bg-content2 rounded-md p-3"
          whileHover={{ scale: 1.02 }}
          transition={{ duration: 0.2 }}
        >
          <div className="flex items-center gap-2 mb-1">
            <iconify-icon icon="mdi:clock-outline" class="text-secondary text-lg" />
            <span className="text-xs text-foreground/70">Avg Duration</span>
          </div>
          <p className="text-2xl font-bold text-secondary">
            {formatDuration(Math.floor(history.overall.average_duration_seconds))}
          </p>
        </motion.div>

        <motion.div
          className="bg-content2 rounded-md p-3"
          whileHover={{ scale: 1.02 }}
          transition={{ duration: 0.2 }}
        >
          <div className="flex items-center gap-2 mb-1">
            <iconify-icon icon="mdi:percent-outline" class="text-warning text-lg" />
            <span className="text-xs text-foreground/70">Avg Reduction</span>
          </div>
          <p className="text-2xl font-bold text-warning">
            {history.overall.average_size_reduction_percent.toFixed(1)}%
          </p>
        </motion.div>
      </div>

      {/* Recent Jobs List */}
      <div className="mt-4">
        <h4 className="text-sm font-medium text-foreground/80 mb-2">Recent Jobs</h4>
        <div className="space-y-2 max-h-[250px] overflow-y-auto pr-2">
          {history.jobs.slice(0, 10).map((job, idx) => (
            <motion.div
              key={idx}
              className="bg-content2/50 rounded-md p-3 hover:bg-content2 transition-colors"
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.3, delay: idx * 0.05 }}
            >
              <div className="flex items-start justify-between mb-1">
                <p className="text-sm font-medium text-foreground truncate flex-1">
                  {job.filename}
                </p>
                <span className={`text-xs ml-2 ${job.size_reduction_percent < 0 ? 'text-danger' : 'text-success'}`}>
                  {job.size_reduction_percent.toFixed(1)}%
                </span>
              </div>
              <div className="flex items-center gap-4 text-xs text-foreground/60">
                <span className="flex items-center gap-1">
                  <iconify-icon icon="mdi:speedometer" />
                  {job.average_speed.toFixed(2)}x
                </span>
                <span className="flex items-center gap-1">
                  <iconify-icon icon="mdi:clock-outline" />
                  {formatDuration(job.duration_seconds)}
                </span>
                <span className={`flex items-center gap-1 ${job.size_saved < 0 ? 'text-danger' : ''}`}>
                  <iconify-icon icon={job.size_saved < 0 ? "mdi:arrow-up" : "mdi:arrow-down"} />
                  {formatBytes(job.size_saved)}
                </span>
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </BentoCard>
  );
}
