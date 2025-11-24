import { motion } from 'framer-motion';
import { useJobStore } from '../../stores/useJobStore';
import { NeonCard } from './NeonCard';
import { GlowText } from './GlowText';

export function VideoList() {
  const { jobQueue, currentJob } = useJobStore();

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'pending':
        return 'text-blue-400';
      case 'in_progress':
      case 'processing':
        return 'neon-text';
      case 'completed':
        return 'text-[var(--neon-green)]';
      case 'failed':
        return 'text-red-400';
      default:
        return 'text-gray-400';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status.toLowerCase()) {
      case 'pending':
        return '⏳';
      case 'in_progress':
      case 'processing':
        return '⚡';
      case 'completed':
        return '✓';
      case 'failed':
        return '✗';
      default:
        return '○';
    }
  };

  const allJobs = currentJob ? [currentJob, ...jobQueue] : jobQueue;

  return (
    <NeonCard className="h-full flex flex-col">
      <div className="mb-4">
        <GlowText size="lg">Job Queue</GlowText>
        <p className="text-sm text-gray-400 mt-1">
          {allJobs.length} {allJobs.length === 1 ? 'job' : 'jobs'} total
        </p>
      </div>

      <div className="flex-1 overflow-y-auto space-y-3 pr-2">
        {allJobs.length === 0 ? (
          <div className="text-center py-8 text-gray-500">
            <p>No jobs in queue</p>
          </div>
        ) : (
          allJobs.map((job, index) => {
            const filename = job.media_file_path.split(/[/\\]/).pop() || 'Unknown';
            const isActive = currentJob?.id === job.id;

            return (
              <motion.div
                key={job.id}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: index * 0.05 }}
                className={`
                  glass p-4 rounded-lg cursor-pointer transition-all
                  ${isActive ? 'ring-2 ring-[var(--neon-primary)] shadow-[var(--glow-md)_rgba(var(--neon-primary-rgb),0.5)]' : ''}
                `}
              >
                <div className="flex items-start gap-3">
                  {/* Status Icon */}
                  <motion.div
                    className={`text-2xl ${getStatusColor(job.status)}`}
                    animate={isActive ? { scale: [1, 1.2, 1] } : {}}
                    transition={{ duration: 2, repeat: Infinity }}
                  >
                    {getStatusIcon(job.status)}
                  </motion.div>

                  {/* Job Info */}
                  <div className="flex-1 min-w-0">
                    <p
                      className={`font-medium truncate ${isActive ? 'neon-text' : 'text-white'}`}
                      title={filename}
                    >
                      {filename}
                    </p>
                    <p className="text-xs text-gray-500 mt-1">Priority: {job.priority}</p>
                    {isActive && (
                      <motion.div
                        className="text-xs mt-2 flex items-center gap-1"
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                      >
                        <div className="w-2 h-2 rounded-full bg-[var(--neon-green)] animate-pulse" />
                        <span className="text-[var(--neon-green)]">Encoding...</span>
                      </motion.div>
                    )}
                  </div>
                </div>
              </motion.div>
            );
          })
        )}
      </div>
    </NeonCard>
  );
}
