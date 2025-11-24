import { motion } from 'framer-motion';
import { ReactNode } from 'react';

interface NeonCardProps {
  children: ReactNode;
  className?: string;
  variant?: 'glass' | 'gradient' | 'solid';
  animate?: boolean;
}

export function NeonCard({ children, className = '', variant = 'glass', animate = true }: NeonCardProps) {
  const baseClass = variant === 'gradient' ? 'gradient-border' : variant === 'solid' ? 'neon-border' : 'glass-card';

  if (variant === 'gradient') {
    return (
      <motion.div
        className={`${baseClass} ${className}`}
        initial={animate ? 'hidden' : undefined}
        animate={animate ? 'visible' : undefined}
      >
        <div className="gradient-border-content">
          {children}
        </div>
      </motion.div>
    );
  }

  return (
    <motion.div
      className={`${baseClass} p-6 ${className}`}
      initial={animate ? 'hidden' : undefined}
      animate={animate ? 'visible' : undefined}
      whileHover={{ scale: 1.02 }}
      transition={{ duration: 0.2 }}
    >
      {children}
    </motion.div>
  );
}
