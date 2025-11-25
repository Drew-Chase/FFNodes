import { Card as HeroUICard, CardBody, CardHeader, CardFooter } from '@heroui/react';
import { ReactNode } from 'react';

interface CustomCardProps {
  variant?: 'glass' | 'gradient' | 'solid';
  children: ReactNode;
  className?: string;
}

export function Card({
  variant = 'solid',
  children,
  className = ''
}: CustomCardProps) {
  const getClassName = () => {
    switch(variant) {
      case 'glass': return 'bg-default-100/50 backdrop-blur-md';
      case 'gradient': return 'bg-gradient-to-br from-primary-500/20 to-secondary-500/20';
      case 'solid': return 'bg-default-100';
    }
  };

  return (
    <HeroUICard shadow="md" className={`${getClassName()} ${className}`}>
      <CardBody>{children}</CardBody>
    </HeroUICard>
  );
}

// Export sub-components
export { CardBody, CardHeader, CardFooter };
