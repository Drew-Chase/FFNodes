import { Card as HeroUICard, CardBody, CardHeader, CardFooter } from '@heroui/react';
import { ReactNode } from 'react';

interface CustomCardProps {
  variant?: 'glass' | 'solid';
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
