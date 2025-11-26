import { invoke } from '@tauri-apps/api/core';

/**
 * Frontend logger that sends logs to the backend for file logging
 */
export class Logger {
  private source: string;

  constructor(source: string = 'frontend') {
    this.source = source;
  }

  /**
   * Log an error message
   */
  error(message: string, ...args: any[]) {
    const fullMessage = this.formatMessage(message, args);
    console.error(`[${this.source}]`, fullMessage);
    this.sendToBackend('error', fullMessage);
  }

  /**
   * Log a warning message
   */
  warn(message: string, ...args: any[]) {
    const fullMessage = this.formatMessage(message, args);
    console.warn(`[${this.source}]`, fullMessage);
    this.sendToBackend('warn', fullMessage);
  }

  /**
   * Log an info message
   */
  info(message: string, ...args: any[]) {
    const fullMessage = this.formatMessage(message, args);
    console.info(`[${this.source}]`, fullMessage);
    this.sendToBackend('info', fullMessage);
  }

  /**
   * Log a debug message
   */
  debug(message: string, ...args: any[]) {
    const fullMessage = this.formatMessage(message, args);
    console.debug(`[${this.source}]`, fullMessage);
    this.sendToBackend('debug', fullMessage);
  }

  /**
   * Log a trace message
   */
  trace(message: string, ...args: any[]) {
    const fullMessage = this.formatMessage(message, args);
    console.trace(`[${this.source}]`, fullMessage);
    this.sendToBackend('trace', fullMessage);
  }

  /**
   * Format message with additional arguments
   */
  private formatMessage(message: string, args: any[]): string {
    if (args.length === 0) return message;

    // Convert objects to JSON strings
    const formattedArgs = args.map(arg => {
      if (typeof arg === 'object') {
        try {
          return JSON.stringify(arg, null, 2);
        } catch {
          return String(arg);
        }
      }
      return String(arg);
    });

    return `${message} ${formattedArgs.join(' ')}`;
  }

  /**
   * Send log to backend
   */
  private sendToBackend(level: string, message: string) {
    // Fire and forget - don't await to avoid blocking
    invoke('log_frontend', {
      level,
      message,
      source: this.source
    }).catch(err => {
      // If backend logging fails, just log to console
      console.error('[Logger] Failed to send log to backend:', err);
    });
  }
}

// Create default logger instance
export const logger = new Logger('frontend');

// Export convenience functions
export const log = {
  error: (msg: string, ...args: any[]) => logger.error(msg, ...args),
  warn: (msg: string, ...args: any[]) => logger.warn(msg, ...args),
  info: (msg: string, ...args: any[]) => logger.info(msg, ...args),
  debug: (msg: string, ...args: any[]) => logger.debug(msg, ...args),
  trace: (msg: string, ...args: any[]) => logger.trace(msg, ...args),
};
