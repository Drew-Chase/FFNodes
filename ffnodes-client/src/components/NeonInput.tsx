import { motion } from 'framer-motion';
import { ChangeEvent, useState } from 'react';

interface NeonInputProps {
  label?: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  type?: 'text' | 'password' | 'email' | 'url';
  error?: string;
  required?: boolean;
  disabled?: boolean;
  className?: string;
}

export function NeonInput({
  label,
  value,
  onChange,
  placeholder = '',
  type = 'text',
  error,
  required = false,
  disabled = false,
  className = ''
}: NeonInputProps) {
  const [isFocused, setIsFocused] = useState(false);

  const handleChange = (e: ChangeEvent<HTMLInputElement>) => {
    onChange(e.target.value);
  };

  return (
    <div className={`flex flex-col gap-2 ${className}`}>
      {label && (
        <label className="text-sm font-medium text-gray-300 flex items-center gap-1">
          {label}
          {required && <span className="neon-text">*</span>}
        </label>
      )}
      <motion.div
        className="relative"
        initial={{ opacity: 0, y: -10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.3 }}
      >
        <input
          type={type}
          value={value}
          onChange={handleChange}
          onFocus={() => setIsFocused(true)}
          onBlur={() => setIsFocused(false)}
          placeholder={placeholder}
          disabled={disabled}
          className={`
            w-full px-4 py-3 rounded-lg
            bg-[var(--bg-dark-secondary)]
            border-2 transition-all duration-300
            text-white placeholder-gray-500
            focus:outline-none
            ${isFocused ? 'border-[var(--neon-primary)] shadow-[var(--glow-sm)_rgba(var(--neon-primary-rgb),0.5)]' : 'border-gray-700'}
            ${error ? 'border-[var(--neon-primary)] !shadow-[var(--glow-sm)_rgba(var(--neon-primary-rgb),0.7)]' : ''}
            ${disabled ? 'opacity-50 cursor-not-allowed' : ''}
          `}
        />
        {isFocused && (
          <motion.div
            className="absolute inset-0 rounded-lg pointer-events-none"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            style={{
              boxShadow: 'inset 0 0 10px rgba(255, 50, 71, 0.2)'
            }}
          />
        )}
      </motion.div>
      {error && (
        <motion.span
          className="text-sm neon-text"
          initial={{ opacity: 0, x: -10 }}
          animate={{ opacity: 1, x: 0 }}
        >
          {error}
        </motion.span>
      )}
    </div>
  );
}
