import { motion } from 'framer-motion';
import { useJobStore } from '../../stores/useJobStore';
import { NeonCard } from './NeonCard';
import { NeonProgress } from './NeonProgress';
import { GlowText } from './GlowText';

export function CurrentJob() {
  const { currentJob, currentProgress, isProcessing } = useJobStore();

  if (!currentJob || !currentProgress) {
    return (
      <NeonCard className="text-center py-16">
        <div className="flex flex-col items-center gap-4">
          <div className="neon-spinner" />
          <p className="text-gray-400">
            {isProcessing ? 'Waiting for job assignment...' : 'No active encoding job'}
          </p>
        </div>
      </NeonCard>
    );
  }

  const filename = currentJob.media_file_path.split(/[/\\]/).pop() || 'Unknown';
  const { frame, totalFrames, fps, bitrate, speed, percentage } = currentProgress;

  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.95 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ duration: 0.5 }}
    >
      <NeonCard variant="gradient" className="overflow-hidden">
        {/* File Name Header */}
        <div className="mb-6">
          <GlowText size="xl" className="block mb-2">
            {filename}
          </GlowText>
          <p className="text-sm text-gray-400">Job ID: {currentJob.id}</p>
        </div>

        {/* Progress Bar */}
        <div className="mb-8">
          <NeonProgress value={percentage} height="lg" />
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {/* Frame Progress */}
          <div className="stat-card">
            <div className="stat-label">Frame</div>
            <div className="stat-value">
              {frame.toLocaleString()}
              <span className="text-sm text-gray-400 ml-1">/ {totalFrames.toLocaleString()}</span>
            </div>
          </div>

          {/* FPS */}
          <div className="stat-card">
            <div className="stat-label">FPS</div>
            <div className="stat-value neon-text-accent">{fps.toFixed(2)}</div>
          </div>

          {/* Bitrate */}
          <div className="stat-card">
            <div className="stat-label">Bitrate</div>
            <div className="stat-value">
              {(bitrate / 1000).toFixed(1)}
              <span className="text-sm text-gray-400 ml-1">kbps</span>
            </div>
          </div>

          {/* Speed */}
          <div className="stat-card">
            <div className="stat-label">Speed</div>
            <div className="stat-value" style={{ color: speed >= 1 ? 'var(--neon-green)' : 'var(--neon-primary)' }}>
              {speed.toFixed(2)}x
            </div>
          </div>
        </div>

        {/* ETA */}
        {speed > 0 && totalFrames > frame && (
          <motion.div
            className="mt-6 text-center"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.3 }}
          >
            <span className="text-sm text-gray-400">Estimated Time Remaining: </span>
            <span className="neon-text-accent font-bold">
              {calculateETA(totalFrames - frame, fps, speed)}
            </span>
          </motion.div>
        )}

        {/* Processing Indicator */}
        <motion.div
          className="absolute top-4 right-4"
          animate={{ scale: [1, 1.2, 1] }}
          transition={{ duration: 2, repeat: Infinity }}
        >
          <div className="w-3 h-3 rounded-full bg-[var(--neon-green)]" style={{ boxShadow: 'var(--glow-md) rgba(var(--neon-green-rgb), 0.8)' }} />
        </motion.div>
      </NeonCard>
    </motion.div>
  );
}

function calculateETA(remainingFrames: number, fps: number, speed: number): string {
  if (fps === 0 || speed === 0) return 'Calculating...';

  const secondsRemaining = remainingFrames / (fps * speed);
  const hours = Math.floor(secondsRemaining / 3600);
  const minutes = Math.floor((secondsRemaining % 3600) / 60);
  const seconds = Math.floor(secondsRemaining % 60);

  if (hours > 0) {
    return `${hours}h ${minutes}m ${seconds}s`;
  } else if (minutes > 0) {
    return `${minutes}m ${seconds}s`;
  } else {
    return `${seconds}s`;
  }
}
