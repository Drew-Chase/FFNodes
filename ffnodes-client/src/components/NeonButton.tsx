import { motion } from 'framer-motion';
import { ReactNode } from 'react';

interface NeonButtonProps {
  children: ReactNode;
  onClick?: () => void;
  className?: string;
  variant?: 'primary' | 'accent' | 'success';
  disabled?: boolean;
  type?: 'button' | 'submit' | 'reset';
  loading?: boolean;
}

export function NeonButton({
  children,
  onClick,
  className = '',
  variant = 'primary',
  disabled = false,
  type = 'button',
  loading = false
}: NeonButtonProps) {
  const variantClasses = {
    primary: 'neon-button',
    accent: 'neon-button neon-border-accent',
    success: 'neon-button border-[var(--neon-green)] text-[var(--neon-green)]'
  };

  return (
    <motion.button
      type={type}
      className={`${variantClasses[variant]} rounded-lg ${className} ${disabled || loading ? 'opacity-50 cursor-not-allowed' : ''}`}
      onClick={disabled || loading ? undefined : onClick}
      disabled={disabled || loading}
      whileTap={!disabled && !loading ? { scale: 0.95 } : undefined}
      whileHover={!disabled && !loading ? { scale: 1.05 } : undefined}
    >
      {loading ? (
        <div className="flex items-center gap-2">
          <div className="neon-spinner w-5 h-5 border-2" />
          <span>Loading...</span>
        </div>
      ) : (
        children
      )}
    </motion.button>
  );
}
