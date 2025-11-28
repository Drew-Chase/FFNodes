import { motion } from 'framer-motion';
import { useJobStore } from '../stores/useJobStore';
import { Card, Progress } from './ui';

// Utility functions
function getPhaseLabel(phase: string): string {
  const labels: Record<string, string> = {
    downloading: 'Downloading Input',
    encoding: 'Encoding Video',
    uploading: 'Uploading Output',
    completing: 'Completing Job',
  };
  return labels[phase] || 'Processing';
}

function getPhaseColor(phase: string): 'default' | 'primary' | 'success' {
  const colors: Record<string, 'default' | 'primary' | 'success'> = {
    downloading: 'default',
    encoding: 'primary',
    uploading: 'success',
    completing: 'success',
  };
  return colors[phase] || 'primary';
}

function formatBytes(bytes?: number): string {
  if (!bytes || bytes === 0) return '0 B';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function formatSpeed(bytesPerSecond?: number): string {
  if (!bytesPerSecond) return '0 B/s';
  return `${formatBytes(bytesPerSecond)}/s`;
}

function calculateTransferETA(transferred?: number, total?: number, speed?: number): string {
  if (!transferred || !total || !speed || speed === 0) return 'Calculating...';
  const remaining = total - transferred;
  const seconds = remaining / speed;

  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);

  if (hours > 0) return `${hours}h ${minutes}m ${secs}s`;
  if (minutes > 0) return `${minutes}m ${secs}s`;
  return `${secs}s`;
}

function calculateEncodingETA(remainingFrames: number, fps: number): string {
  if (fps === 0) return 'Calculating...';

  // FFmpeg's fps value is the actual encoding speed (frames per second being processed)
  // So we just divide remaining frames by fps to get seconds remaining
  const secondsRemaining = remainingFrames / fps;
  const hours = Math.floor(secondsRemaining / 3600);
  const minutes = Math.floor((secondsRemaining % 3600) / 60);
  const seconds = Math.floor(secondsRemaining % 60);

  if (hours > 0) return `${hours}h ${minutes}m ${seconds}s`;
  if (minutes > 0) return `${minutes}m ${seconds}s`;
  return `${seconds}s`;
}

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
  const { phase, frame, totalFrames, fps, bitrate, speed, percentage, transferredBytes, totalTransferBytes, transferSpeed } = currentProgress;

  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.95 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ duration: 0.5 }}
    >
      <Card variant="glass" className="overflow-hidden">
        {/* Phase Indicator Badge */}
        <div className="flex items-center gap-3 mb-6">
          <div className={`px-3 py-1 rounded-full text-sm font-medium ${
            phase === 'downloading' ? 'bg-default-100 text-default-700' :
            phase === 'encoding' ? 'bg-primary-100 text-primary-700' :
            phase === 'uploading' ? 'bg-success-100 text-success-700' :
            'bg-success-100 text-success-700'
          }`}>
            <iconify-icon icon={
              phase === 'downloading' ? 'mdi:download' :
              phase === 'encoding' ? 'mdi:video' :
              phase === 'uploading' ? 'mdi:upload' :
              'mdi:check-circle'
            } class="mr-1" />
            {getPhaseLabel(phase)}
          </div>
        </div>

        {/* File Name Header */}
        <div className="mb-6">
          <h2 className="text-xl font-bold mb-2">
            {filename}
          </h2>
          <p className="text-sm text-default-500">Job ID: {currentJob.id}</p>
        </div>

        {/* Progress Bar */}
        <div className="mb-8">
          <Progress value={percentage} height="lg" color={getPhaseColor(phase)} />
        </div>

        {/* Stats Grid - Dynamic based on phase */}
        {phase === 'downloading' && (
          <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
            {/* Downloaded / Total */}
            <div className="bg-default-50 rounded-lg p-4">
              <div className="text-sm text-default-500 mb-1">Downloaded</div>
              <div className="text-2xl font-bold">
                {formatBytes(transferredBytes)}
                <span className="text-sm text-default-500 ml-1">/ {formatBytes(totalTransferBytes)}</span>
              </div>
            </div>

            {/* Download Speed */}
            <div className="bg-default-50 rounded-lg p-4">
              <div className="text-sm text-default-500 mb-1">Speed</div>
              <div className="text-2xl font-bold text-primary">{formatSpeed(transferSpeed)}</div>
            </div>

            {/* ETA */}
            <div className="bg-default-50 rounded-lg p-4">
              <div className="text-sm text-default-500 mb-1">ETA</div>
              <div className="text-2xl font-bold">
                {calculateTransferETA(transferredBytes, totalTransferBytes, transferSpeed)}
              </div>
            </div>
          </div>
        )}

        {phase === 'encoding' && (
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
        )}

        {phase === 'uploading' && (
          <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
            {/* Uploaded / Total */}
            <div className="bg-default-50 rounded-lg p-4">
              <div className="text-sm text-default-500 mb-1">Uploaded</div>
              <div className="text-2xl font-bold">
                {formatBytes(transferredBytes)}
                <span className="text-sm text-default-500 ml-1">/ {formatBytes(totalTransferBytes)}</span>
              </div>
            </div>

            {/* Upload Speed */}
            <div className="bg-default-50 rounded-lg p-4">
              <div className="text-sm text-default-500 mb-1">Speed</div>
              <div className="text-2xl font-bold text-success">{formatSpeed(transferSpeed)}</div>
            </div>

            {/* ETA */}
            <div className="bg-default-50 rounded-lg p-4">
              <div className="text-sm text-default-500 mb-1">ETA</div>
              <div className="text-2xl font-bold">
                {calculateTransferETA(transferredBytes, totalTransferBytes, transferSpeed)}
              </div>
            </div>
          </div>
        )}

        {phase === 'completing' && (
          <div className="text-center py-8">
            <iconify-icon icon="mdi:check-circle" class="text-6xl text-success mb-4" />
            <p className="text-lg text-success font-bold">Job Completing...</p>
          </div>
        )}

        {/* ETA for encoding */}
        {phase === 'encoding' && speed > 0 && totalFrames > frame && (
          <div className="mt-6 text-center">
            <span className="text-sm text-default-500">Estimated Time Remaining: </span>
            <span className="text-secondary font-bold">
              {calculateEncodingETA(totalFrames - frame, fps)}
            </span>
          </div>
        )}

        {/* Processing Indicator */}
        <div className="absolute top-4 right-4">
          <div className={`w-3 h-3 rounded-full ${
            phase === 'completing' ? 'bg-success' : 'bg-success animate-pulse'
          }`} />
        </div>
      </Card>
    </motion.div>
  );
}
