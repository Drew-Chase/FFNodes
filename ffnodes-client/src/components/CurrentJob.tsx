import { motion } from 'framer-motion';
import { useJobStore } from '../stores/useJobStore';
import { Card, Progress } from './ui';

export function CurrentJob() {
  const { currentJob, currentProgress, isProcessing } = useJobStore();

  if (!currentJob || !currentProgress) {
    return (
      <Card className="text-center py-16">
        <div className="flex flex-col items-center gap-4">
          <div className="neon-spinner" />
          <p className="text-default-500">
            {isProcessing ? 'Waiting for job assignment...' : 'No active encoding job'}
          </p>
        </div>
      </Card>
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
      <Card variant="gradient" className="overflow-hidden">
        {/* File Name Header */}
        <div className="mb-6">
          <h2 className="text-xl font-bold mb-2">
            {filename}
          </h2>
          <p className="text-sm text-default-500">Job ID: {currentJob.id}</p>
        </div>

        {/* Progress Bar */}
        <div className="mb-8">
          <Progress value={percentage} height="lg" />
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {/* Frame Progress */}
          <div className="bg-default-50 rounded-lg p-4">
            <div className="text-sm text-default-500 mb-1">Frame</div>
            <div className="text-2xl font-bold">
              {frame.toLocaleString()}
              <span className="text-sm text-default-500 ml-1">/ {totalFrames.toLocaleString()}</span>
            </div>
          </div>

          {/* FPS */}
          <div className="bg-default-50 rounded-lg p-4">
            <div className="text-sm text-default-500 mb-1">FPS</div>
            <div className="text-2xl font-bold text-secondary">{fps.toFixed(2)}</div>
          </div>

          {/* Bitrate */}
          <div className="bg-default-50 rounded-lg p-4">
            <div className="text-sm text-default-500 mb-1">Bitrate</div>
            <div className="text-2xl font-bold">
              {(bitrate / 1000).toFixed(1)}
              <span className="text-sm text-default-500 ml-1">kbps</span>
            </div>
          </div>

          {/* Speed */}
          <div className="bg-default-50 rounded-lg p-4">
            <div className="text-sm text-default-500 mb-1">Speed</div>
            <div className={`text-2xl font-bold ${speed >= 1 ? 'text-success' : 'text-primary'}`}>
              {speed.toFixed(2)}x
            </div>
          </div>
        </div>

        {/* ETA */}
        {speed > 0 && totalFrames > frame && (
          <div className="mt-6 text-center">
            <span className="text-sm text-default-500">Estimated Time Remaining: </span>
            <span className="text-secondary font-bold">
              {calculateETA(totalFrames - frame, fps, speed)}
            </span>
          </div>
        )}

        {/* Processing Indicator */}
        <div className="absolute top-4 right-4">
          <div className="w-3 h-3 rounded-full bg-success" />
        </div>
      </Card>
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
