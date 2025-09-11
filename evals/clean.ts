import { join } from 'path';
import { existsSync, readdirSync, readFileSync, rmSync } from 'fs';

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

function cleanEvals() {
  const evalsDir = join(process.cwd(), '.evals');
  
  if (!existsSync(evalsDir)) {
    console.log('No .evals directory found - nothing to clean');
    return;
  }

  const files = readdirSync(evalsDir);
  const jsonFiles = files.filter(file => file.endsWith('.json'));
  
  if (jsonFiles.length === 0) {
    console.log('No evaluation files found in .evals directory');
    return;
  }

  console.log(`Found ${jsonFiles.length} evaluation files to clean up`);

  for (const file of jsonFiles) {
    const filePath = join(evalsDir, file);
    
    const content = readFileSync(filePath, 'utf-8');
    const evalResult: EvaluationResult = JSON.parse(content);
    
    // Clean up tmp directory if it exists
    if (evalResult.tmpDir && existsSync(evalResult.tmpDir)) {
      console.log(`Removing tmp directory: ${evalResult.tmpDir}`);
      rmSync(evalResult.tmpDir, { recursive: true, force: true });
    } else {
      console.log(`Tmp directory already gone or invalid: ${evalResult.tmpDir}`);
    }
    
    // Remove the JSON file
    console.log(`Removing evaluation file: ${file}`);
    rmSync(filePath);
  }
  
  console.log('✅ Cleanup completed');
}

if (import.meta.main) {
  cleanEvals();
}