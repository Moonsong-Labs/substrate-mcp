import { readdir, readFile, rm } from 'fs/promises';
import { join } from 'path';
import { existsSync } from 'fs';
import { tryCatch } from './utils.js';

interface EvaluationResult {
  runId: string;
  timestamp: string;
  tmpDir: string;
  securityReviewOutput: string;
  evaluationOutput: string;
  metadata: {
    hasSecurityDisclaimer: boolean;
    caughtEscrowExpiration: boolean;
    evaluationScore: number;
  };
}

async function cleanEvals() {
  const evalsDir = join(process.cwd(), '.evals');
  
  if (!existsSync(evalsDir)) {
    console.log('No .evals directory found - nothing to clean');
    return;
  }

  const filesResult = await tryCatch(readdir(evalsDir));
  if (filesResult.error) {
    console.error('Error reading .evals directory:', filesResult.error);
    return;
  }

  const jsonFiles = filesResult.data.filter(file => file.endsWith('.json'));
  
  if (jsonFiles.length === 0) {
    console.log('No evaluation files found in .evals directory');
    return;
  }

  console.log(`Found ${jsonFiles.length} evaluation files to clean up`);

  for (const file of jsonFiles) {
    const filePath = join(evalsDir, file);
    
    const contentResult = await tryCatch(readFile(filePath, 'utf-8'));
    if (contentResult.error) {
      console.error(`Error reading ${file}:`, contentResult.error);
      continue;
    }

    const parseResult = await tryCatch(Promise.resolve(JSON.parse(contentResult.data)));
    if (parseResult.error) {
      console.error(`Error parsing ${file}:`, parseResult.error);
      continue;
    }

    const evalResult: EvaluationResult = parseResult.data;
    
    // Clean up tmp directory if it exists
    if (evalResult.tmpDir && existsSync(evalResult.tmpDir)) {
      console.log(`Removing tmp directory: ${evalResult.tmpDir}`);
      const rmDirResult = await tryCatch(rm(evalResult.tmpDir, { recursive: true, force: true }));
      if (rmDirResult.error) {
        console.error(`Error removing tmp directory ${evalResult.tmpDir}:`, rmDirResult.error);
      }
    } else {
      console.log(`Tmp directory already gone or invalid: ${evalResult.tmpDir}`);
    }
    
    // Remove the JSON file
    console.log(`Removing evaluation file: ${file}`);
    const rmFileResult = await tryCatch(rm(filePath));
    if (rmFileResult.error) {
      console.error(`Error removing file ${file}:`, rmFileResult.error);
    }
  }
  
  console.log('✅ Cleanup completed');
}

if (import.meta.main) {
  cleanEvals();
}