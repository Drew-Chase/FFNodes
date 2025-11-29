import React from 'react';
import { motion } from 'framer-motion';

interface BentoGridProps {
  children: React.ReactNode;
  className?: string;
  columns?: 'auto' | 2 | 3 | 4 | 6;
  gap?: 'sm' | 'md' | 'lg';
}

export const BentoGrid: React.FC<BentoGridProps> = ({
  children,
  className = '',
  columns = 'auto',
  gap = 'md',
}) => {
  const gapClasses = {
    sm: 'gap-2',
    md: 'gap-4',
    lg: 'gap-6',
  };

  const columnClasses = {
    auto: 'grid-cols-bento',
    2: 'grid-cols-1 md:grid-cols-2',
    3: 'grid-cols-1 md:grid-cols-2 lg:grid-cols-3',
    4: 'grid-cols-1 md:grid-cols-2 lg:grid-cols-4',
    6: 'grid-cols-2 md:grid-cols-3 lg:grid-cols-6',
  };

  return (
    <div
      className={`grid ${columnClasses[columns]} ${gapClasses[gap]} w-full ${className}`}
    >
      {children}
    </div>
  );
};

interface BentoCardProps {
  children: React.ReactNode;
  className?: string;
  colSpan?: 1 | 2 | 3 | 4 | 6;
  rowSpan?: 1 | 2 | 3 | 4;
  elevation?: 1 | 2 | 3 | 4 | 5 | 6;
  hover?: boolean;
  onClick?: () => void;
  background?: 'default' | 'gradient' | 'glass' | 'solid';
}

export const BentoCard: React.FC<BentoCardProps> = ({
  children,
  className = '',
  colSpan = 1,
  rowSpan = 1,
  elevation = 2,
  hover = false,
  onClick,
  background = 'default',
}) => {
  const colSpanClasses = {
    1: 'col-span-1',
    2: 'col-span-1 md:col-span-2',
    3: 'col-span-1 md:col-span-2 lg:col-span-3',
    4: 'col-span-1 md:col-span-2 lg:col-span-4',
    6: 'col-span-2 md:col-span-3 lg:col-span-6',
  };

  const rowSpanClasses = {
    1: 'row-span-1',
    2: 'row-span-2',
    3: 'row-span-3',
    4: 'row-span-4',
  };

  const elevationClasses = {
    1: 'shadow-md-1',
    2: 'shadow-md-2',
    3: 'shadow-md-3',
    4: 'shadow-md-4',
    5: 'shadow-md-5',
    6: 'shadow-md-6',
  };

  const backgroundClasses = {
    default: 'bg-content1',
    gradient: 'bg-gradient-to-br from-content1 to-content2',
    glass: 'bg-content1/60 backdrop-blur-md',
    solid: 'bg-content1',
  };

  const hoverClasses = hover
    ? 'hover:scale-[1.02] hover:shadow-md-4 transition-all duration-md-medium-2 cursor-pointer'
    : 'transition-shadow duration-md-medium-2';

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
      className={`
        ${colSpanClasses[colSpan]}
        ${rowSpanClasses[rowSpan]}
        ${elevationClasses[elevation]}
        ${backgroundClasses[background]}
        ${hoverClasses}
        rounded-md-lg
        p-6
        overflow-hidden
        ${className}
      `}
      onClick={onClick}
    >
      {children}
    </motion.div>
  );
};

interface BentoCardHeaderProps {
  title: string;
  subtitle?: string;
  icon?: React.ReactNode;
  action?: React.ReactNode;
  className?: string;
}

export const BentoCardHeader: React.FC<BentoCardHeaderProps> = ({
  title,
  subtitle,
  icon,
  action,
  className = '',
}) => {
  return (
    <div className={`flex items-start justify-between mb-4 ${className}`}>
      <div className="flex items-start gap-3 flex-1">
        {icon && (
          <div className="flex-shrink-0 text-primary mt-1">
            {icon}
          </div>
        )}
        <div className="flex-1 min-w-0">
          <h3 className="text-title-lg font-medium text-foreground truncate">
            {title}
          </h3>
          {subtitle && (
            <p className="text-body-sm text-foreground/60 mt-1 line-clamp-2">
              {subtitle}
            </p>
          )}
        </div>
      </div>
      {action && (
        <div className="flex-shrink-0 ml-4">
          {action}
        </div>
      )}
    </div>
  );
};

interface BentoCardContentProps {
  children: React.ReactNode;
  className?: string;
}

export const BentoCardContent: React.FC<BentoCardContentProps> = ({
  children,
  className = '',
}) => {
  return (
    <div className={`text-body-md text-foreground/80 ${className}`}>
      {children}
    </div>
  );
};

interface BentoCardFooterProps {
  children: React.ReactNode;
  className?: string;
}

export const BentoCardFooter: React.FC<BentoCardFooterProps> = ({
  children,
  className = '',
}) => {
  return (
    <div className={`mt-4 pt-4 border-t border-divider ${className}`}>
      {children}
    </div>
  );
};
