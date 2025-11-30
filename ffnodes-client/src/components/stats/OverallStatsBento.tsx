import { BentoCard, BentoCardHeader, BentoCardContent } from '../layout/BentoGrid';
import { useSystemStatsStore } from '../../stores/useSystemStatsStore';

// Utility functions for formatting
function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

function formatDuration(seconds: number): string {
  if (seconds === 0) return '0s';
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;

  const parts: string[] = [];
  if (hours > 0) parts.push(`${hours}h`);
  if (minutes > 0) parts.push(`${minutes}m`);
  if (secs > 0) parts.push(`${secs}s`);

  return parts.join(' ');
}

export function OverallStatsBento() {
  const { overallStats } = useSystemStatsStore();

  if (!overallStats) {
    return (
      <BentoCard colSpan={6} rowSpan={1} elevation={3} background="glass">
        <BentoCardHeader
          title="System Statistics"
          icon={<iconify-icon icon="mdi:chart-box" class="text-2xl" />}
        />
        <BentoCardContent>
          <div className="flex items-center justify-center h-32">
            <div className="text-foreground/60">Loading statistics...</div>
          </div>
        </BentoCardContent>
      </BentoCard>
    );
  }

  const {
    total_media_files,
    processed_files,
    pending_files,
    total_saved_bytes,
    total_processing_time_seconds,
    average_encoding_speed,
  } = overallStats;

  const processedPercentage = total_media_files > 0
    ? ((processed_files / total_media_files) * 100).toFixed(1)
    : '0.0';

  return (
    <BentoCard colSpan={6} rowSpan={1} elevation={3} background="glass">
      <BentoCardHeader
        title="System Statistics"
        icon={<iconify-icon icon="mdi:chart-box" class="text-2xl" />}
      />
      <BentoCardContent>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mt-2">
          {/* Files Processed */}
          <div className="flex flex-col">
            <div className="text-body-sm text-foreground/60 mb-1">Files Processed</div>
            <div className="text-title-lg font-medium text-foreground">
              {processed_files.toLocaleString()} / {total_media_files.toLocaleString()}
            </div>
            <div className="text-body-sm text-foreground/70 mt-1">
              {processedPercentage}% complete • {pending_files.toLocaleString()} pending
            </div>
          </div>

          {/* Storage Saved */}
          <div className="flex flex-col">
            <div className="text-body-sm text-foreground/60 mb-1">Storage Saved</div>
            <div className="text-title-lg font-medium text-success">
              {formatBytes(total_saved_bytes)}
            </div>
            <div className="text-body-sm text-foreground/70 mt-1">
              {total_saved_bytes > 0 ? 'Space reclaimed' : 'No savings yet'}
            </div>
          </div>

          {/* Processing Time */}
          <div className="flex flex-col">
            <div className="text-body-sm text-foreground/60 mb-1">Total Processing Time</div>
            <div className="text-title-lg font-medium text-foreground">
              {formatDuration(total_processing_time_seconds)}
            </div>
            <div className="text-body-sm text-foreground/70 mt-1">
              {total_processing_time_seconds > 0 ? 'Cumulative' : 'No jobs completed'}
            </div>
          </div>

          {/* Average Speed */}
          <div className="flex flex-col">
            <div className="text-body-sm text-foreground/60 mb-1">Avg. Encoding Speed</div>
            <div className="text-title-lg font-medium text-primary">
              {average_encoding_speed > 0 ? `${average_encoding_speed.toFixed(2)}x` : '0x'}
            </div>
            <div className="text-body-sm text-foreground/70 mt-1">
              {average_encoding_speed > 0 ? 'Across all clients' : 'Waiting for data'}
            </div>
          </div>
        </div>
      </BentoCardContent>
    </BentoCard>
  );
}
