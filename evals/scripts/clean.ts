import { join } from 'path';
import { existsSync, readdirSync, readFileSync, rmSync, statSync } from 'fs';
import { logger } from '../src/utils/logger.ts';

interface RunResult {
  runId: string;
  timestamp: string;
  tmpDir: string;
  securityReviewOutput: string[];
}

function cleanEvals() {
  const evalsDir = join(process.cwd(), '.evals');

  if (!existsSync(evalsDir)) {
    logger.info('No .evals directory found - nothing to clean');
    return;
  }

  const entries = readdirSync(evalsDir);
  const runDirs = entries.filter(entry => {
    const entryPath = join(evalsDir, entry);
    return statSync(entryPath).isDirectory();
  });

  if (runDirs.length === 0) {
    logger.info('No evaluation run directories found in .evals directory');
    return;
  }

  logger.info(`Found ${runDirs.length} evaluation runs to clean up`);

  for (const runDir of runDirs) {
    const runDirPath = join(evalsDir, runDir);
    const runFilePath = join(runDirPath, 'run.json');

    // Try to read run.json to get tmpDir info
    if (existsSync(runFilePath)) {
      try {
        const content = readFileSync(runFilePath, 'utf-8');
        const runResult: RunResult = JSON.parse(content);

        // Clean up tmp directory if it exists
        if (runResult.tmpDir && existsSync(runResult.tmpDir)) {
          logger.info(`Removing tmp directory: ${runResult.tmpDir}`);
          rmSync(runResult.tmpDir, { recursive: true, force: true });
        } else {
          logger.info(`Tmp directory already gone or invalid: ${runResult.tmpDir}`);
        }
      } catch (error) {
        logger.warn(`Warning: Could not read run.json in ${runDir}, skipping tmp cleanup`);
      }
    }

    // Remove the entire run directory
    logger.info(`Removing evaluation run directory: ${runDir}`);
    rmSync(runDirPath, { recursive: true, force: true });
  }

  logger.info('✅ Cleanup completed');
}

if (import.meta.main) {
  cleanEvals();
}
