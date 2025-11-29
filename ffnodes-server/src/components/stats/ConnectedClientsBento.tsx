import { useEffect } from 'react';
import { BentoCard, BentoCardHeader, BentoCardContent } from '../layout/BentoGrid';
import { useDashboardStore } from '../../stores/useDashboardStore';

function formatRelativeTime(timestamp: number): string {
  const now = Date.now();
  const diff = now - timestamp * 1000; // Convert to milliseconds
  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days > 0) return `${days}d ago`;
  if (hours > 0) return `${hours}h ago`;
  if (minutes > 0) return `${minutes}m ago`;
  return `${seconds}s ago`;
}

export function ConnectedClientsBento() {
  const { connectedClients, fetchConnectedClients, activeJobs } = useDashboardStore();

  useEffect(() => {
    // Initial fetch
    fetchConnectedClients();

    // Poll every 10 seconds
    const interval = setInterval(fetchConnectedClients, 10000);

    return () => clearInterval(interval);
  }, [fetchConnectedClients]);

  // Calculate active jobs per client
  const jobsPerClient = activeJobs.reduce((acc, job) => {
    if (job.assigned_client) {
      acc[job.assigned_client] = (acc[job.assigned_client] || 0) + 1;
    }
    return acc;
  }, {} as Record<string, number>);

  return (
    <BentoCard colSpan={3} rowSpan={2} elevation={3} background="glass" className="min-h-[400px]">
      <BentoCardHeader
        title="Connected Clients"
        subtitle={`${connectedClients.length} clients online`}
        icon={<iconify-icon icon="mdi:desktop-tower" class="text-2xl" />}
      />
      <BentoCardContent>
        {connectedClients.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-64 text-foreground/60">
            <iconify-icon icon="mdi:lan-disconnect" class="text-6xl mb-4 text-foreground/40" />
            <p className="text-body-lg">No clients connected</p>
            <p className="text-body-sm mt-2">Waiting for clients to connect...</p>
          </div>
        ) : (
          <div className="space-y-3 mt-4 max-h-[500px] overflow-y-auto">
            {connectedClients.map((client) => {
              const activeJobCount = jobsPerClient[client.id] || 0;
              const isActive = activeJobCount > 0;
              const lastSeen = formatRelativeTime(client.last_heartbeat);

              return (
                <div
                  key={client.id}
                  className="p-4 rounded-md-lg bg-content2/50 border border-divider hover:bg-content2/70 transition-colors"
                >
                  <div className="flex items-start gap-3">
                    {/* Status indicator */}
                    <div className="flex-shrink-0 mt-1">
                      <div className={`w-3 h-3 rounded-full ${
                        isActive
                          ? 'bg-success animate-pulse shadow-md shadow-success'
                          : 'bg-primary'
                      }`} />
                    </div>

                    {/* Client info */}
                    <div className="flex-1 min-w-0">
                      <p className="text-body-md font-medium text-foreground truncate">
                        {client.display_name}
                      </p>
                      <p className="text-body-sm text-foreground/60 truncate mt-0.5">
                        {client.computer_name}
                      </p>

                      {/* Stats */}
                      <div className="flex items-center gap-4 mt-2 text-body-sm text-foreground/70">
                        <span className="flex items-center gap-1">
                          <iconify-icon icon="mdi:clock-outline" class="text-base" />
                          {lastSeen}
                        </span>
                        {activeJobCount > 0 && (
                          <span className="flex items-center gap-1 text-success">
                            <iconify-icon icon="mdi:play-circle" class="text-base" />
                            {activeJobCount} {activeJobCount === 1 ? 'job' : 'jobs'}
                          </span>
                        )}
                      </div>
                    </div>

                    {/* Badge */}
                    <div className="flex-shrink-0">
                      {isActive ? (
                        <div className="px-2 py-1 rounded bg-success/20 text-success text-body-sm font-medium">
                          Active
                        </div>
                      ) : (
                        <div className="px-2 py-1 rounded bg-foreground/10 text-foreground/70 text-body-sm font-medium">
                          Idle
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </BentoCardContent>
    </BentoCard>
  );
}
