import { Progress as HeroUIProgress } from '@heroui/react';

interface CustomProgressProps {
  value: number; // 0-100
  showPercentage?: boolean;
  height?: 'sm' | 'md' | 'lg';
  className?: string;
}

export function Progress({
  value,
  showPercentage = true,
  height = 'md',
  className = ''
}: CustomProgressProps) {
  return (
    <HeroUIProgress
      value={value}
      size={height}
      color="primary"
      showValueLabel={showPercentage}
      valueLabel={`${value.toFixed(0)}%`}
      className={className}
    />
  );
}
