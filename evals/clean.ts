import { readdir, readFile, rm } from 'fs/promises';
import { join } from 'path';
import { existsSync } from 'fs';
import { Result, ok, err, fromThrowable } from 'neverthrow';

// Create fromThrowable wrappers for operations that might throw
const safeReaddir = fromThrowable(readdir, (error) => error as Error);
const safeReadFile = fromThrowable(readFile, (error) => error as Error);
const safeJsonParse = fromThrowable(JSON.parse, (error) => error as Error);
const safeRm = fromThrowable(rm, (error) => error as Error);

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

  const filesResult = await safeReaddir(evalsDir);
  if (filesResult.isErr()) {
    console.error('Error reading .evals directory:', filesResult.error);
    return;
  }

  const jsonFiles = filesResult.value.filter(file => file.endsWith('.json'));
  
  if (jsonFiles.length === 0) {
    console.log('No evaluation files found in .evals directory');
    return;
  }

  console.log(`Found ${jsonFiles.length} evaluation files to clean up`);

  for (const file of jsonFiles) {
    const filePath = join(evalsDir, file);
    
    const contentResult = await safeReadFile(filePath, 'utf-8');
    if (contentResult.isErr()) {
      console.error(`Error reading ${file}:`, contentResult.error);
      continue;
    }

    const parseResult = safeJsonParse(contentResult.value);
    if (parseResult.isErr()) {
      console.error(`Error parsing ${file}:`, parseResult.error);
      continue;
    }

    const evalResult: EvaluationResult = parseResult.value;
    
    // Clean up tmp directory if it exists
    if (evalResult.tmpDir && existsSync(evalResult.tmpDir)) {
      console.log(`Removing tmp directory: ${evalResult.tmpDir}`);
      const rmDirResult = await safeRm(evalResult.tmpDir, { recursive: true, force: true });
      if (rmDirResult.isErr()) {
        console.error(`Error removing tmp directory ${evalResult.tmpDir}:`, rmDirResult.error);
      }
    } else {
      console.log(`Tmp directory already gone or invalid: ${evalResult.tmpDir}`);
    }
    
    // Remove the JSON file
    console.log(`Removing evaluation file: ${file}`);
    const rmFileResult = await safeRm(filePath);
    if (rmFileResult.isErr()) {
      console.error(`Error removing file ${file}:`, rmFileResult.error);
    }
  }
  
  console.log('✅ Cleanup completed');
}

if (import.meta.main) {
  cleanEvals();
}