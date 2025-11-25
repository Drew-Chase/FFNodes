import { Input as HeroUIInput, InputProps } from '@heroui/react';

interface CustomInputProps extends Omit<InputProps, 'onChange'> {
  value: string;
  onChange: (value: string) => void;
  error?: string;
  required?: boolean;
}

export function Input({
  label,
  value,
  onChange,
  error,
  required = false,
  ...props
}: CustomInputProps) {
  return (
    <HeroUIInput
      label={label}
      value={value}
      onValueChange={onChange}
      isInvalid={!!error}
      errorMessage={error}
      isRequired={required}
      variant="flat"
      {...props}
    />
  );
}
