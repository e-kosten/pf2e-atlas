import { clearLine, cursorTo } from "node:readline";

export interface ProgressReporter {
  log(message: string): void;
  status(message: string): void;
  clear(): void;
}

export class ConsoleProgressReporter implements ProgressReporter {
  private readonly isInteractive: boolean;
  private statusActive = false;
  private readonly startTime = Date.now();

  constructor(private readonly stream: NodeJS.WriteStream = process.stderr) {
    this.isInteractive = Boolean(stream.isTTY);
  }

  log(message: string): void {
    this.clear();
    this.stream.write(`${message}\n`);
  }

  status(message: string): void {
    if (!this.isInteractive) {
      this.log(`${this.formatElapsedPrefix()} ${message}`);
      return;
    }

    clearLine(this.stream, 0);
    cursorTo(this.stream, 0);
    this.stream.write(this.truncateToTerminalWidth(`${this.formatElapsedPrefix()} ${message}`));
    this.statusActive = true;
  }

  clear(): void {
    if (!this.statusActive || !this.isInteractive) {
      return;
    }

    clearLine(this.stream, 0);
    cursorTo(this.stream, 0);
    this.statusActive = false;
  }

  private truncateToTerminalWidth(message: string): string {
    const terminalWidth = this.stream.columns ?? 0;
    if (terminalWidth <= 0 || message.length < terminalWidth) {
      return message;
    }

    if (terminalWidth <= 3) {
      return ".".repeat(Math.max(terminalWidth, 0));
    }

    return `${message.slice(0, terminalWidth - 3)}...`;
  }

  private formatElapsedPrefix(): string {
    const totalSeconds = Math.max(0, Math.floor((Date.now() - this.startTime) / 1000));
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;

    if (minutes === 0) {
      return `[${seconds}s]`;
    }

    return `[${minutes}m ${seconds}s]`;
  }
}
