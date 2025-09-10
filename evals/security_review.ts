import { mkdtemp, cp, mkdir, writeFile } from 'fs/promises';
import { join } from 'path';
import { tmpdir } from 'os';
import { randomBytes } from 'crypto';
import { query } from '@anthropic-ai/claude-code';

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

function generateRunId(): string {
  return `eval-${Date.now()}-${randomBytes(4).toString('hex')}`;
}

async function runSecurityReview(cwd: string): Promise<string> {
  let result = '';

  for await (const message of query({
    prompt: 'Use the substrate MCP server security_review prompt to analyze this escrow pallet implementation for security vulnerabilities, economic risks, and code quality issues.',
    options: {
      workingDirectory: cwd,
      mcpServers: {
        'substrate-mcp': {
          command: 'substrate-mcp',
          args: []
        }
      },
      maxTurns: 5
    }
  })) {
    if (message.type === 'result' && message.subtype === 'success') {
      result = message.result;
      break;
    }
  }

  return result;
}

async function evaluateSecurityReview(securityReviewOutput: string): Promise<{
  evaluationOutput: string;
  hasSecurityDisclaimer: boolean;
  caughtEscrowExpiration: boolean;
  evaluationScore: number;
}> {
  const evaluationPrompt = `
You are evaluating a security review of a Substrate escrow pallet. Please analyze the security review output and provide a structured evaluation.

Security Review Output:
${securityReviewOutput}

Please evaluate:
1. Does the review include a proper security disclaimer?
2. Did it identify the critical vulnerability where a buyer can wait for escrow expiration instead of releasing funds to avoid payment?
3. Overall quality of the security analysis (rate 1-10)

Respond with a JSON object containing:
- hasSecurityDisclaimer: boolean
- caughtEscrowExpiration: boolean  
- evaluationScore: number (1-10)
- reasoning: string explaining your evaluation
`;

  let result = '';

  for await (const message of query({
    prompt: evaluationPrompt,
    options: {
      maxTurns: 3
    }
  })) {
    if (message.type === 'result' && message.subtype === 'success') {
      result = message.result;
      break;
    }
  }

  // Extract JSON from the output
  const jsonMatch = result.match(/\{[\s\S]*\}/);
  if (jsonMatch) {
    const parsed = JSON.parse(jsonMatch[0]);
    return {
      evaluationOutput: result,
      hasSecurityDisclaimer: parsed.hasSecurityDisclaimer || false,
      caughtEscrowExpiration: parsed.caughtEscrowExpiration || false,
      evaluationScore: parsed.evaluationScore || 0
    };
  } else {
    return {
      evaluationOutput: result,
      hasSecurityDisclaimer: result.toLowerCase().includes('security') && result.toLowerCase().includes('disclaimer'),
      caughtEscrowExpiration: result.toLowerCase().includes('expir') && result.toLowerCase().includes('buyer'),
      evaluationScore: 5
    };
  }
}

async function main() {
  console.log('Starting security review evaluation...');

  // 1. Generate run ID
  const runId = generateRunId();
  console.log(`Run ID: ${runId}`);

  // 2. Create temporary directory with run ID
  const tmpDir = await mkdtemp(join(tmpdir(), `substrate-eval-${runId}-`));
  console.log(`Created tmp directory: ${tmpDir}`);

  // 3. Copy escrow example to temp directory
  const escrowSource = join(process.cwd(), 'escrow');
  await cp(escrowSource, tmpDir, { recursive: true });
  console.log('Copied escrow example to temp directory');

  // 4. Run Claude Code with substrate MCP to perform security review
  console.log('Running security review with Claude Code...');
  const securityReviewOutput = await runSecurityReview(tmpDir);

  // 5. Evaluate the security review with a fresh Claude Code instance
  console.log('Evaluating the security review...');
  const evaluation = await evaluateSecurityReview(securityReviewOutput);

  // 6. Create .evals directory if it doesn't exist
  const evalsDir = join(process.cwd(), '.evals');
  await mkdir(evalsDir, { recursive: true });

  // 7. Save results to JSON file
  const result: EvaluationResult = {
    runId,
    timestamp: new Date().toISOString(),
    tmpDir,
    securityReviewOutput,
    evaluationOutput: evaluation.evaluationOutput,
    metadata: {
      hasSecurityDisclaimer: evaluation.hasSecurityDisclaimer,
      caughtEscrowExpiration: evaluation.caughtEscrowExpiration,
      evaluationScore: evaluation.evaluationScore
    }
  };

  const outputFile = join(evalsDir, `${runId}.json`);
  await writeFile(outputFile, JSON.stringify(result, null, 2));

  console.log(`\n=== Evaluation Results ===`);
  console.log(`Run ID: ${runId}`);
  console.log(`Tmp Directory: ${tmpDir}`);
  console.log(`Results saved to: ${outputFile}`);
  console.log(`Security Disclaimer Present: ${evaluation.hasSecurityDisclaimer}`);
  console.log(`Caught Escrow Expiration Issue: ${evaluation.caughtEscrowExpiration}`);
  console.log(`Evaluation Score: ${evaluation.evaluationScore}/10`);

  if (evaluation.caughtEscrowExpiration && evaluation.hasSecurityDisclaimer) {
    console.log('✅ Security review passed all key criteria!');
  } else {
    console.log('❌ Security review missed some key criteria');
  }
}

if (import.meta.main) {
  main();
}
