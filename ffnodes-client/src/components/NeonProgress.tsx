import { motion } from 'framer-motion';

interface NeonProgressProps {
  value: number; // 0-100
  showPercentage?: boolean;
  className?: string;
  height?: 'sm' | 'md' | 'lg';
}

export function NeonProgress({
  value,
  showPercentage = true,
  className = '',
  height = 'md'
}: NeonProgressProps) {
  const clampedValue = Math.max(0, Math.min(100, value));

  const heightClasses = {
    sm: 'h-1',
    md: 'h-2',
    lg: 'h-3'
  };

  return (
    <div className={`w-full ${className}`}>
      {showPercentage && (
        <div className="flex justify-between items-center mb-2">
          <span className="text-sm text-gray-400">Progress</span>
          <span className="neon-text text-sm font-bold">{clampedValue.toFixed(1)}%</span>
        </div>
      )}
      <div className={`neon-progress-bar ${heightClasses[height]}`}>
        <motion.div
          className="neon-progress-fill"
          initial={{ width: 0 }}
          animate={{ width: `${clampedValue}%` }}
          transition={{ duration: 0.5, ease: 'easeOut' }}
        />
      </div>
    </div>
  );
}
