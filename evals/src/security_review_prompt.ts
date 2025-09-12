import { join } from 'path';
import { tmpdir } from 'os';
import { randomBytes } from 'crypto';
import { query, type SDKMessage, type SDKAssistantMessage } from '@anthropic-ai/claude-code';
import { Result, ok, err } from 'neverthrow';
import { mkdtempSync, cpSync, mkdirSync, writeFileSync } from 'fs';
import { config } from 'dotenv';
import { logger } from './utils/logger.ts';

interface RunMetadata {
  id: string;
  directory: string;
  timestamp: string;
}

interface Run {
  metadata: RunMetadata;
  output: {
    assistantMessages: string[];
  };
}

interface Score {
  key: string;
  score: number;
}

interface Eval {
  runId: string;
  output: {
    evaluatorReasoning: string;
    scores: Score[];
  };
}


function generateRunId(evalName: string): string {
  return `${evalName}-${randomBytes(4).toString('hex')}`;
}

function getRunDirectory(runId: string): string {
  return join(process.cwd(), '.evals', runId);
}

function setup(evalName: string, exampleName: string): RunMetadata {
  const runId = generateRunId(evalName);
  const runDir = mkdtempSync(join(tmpdir(), `substrate-eval-${runId}-`));

  const exampleSource = join(process.cwd(), 'examples', exampleName);
  cpSync(exampleSource, runDir, { recursive: true });

  return {
    id: runId,
    directory: runDir,
    timestamp: new Date().toISOString()
  };
}

async function run(metadata: RunMetadata): Promise<Result<Run, Error>> {

  const assistantMessages: string[] = [];

  for await (const message: SDKMessage of query({
    prompt: 'Use the substrate MCP server security_review prompt to analyze this escrow pallet implementation for security vulnerabilities, economic risks, and code quality issues.',
    options: {
      cwd: metadata.directory,
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

  const runResult: Run = {
    metadata,
    output: {
      assistantMessages
    }
  };

  // Create .evals directory and run-specific directory
  const runDir = getRunDirectory(metadata.id);
  mkdirSync(runDir, { recursive: true });

  // Save run results to run.json
  const runFile = join(runDir, 'run.json');
  writeFileSync(runFile, JSON.stringify(runResult, null, 2));

  return ok(runResult);
}

async function evaluate(run: Run): Promise<Result<Eval, Error>> {
  // Concatenate all assistant messages for evaluation
  const securityReviewOutput = run.output.assistantMessages.join('\n\n');

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
  let hasSecurityDisclaimer, caughtEscrowExpiration, evaluationScore;

  if (jsonMatch) {
    const parsed = JSON.parse(jsonMatch[0]);
    hasSecurityDisclaimer = parsed.hasSecurityDisclaimer || false;
    caughtEscrowExpiration = parsed.caughtEscrowExpiration || false;
    evaluationScore = parsed.evaluationScore || 0;
  } else {
    hasSecurityDisclaimer = result.toLowerCase().includes('security') && result.toLowerCase().includes('disclaimer');
    caughtEscrowExpiration = result.toLowerCase().includes('expir') && result.toLowerCase().includes('buyer');
    evaluationScore = 5;
  }

  const evalResult: Eval = {
    runId: run.metadata.id,
    output: {
      evaluatorReasoning: result,
      scores: [
        { key: 'hasSecurityDisclaimer', score: hasSecurityDisclaimer ? 1 : 0 },
        { key: 'caughtEscrowExpiration', score: caughtEscrowExpiration ? 1 : 0 },
        { key: 'evaluationScore', score: evaluationScore }
      ]
    }
  };

  // Save evaluation results to eval.json
  const runDir = getRunDirectory(run.metadata.id);
  const evalFile = join(runDir, 'eval.json');
  writeFileSync(evalFile, JSON.stringify(evalResult, null, 2));

  return ok(evalResult);
}


async function main() {
  config();
  logger.info('Starting security review prompt evaluation...');

  const runMetadata = setup('security_review_prompt', 'escrow');

  // Run Claude Code with substrate MCP to perform security review
  logger.info(`Starting run ${runMetadata.id} on ${runMetadata.directory}`);
  const runResult = await run(runMetadata);


  if (runResult.isErr()) {
    logger.error('Security review failed:', runResult.error.message);
    process.exit(1);
  }

  const run = runResult.value;
  logger.info(`\n=== Run Results ===`);
  logger.info(JSON.stringify(run, null, 2));

  // Evaluate the security review with a fresh Claude Code instance
  logger.info('Evaluating the security review...');
  const evaluationResult = await evaluate(run);

  if (evalResult.isErr()) {
    logger.error('Security review evaluation failed:', evaluationResult.error.message);
    process.exit(1);
  }

  const eval = evalResult.value;

  logger.info(`\n=== Eval Results ===`);
  logger.info(JSON.stringify(eval, null, 2));

  logger.info(`Results saved to: ${getRunDirectory(runMetadata.id)}`);
}

if (import.meta.main) {
  main();
}
