import { motion } from 'framer-motion';
import { ReactNode } from 'react';

interface GlowTextProps {
  children: ReactNode;
  className?: string;
  variant?: 'primary' | 'accent' | 'purple';
  animate?: boolean;
  size?: 'sm' | 'md' | 'lg' | 'xl' | '2xl';
}

export function GlowText({
  children,
  className = '',
  variant = 'primary',
  animate = false,
  size = 'md'
}: GlowTextProps) {
  const variantClasses = {
    primary: 'neon-text',
    accent: 'neon-text-accent',
    purple: 'text-[var(--neon-purple)] shadow-[var(--glow-sm)_rgba(var(--neon-purple-rgb),0.7)]'
  };

  const sizeClasses = {
    sm: 'text-sm',
    md: 'text-base',
    lg: 'text-lg',
    xl: 'text-xl',
    '2xl': 'text-2xl'
  };

  return (
    <motion.span
      className={`${variantClasses[variant]} ${sizeClasses[size]} font-bold ${animate ? 'neon-text-glow' : ''} ${className}`}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.5 }}
    >
      {children}
    </motion.span>
  );
}
