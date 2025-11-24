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

  const cardVariants = {
    hidden: { opacity: 0, y: 20 },
    visible: {
      opacity: 1,
      y: 0,
      transition: { duration: 0.5, ease: 'easeOut' }
    }
  };

  if (variant === 'gradient') {
    return (
      <motion.div
        className={`${baseClass} ${className}`}
        variants={animate ? cardVariants : undefined}
        initial={animate ? 'hidden' : undefined}
        animate={animate ? 'visible' : undefined}
        whileHover={{ scale: 1.02 }}
        transition={{ duration: 0.2 }}
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
      variants={animate ? cardVariants : undefined}
      initial={animate ? 'hidden' : undefined}
      animate={animate ? 'visible' : undefined}
      whileHover={{ scale: 1.02 }}
      transition={{ duration: 0.2 }}
    >
      {children}
    </motion.div>
  );
}
