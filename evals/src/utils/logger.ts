export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3
}

export class Logger {
  private level: LogLevel;

  constructor() {
    this.level = this.parseLogLevel(process.env.LOG_LEVEL || 'INFO');
  }

  private parseLogLevel(level: string): LogLevel {
    const upperLevel = level.toUpperCase();
    switch (upperLevel) {
      case 'DEBUG':
        return LogLevel.DEBUG;
      case 'INFO':
        return LogLevel.INFO;
      case 'WARN':
      case 'WARNING':
        return LogLevel.WARN;
      case 'ERROR':
        return LogLevel.ERROR;
      default:
        return LogLevel.INFO;
    }
  }

  private formatMessage(level: string, message: string): string {
    const timestamp = new Date().toISOString();
    return `[${timestamp}] ${level}: ${message}`;
  }

  private log(level: LogLevel, levelName: string, message: string, ...args: any[]): void {
    if (level >= this.level) {
      const formattedMessage = this.formatMessage(levelName, message);
      if (args.length > 0) {
        console.log(formattedMessage, ...args);
      } else {
        console.log(formattedMessage);
      }
    }
  }

  debug(message: string, ...args: any[]): void {
    this.log(LogLevel.DEBUG, 'DEBUG', message, ...args);
  }

  info(message: string, ...args: any[]): void {
    this.log(LogLevel.INFO, 'INFO', message, ...args);
  }

  warn(message: string, ...args: any[]): void {
    this.log(LogLevel.WARN, 'WARN', message, ...args);
  }

  error(message: string, ...args: any[]): void {
    this.log(LogLevel.ERROR, 'ERROR', message, ...args);
  }

  setLevel(level: LogLevel | string): void {
    if (typeof level === 'string') {
      this.level = this.parseLogLevel(level);
    } else {
      this.level = level;
    }
  }

  getLevel(): LogLevel {
    return this.level;
  }
}

export const logger = new Logger();