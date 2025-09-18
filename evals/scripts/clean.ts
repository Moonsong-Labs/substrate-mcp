import { join } from 'path';
import { existsSync, readdirSync, readFileSync, rmSync, statSync } from 'fs';
import { logger } from '../src/utils/logger.ts';

interface RunMetadata {
  id: string;
  task_directory: string;
  timestamp: string;
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
    const runMetadataPath = join(runDirPath, 'run_metadata.json');

    // Try to read run_metadata.json to get task_directory info
    if (existsSync(runMetadataPath)) {
      try {
        const content = readFileSync(runMetadataPath, 'utf-8');
        const runMetadata: RunMetadata = JSON.parse(content);

        // Clean up task directory if it exists
        if (runMetadata.task_directory && existsSync(runMetadata.task_directory)) {
          logger.info(`Removing task directory: ${runMetadata.task_directory}`);
          rmSync(runMetadata.task_directory, { recursive: true, force: true });
        } else {
          logger.info(`Task directory already gone or invalid: ${runMetadata.task_directory}`);
        }
      } catch (error) {
        logger.warn(`Warning: Could not read run_metadata.json in ${runDir}, skipping task cleanup`);
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
