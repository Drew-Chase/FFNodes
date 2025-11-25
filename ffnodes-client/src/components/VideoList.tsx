import { motion } from 'framer-motion';
import { useJobStore } from '../stores/useJobStore';
import { Card } from './ui';

export function VideoList() {
  const { jobQueue, currentJob } = useJobStore();

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'pending':
        return 'text-blue-400';
      case 'in_progress':
      case 'processing':
        return 'text-primary';
      case 'completed':
        return 'text-success';
      case 'failed':
        return 'text-danger';
      default:
        return 'text-default-500';
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
    <Card variant="glass" className="h-full flex flex-col">
      <div className="mb-4">
        <h2 className="text-lg font-semibold text-primary mb-1">Job Queue</h2>
        <p className="text-sm text-default-500 mt-1">
          {allJobs.length} {allJobs.length === 1 ? 'job' : 'jobs'} total
        </p>
      </div>

      <div className="flex-1 overflow-y-auto space-y-3 pr-2">
        {allJobs.length === 0 ? (
          <div className="text-center py-8 text-default-500">
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
                  bg-default-100/50 backdrop-blur-md p-4 rounded-lg cursor-pointer transition-all
                  ${isActive ? 'ring-2 ring-primary' : ''}
                `}
              >
                <div className="flex items-start gap-3">
                  {/* Status Icon */}
                  <div className={`text-2xl ${getStatusColor(job.status)}`}>
                    {getStatusIcon(job.status)}
                  </div>

                  {/* Job Info */}
                  <div className="flex-1 min-w-0">
                    <p
                      className={`font-medium truncate ${isActive ? 'text-primary' : 'text-foreground'}`}
                      title={filename}
                    >
                      {filename}
                    </p>
                    <p className="text-xs text-default-500 mt-1">Priority: {job.priority}</p>
                    {isActive && (
                      <div className="text-xs mt-2 flex items-center gap-1">
                        <div className="w-2 h-2 rounded-full bg-success" />
                        <span className="text-success">Encoding...</span>
                      </div>
                    )}
                  </div>
                </div>
              </motion.div>
            );
          })
        )}
      </div>
    </Card>
  );
}
