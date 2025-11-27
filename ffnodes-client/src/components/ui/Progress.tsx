import { Progress as HeroUIProgress } from '@heroui/react';

interface CustomProgressProps {
  value: number; // 0-100
  showPercentage?: boolean;
  height?: 'sm' | 'md' | 'lg';
  color?: 'default' | 'primary' | 'success' | 'warning' | 'danger' | 'secondary';
  className?: string;
}

export function Progress({
  value,
  showPercentage = true,
  height = 'md',
  color = 'primary',
  className = ''
}: CustomProgressProps) {
  return (
    <HeroUIProgress
      value={value}
      size={height}
      color={color}
      showValueLabel={showPercentage}
      valueLabel={`${value.toFixed(0)}%`}
      className={className}
    />
  );
}
