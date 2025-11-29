import { Button as HeroUIButton, ButtonProps } from '@heroui/react';

interface CustomButtonProps extends Omit<ButtonProps, 'variant'> {
  variant?: 'primary' | 'accent' | 'success';
  loading?: boolean;
}

export function Button({
  variant = 'primary',
  loading = false,
  isDisabled,
  children,
  ...props
}: CustomButtonProps) {
  const getHeroUIProps = () => {
    switch(variant) {
      case 'primary': return { color: 'primary' as const, variant: 'solid' as const };
      case 'accent': return { color: 'secondary' as const, variant: 'bordered' as const };
      case 'success': return { color: 'success' as const, variant: 'solid' as const };
    }
  };

  return (
    <HeroUIButton
      {...getHeroUIProps()}
      isLoading={loading}
      isDisabled={isDisabled || loading}
      {...props}
    >
      {children}
    </HeroUIButton>
  );
}
