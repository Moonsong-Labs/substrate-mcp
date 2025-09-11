import { mkdtemp, cp, mkdir, writeFile } from 'fs/promises';
import { join } from 'path';
import { tmpdir } from 'os';
import { randomBytes } from 'crypto';
import { query } from '@anthropic-ai/claude-code';
import { Result, ok, err, fromThrowable } from 'neverthrow';

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

// Create fromThrowable wrappers for operations that might throw
const safeMkdtemp = fromThrowable(mkdtemp, (error) => error as Error);
const safeCp = fromThrowable(cp, (error) => error as Error);
const safeMkdir = fromThrowable(mkdir, (error) => error as Error);
const safeWriteFile = fromThrowable(writeFile, (error) => error as Error);
const safeJsonParse = fromThrowable(JSON.parse, (error) => error as Error);

function generateRunId(): string {
  return `eval-${Date.now()}-${randomBytes(4).toString('hex')}`;
}

async function runSecurityReview(cwd: string): Promise<Result<string, Error>> {

  let result = '';

  for await (const message of query({
    prompt: 'Use the substrate MCP server security_review prompt to analyze this escrow pallet implementation for security vulnerabilities, economic risks, and code quality issues.',
    options: {
      cwd: cwd,
      env: process.env,
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
    } else if (message.type === 'result' && message.is_error) {
      // Handle structured errors
      const errorMessage = `Claude Code error (${message.subtype}): ${message.result || 'Unknown error'}`;

      // Check for authentication-related errors in the result string
      if (message.result && (
        message.result.includes('authentication') ||
        message.result.includes('Authentication') ||
        message.result.includes('API key') ||
        message.result.includes('Unauthorized') ||
        message.result.includes('401') ||
        message.result.includes('403')
      )) {
        return err(new Error(`Authentication failed: ${message.result}`));
      }

      return err(new Error(errorMessage));
    }
  }

  return ok(result);
}

async function evaluateSecurityReview(securityReviewOutput: string): Promise<Result<{
  evaluationOutput: string;
  hasSecurityDisclaimer: boolean;
  caughtEscrowExpiration: boolean;
  evaluationScore: number;
}, Error>> {
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


  try {
    for await (const message of query({
      prompt: evaluationPrompt,
      options: {
        env: process.env,
        maxTurns: 3
      }
    })) {
      if (message.type === 'result' && message.subtype === 'success') {
        result = message.result;
        break;
      } else if (message.type === 'result' && message.is_error) {
        // Handle structured errors
        const errorMessage = `Claude Code error (${message.subtype}): ${message.result || 'Unknown error'}`;

        // Check for authentication-related errors in the result string
        if (message.result && (
          message.result.includes('authentication') ||
          message.result.includes('Authentication') ||
          message.result.includes('API key') ||
          message.result.includes('Unauthorized') ||
          message.result.includes('401') ||
          message.result.includes('403')
        )) {
          return err(new Error(`Authentication failed: ${message.result}`));
        }

        return err(new Error(errorMessage));
      }
    }

    // Extract JSON from the output
    const jsonMatch = result.match(/\{[\s\S]*\}/);
    if (jsonMatch) {
      const parseResult = safeJsonParse(jsonMatch[0]);
      if (parseResult.isErr()) {
        return err(new Error(`Failed to parse evaluation JSON: ${parseResult.error.message}`));
      }

      const parsed = parseResult.value;
      return ok({
        evaluationOutput: result,
        hasSecurityDisclaimer: parsed.hasSecurityDisclaimer || false,
        caughtEscrowExpiration: parsed.caughtEscrowExpiration || false,
        evaluationScore: parsed.evaluationScore || 0
      });
    } else {
      return ok({
        evaluationOutput: result,
        hasSecurityDisclaimer: result.toLowerCase().includes('security') && result.toLowerCase().includes('disclaimer'),
        caughtEscrowExpiration: result.toLowerCase().includes('expir') && result.toLowerCase().includes('buyer'),
        evaluationScore: 5
      });
    }
  }


  async function main() {
    console.log('Starting security review evaluation...');

    // 1. Generate run ID
    const runId = generateRunId();
    console.log(`Run ID: ${runId}`);

    // 2. Create temporary directory with run ID
    const tmpDirResult = await safeMkdtemp(join(tmpdir(), `substrate-eval-${runId}-`));
    if (tmpDirResult.isErr()) {
      console.error('Failed to create temporary directory:', tmpDirResult.error.message);
      process.exit(1);
    }
    const tmpDir = tmpDirResult.value;
    console.log(`Created tmp directory: ${tmpDir}`);

    // 3. Copy escrow example to temp directory
    const escrowSource = join(process.cwd(), 'examples', 'escrow');
    const cpResult = await safeCp(escrowSource, tmpDir, { recursive: true });
    if (cpResult.isErr()) {
      console.error('Failed to copy escrow example:', cpResult.error.message);
      process.exit(1);
    }
    console.log('Copied escrow example to temp directory');

    // 4. Run Claude Code with substrate MCP to perform security review
    console.log('Running security review with Claude Code...');
    const securityReviewResult = await runSecurityReview(tmpDir);

    if (securityReviewResult.isErr()) {
      console.error('Security review failed:', securityReviewResult.error.message);
      process.exit(1);
    }

    const securityReviewOutput = securityReviewResult.value;

    // 5. Evaluate the security review with a fresh Claude Code instance
    console.log('Evaluating the security review...');
    const evaluationResult = await evaluateSecurityReview(securityReviewOutput);

    if (evaluationResult.isErr()) {
      console.error('Security review evaluation failed:', evaluationResult.error.message);
      process.exit(1);
    }

    const evaluation = evaluationResult.value;

    // 6. Create .evals directory if it doesn't exist
    const evalsDir = join(process.cwd(), '.evals');
    const mkdirResult = await safeMkdir(evalsDir, { recursive: true });
    if (mkdirResult.isErr()) {
      console.error('Failed to create .evals directory:', mkdirResult.error.message);
      process.exit(1);
    }

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
    const writeResult = await safeWriteFile(outputFile, JSON.stringify(result, null, 2));
    if (writeResult.isErr()) {
      console.error('Failed to write results file:', writeResult.error.message);
      process.exit(1);
    }

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
