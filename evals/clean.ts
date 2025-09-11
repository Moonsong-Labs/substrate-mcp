import { join } from 'path';
import { existsSync, readdirSync, readFileSync, rmSync, statSync } from 'fs';

interface RunResult {
  runId: string;
  timestamp: string;
  tmpDir: string;
  securityReviewOutput: string[];
}

function cleanEvals() {
  const evalsDir = join(process.cwd(), '.evals');
  
  if (!existsSync(evalsDir)) {
    console.log('No .evals directory found - nothing to clean');
    return;
  }

  const entries = readdirSync(evalsDir);
  const runDirs = entries.filter(entry => {
    const entryPath = join(evalsDir, entry);
    return statSync(entryPath).isDirectory();
  });
  
  if (runDirs.length === 0) {
    console.log('No evaluation run directories found in .evals directory');
    return;
  }

  console.log(`Found ${runDirs.length} evaluation runs to clean up`);

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
          console.log(`Removing tmp directory: ${runResult.tmpDir}`);
          rmSync(runResult.tmpDir, { recursive: true, force: true });
        } else {
          console.log(`Tmp directory already gone or invalid: ${runResult.tmpDir}`);
        }
      } catch (error) {
        console.log(`Warning: Could not read run.json in ${runDir}, skipping tmp cleanup`);
      }
    }
    
    // Remove the entire run directory
    console.log(`Removing evaluation run directory: ${runDir}`);
    rmSync(runDirPath, { recursive: true, force: true });
  }
  
  console.log('✅ Cleanup completed');
}

if (import.meta.main) {
  cleanEvals();
}