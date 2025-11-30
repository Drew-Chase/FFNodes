import { BentoCard, BentoCardHeader, BentoCardContent } from '../layout/BentoGrid';
import { useSystemStatsStore } from '../../stores/useSystemStatsStore';
import { motion, AnimatePresence } from 'framer-motion';

function getFilename(path: string): string {
  return path.split(/[\\/]/).pop() || path;
}

export function ScanProgressBento() {
  const { scanProgress, scanFileHistory, isScanActive } = useSystemStatsStore();

  // Don't render if no active scan
  if (!isScanActive || !scanProgress) {
    return null;
  }

  const percentage = scanProgress.total_files > 0
    ? (scanProgress.completed_files / scanProgress.total_files) * 100
    : 0;

  // Get last 3 files from history
  const recentFiles = scanFileHistory.slice(0, 3);

  return (
    <AnimatePresence>
      <BentoCard
        colSpan={6}
        elevation={4}
        background="gradient"
        className="border-2 border-primary/30 mb-4"
      >
        <BentoCardHeader
          title="Scanning Media Files"
          subtitle={scanProgress.operation}
          icon={<iconify-icon icon="mdi:folder-search" class="text-2xl animate-pulse" />}
        />

        <BentoCardContent>
          {/* Progress Stats */}
          <div className="flex items-center gap-6 mb-4">
            <div>
              <p className="text-body-sm text-foreground/60">Progress</p>
              <p className="text-title-lg font-medium text-primary">
                {scanProgress.completed_files} / {scanProgress.total_files}
              </p>
            </div>

            <div>
              <p className="text-body-sm text-foreground/60">Percentage</p>
              <p className="text-title-lg font-medium text-foreground">
                {percentage.toFixed(1)}%
              </p>
            </div>
          </div>

          {/* Progress Bar */}
          <div className="relative h-3 bg-content2 rounded-full overflow-hidden mb-4">
            <motion.div
              className="absolute inset-y-0 left-0 bg-gradient-to-r from-primary to-secondary rounded-full"
              initial={{ width: 0 }}
              animate={{ width: `${percentage}%` }}
              transition={{ duration: 0.3, ease: 'easeOut' }}
            />
          </div>

          {/* File Log */}
          {recentFiles.length > 0 && (
            <div className="mt-4">
              <p className="text-body-sm text-foreground/60 mb-2">Recent Files:</p>
              <div className="space-y-1.5 max-h-[72px] overflow-y-auto custom-scrollbar">
                <AnimatePresence mode="popLayout">
                  {recentFiles.map((log) => (
                    <motion.div
                      key={log.timestamp}
                      initial={{ opacity: 0, x: -20 }}
                      animate={{ opacity: 1, x: 0 }}
                      exit={{ opacity: 0, x: 20 }}
                      transition={{ duration: 0.2 }}
                      className="px-3 py-1.5 bg-content2/50 rounded text-body-sm text-foreground/80 font-mono truncate"
                      title={log.file}
                    >
                      {getFilename(log.file)}
                    </motion.div>
                  ))}
                </AnimatePresence>
              </div>
            </div>
          )}
        </BentoCardContent>
      </BentoCard>
    </AnimatePresence>
  );
}
