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
  task_directory: string;
  timestamp: string;
}

interface TaskResult {
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

  const runMetadata: RunMetadata = {
    id: runId,
    task_directory: runDir,
    timestamp: new Date().toISOString()
  };

  // Create .evals directory and run-specific directory
  const evalsRunDir = getRunDirectory(runId);
  mkdirSync(evalsRunDir, { recursive: true });

  // Save run metadata to run_metadata.json
  const runMetadataFile = join(evalsRunDir, 'run_metadata.json');
  writeFileSync(runMetadataFile, JSON.stringify(runMetadata, null, 2));

  return runMetadata;
}

async function runTask(metadata: RunMetadata): Promise<Result<TaskResult, Error>> {

  const assistantMessages: string[] = [];

  for await (const message: SDKMessage of query({
    prompt: `/substrateMcp:security_review (MCP) "Analyze the escrow pallet"`,
    options: {
      cwd: metadata.task_directory,
      env: process.env,
      mcpServers: {
        substrateMcp: {
          command: 'substrate-mcp',
          args: []
        }
      },
      permissions: {
        allowRead: true,
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

  const taskResult: TaskResult = {
    output: {
      assistantMessages
    }
  };

  // Save task results to task.json
  const runDir = getRunDirectory(metadata.id);
  const taskFile = join(runDir, 'task_result.json');
  writeFileSync(taskFile, JSON.stringify(taskResult, null, 2));

  return ok(taskResult);
}

async function runEval(taskResult: TaskResult, runMetadata: RunMetadata): Promise<Result<Eval, Error>> {
  // Concatenate all assistant messages for evaluation
  const securityReviewOutput = taskResult.output.assistantMessages.join('\n\n');

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
    runId: runMetadata.id,
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
  const runDir = getRunDirectory(runMetadata.id);
  const evalFile = join(runDir, 'eval.json');
  writeFileSync(evalFile, JSON.stringify(evalResult, null, 2));

  return ok(evalResult);
}


async function main() {
  config();
  logger.info('Starting security review prompt evaluation...');

  const runMetadata = setup('security_review_prompt', 'escrow');

  // Run Claude Code with substrate MCP to perform security review
  logger.info(`Starting run ${runMetadata.id} on ${runMetadata.task_directory}`);
  const taskResult = await runTask(runMetadata);

  if (taskResult.isErr()) {
    logger.error('Security review failed:', taskResult.error.message);
    process.exit(1);
  }

  const taskObj = taskResult.value;
  logger.info(`\n=== Task Results ===`);
  logger.info(JSON.stringify(taskObj, null, 2));

  // Evaluate the security review with a fresh Claude Code instance
  logger.info('Evaluating the security review...');
  const evaluationResult = await runEval(taskObj, runMetadata);

  if (evaluationResult.isErr()) {
    logger.error('Security review evaluation failed:', evaluationResult.error.message);
    process.exit(1);
  }

  const evalResult = evaluationResult.value;

  logger.info(`\n=== Eval Results ===`);
  logger.info(JSON.stringify(evalResult, null, 2));

  logger.info(`Results saved to: ${getRunDirectory(runMetadata.id)}`);
}

if (import.meta.main) {
  main();
}
