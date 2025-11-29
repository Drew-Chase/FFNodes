import { useEffect } from 'react';
import { BentoCard, BentoCardHeader, BentoCardContent } from '../layout/BentoGrid';
import { useDashboardStore } from '../../stores/useDashboardStore';

function getFilename(path: string): string {
  return path.split(/[\\/]/).pop() || path;
}

function getPhaseDisplay(phase: string | null): { text: string; color: string; icon: string } {
  switch (phase) {
    case 'downloading':
      return { text: 'Downloading', color: 'text-secondary', icon: 'mdi:download' };
    case 'encoding':
      return { text: 'Encoding', color: 'text-primary', icon: 'mdi:cog' };
    case 'uploading':
      return { text: 'Uploading', color: 'text-success', icon: 'mdi:upload' };
    default:
      return { text: 'Processing', color: 'text-foreground/70', icon: 'mdi:dots-horizontal' };
  }
}

export function CurrentJobsBento() {
  const { activeJobs, fetchActiveJobs } = useDashboardStore();

  useEffect(() => {
    // Initial fetch
    fetchActiveJobs();

    // Poll every 2 seconds for more responsive updates
    const interval = setInterval(fetchActiveJobs, 2000);

    return () => clearInterval(interval);
  }, [fetchActiveJobs]);

  return (
    <BentoCard colSpan={3} rowSpan={2} elevation={4} background="glass" className="min-h-[400px]">
      <BentoCardHeader
        title="Active Jobs"
        subtitle={`${activeJobs.length} jobs currently processing`}
        icon={<iconify-icon icon="mdi:video-box" class="text-2xl" />}
      />
      <BentoCardContent>
        {activeJobs.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-64 text-foreground/60">
            <iconify-icon icon="mdi:sleep" class="text-6xl mb-4 text-foreground/40" />
            <p className="text-body-lg">No active jobs</p>
            <p className="text-body-sm mt-2">Jobs will appear here when clients start encoding</p>
          </div>
        ) : (
          <div className="space-y-4 mt-4 max-h-[500px] overflow-y-auto">
            {activeJobs.map((job) => {
              const filename = getFilename(job.media_file_path);
              const isInProgress = job.status === 'in_progress';
              const isAssigned = job.status === 'assigned';
              const phaseDisplay = getPhaseDisplay(job.current_phase);

              return (
                <div
                  key={job.id}
                  className="p-4 rounded-md-lg bg-content2/50 border border-divider hover:bg-content2/70 transition-colors"
                >
                  {/* File name */}
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex-1 min-w-0">
                      <p className="text-body-md font-medium text-foreground truncate">
                        {filename}
                      </p>
                      <p className="text-body-sm text-foreground/60 mt-1">
                        Client: {job.assigned_client || 'Unassigned'}
                      </p>
                    </div>
                    <div className="flex-shrink-0 ml-4">
                      <div className={`px-2 py-1 rounded text-body-sm font-medium ${
                        isInProgress
                          ? 'bg-primary/20 text-primary'
                          : isAssigned
                          ? 'bg-secondary/20 text-secondary'
                          : 'bg-foreground/20 text-foreground'
                      }`}>
                        {job.status}
                      </div>
                    </div>
                  </div>

                  {/* Phase indicator */}
                  {(isInProgress || isAssigned) && job.current_phase && (
                    <div className="mt-2 flex items-center gap-2">
                      <iconify-icon icon={phaseDisplay.icon} class={`text-lg ${phaseDisplay.color}`} />
                      <span className={`text-body-sm font-medium ${phaseDisplay.color}`}>
                        {phaseDisplay.text}
                      </span>
                    </div>
                  )}

                  {/* Error message for failed jobs */}
                  {job.status === 'failed' && job.error_message && (
                    <div className="mt-2 p-2 rounded bg-danger/10 border border-danger/30">
                      <p className="text-body-sm text-danger">{job.error_message}</p>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </BentoCardContent>
    </BentoCard>
  );
}
