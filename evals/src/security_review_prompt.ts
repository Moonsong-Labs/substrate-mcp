import { join } from 'path';
import { tmpdir } from 'os';
import { randomBytes } from 'crypto';
import { query, type SDKMessage, type SDKAssistantMessage } from '@anthropic-ai/claude-code';
import { Result, ok, err } from 'neverthrow';
import { mkdtempSync, cpSync, mkdirSync, writeFileSync } from 'fs';
import { config } from 'dotenv';
import { logger } from './utils/logger.ts';

interface EvaluationResult {
  runId: string;
  timestamp: string;
  tmpDir: string;
  securityReviewOutput: string[];
  evaluationOutput: string;
  metadata: {
    hasSecurityDisclaimer: boolean;
    caughtEscrowExpiration: boolean;
    evaluationScore: number;
  };
}


function generateRunId(evalName: string): string {
  return `${evalName}-${randomBytes(4).toString('hex')}`;
}

async function runSecurityReview(cwd: string): Promise<Result<string[], Error>> {

  const assistantMessages: string[] = [];

  for await (const message: SDKMessage of query({
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
      maxTurns: 100
    }
  })) {
    logger.debug('=== DEBUG MESSAGE ===');
    logger.debug('Type:', message.type);
    if (message.type === 'assistant') {
      logger.debug('Content:', message.message.content);
    }
    logger.debug('Full message:', JSON.stringify(message, null, 2));
    logger.debug('=====================');

    // Collect assistant text messages with proper type checking
    if (message.type === 'assistant') {
      const assistantMessage = message as SDKAssistantMessage;
      // Check if content is a string (text content) rather than an array (structured content)
      if (typeof assistantMessage.message.content === 'string') {
        assistantMessages.push(assistantMessage.message.content);
      } else if (Array.isArray(assistantMessage.message.content)) {
        // Handle structured content - extract text blocks
        for (const block of assistantMessage.message.content) {
          if (block.type === 'text') {
            assistantMessages.push(block.text);
          }
        }
      }
    }

    if (message.type === 'result' && message.subtype === 'success') {
      break;
    } else if (message.type === 'result' && message.is_error) {
      // Handle structured errors  
      const errorMessage = `Claude Code error (${message.subtype}): Unknown error`;
      return err(new Error(errorMessage));
    }
  }

  return ok(assistantMessages);
}

async function evaluateSecurityReview(securityReviewMessages: string[]): Promise<Result<{
  evaluationOutput: string;
  hasSecurityDisclaimer: boolean;
  caughtEscrowExpiration: boolean;
  evaluationScore: number;
}, Error>> {
  // Concatenate all assistant messages for evaluation
  const securityReviewOutput = securityReviewMessages.join('\n\n');

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
      env: process.env,
      maxTurns: 3
    }
  })) {
    if (message.type === 'result' && message.subtype === 'success') {
      result = message.result;
      break;
    } else if (message.type === 'result' && message.is_error) {
      // Handle structured errors
      const errorMessage = `Claude Code error (${message.subtype}): Unknown error`;
      return err(new Error(errorMessage));
    }
  }

  // Extract JSON from the output
  const jsonMatch = result.match(/\{[\s\S]*\}/);
  if (jsonMatch) {
    const parsed = JSON.parse(jsonMatch[0]);
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
  config();
  logger.info('Starting security review evaluation...');

  // 1. Generate run ID
  const runId = generateRunId('security_review_prompt');
  logger.info(`Run ID: ${runId}`);

  // 2. Create temporary directory with run ID
  const tmpDir = mkdtempSync(join(tmpdir(), `substrate-eval-${runId}-`));
  logger.info(`Created tmp directory: ${tmpDir}`);

  // 3. Copy escrow example to temp directory
  const escrowSource = join(process.cwd(), 'examples', 'escrow');
  cpSync(escrowSource, tmpDir, { recursive: true });
  logger.info('Copied escrow example to temp directory');

  // 4. Run Claude Code with substrate MCP to perform security review
  logger.info('Running security review with Claude Code...');
  const securityReviewResult = await runSecurityReview(tmpDir);
  logger.info('Security Review Result');

  if (securityReviewResult.isErr()) {
    logger.info('Security review failed:', securityReviewResult.error.message);
    process.exit(1);
  }

  const securityReviewMessages = securityReviewResult.value;

  // 5. Evaluate the security review with a fresh Claude Code instance
  logger.info('Evaluating the security review...');
  const evaluationResult = await evaluateSecurityReview(securityReviewMessages);

  if (evaluationResult.isErr()) {
    logger.info('Security review evaluation failed:', evaluationResult.error.message);
    process.exit(1);
  }

  const evaluation = evaluationResult.value;

  // 6. Create .evals directory and run-specific directory
  const evalsDir = join(process.cwd(), '.evals');
  const runDir = join(evalsDir, runId);
  mkdirSync(runDir, { recursive: true });

  // 7. Save run results to run.json
  const runResult = {
    runId,
    timestamp: new Date().toISOString(),
    tmpDir,
    securityReviewOutput: securityReviewMessages
  };

  const runFile = join(runDir, 'run.json');
  writeFileSync(runFile, JSON.stringify(runResult, null, 2));

  // 8. Save evaluation results to eval.json
  const evalResult = {
    runId,
    timestamp: new Date().toISOString(),
    evaluationOutput: evaluation.evaluationOutput,
    metadata: {
      hasSecurityDisclaimer: evaluation.hasSecurityDisclaimer,
      caughtEscrowExpiration: evaluation.caughtEscrowExpiration,
      evaluationScore: evaluation.evaluationScore
    }
  };

  const evalFile = join(runDir, 'eval.json');
  writeFileSync(evalFile, JSON.stringify(evalResult, null, 2));

  logger.info(`\n=== Evaluation Results ===`);
  logger.info(`Run ID: ${runId}`);
  logger.info(`Tmp Directory: ${tmpDir}`);
  logger.info(`Results saved to: ${runDir}`);
  logger.info(`Security Disclaimer Present: ${evaluation.hasSecurityDisclaimer}`);
  logger.info(`Caught Escrow Expiration Issue: ${evaluation.caughtEscrowExpiration}`);
  logger.info(`Evaluation Score: ${evaluation.evaluationScore}/10`);

  if (evaluation.caughtEscrowExpiration && evaluation.hasSecurityDisclaimer) {
    logger.info('✅ Security review passed all key criteria!');
  } else {
    logger.info('❌ Security review missed some key criteria');
  }
}

if (import.meta.main) {
  main();
}
