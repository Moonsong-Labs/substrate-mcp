# Evals

Evaluations to assess performace of different development flows using the `substrate-mcp`.

## Main Flow

An eval `run` will work on an `example` (by copying it in a `tmp` directory) and will perform a `task` (for example, running a specific prompt on an agent in the context of the `example`). It will then will run some `evals` on the task result to determine how well the task was performed. A run is given an `id` and it's metadata and results are stored in a corresponding `.evals` folder

## File Organization

### Directory Structure
```
.evals/
└── {run-id}/
    ├── run_metadata.json    # Run information (ID, task directory, timestamp)
    ├── task_result.json     # Task execution results and assistant messages
    └── eval.json           # Evaluation scores and reasoning
```

## Running an Eval

### Prerequisites

1. [Bun](https://bun.sh) runtime (required to run evals):
   ```bash
   curl -fsSL https://bun.sh/install | bash
   ```



### Running the Eval

1. Set up environment variables in `.env`:
   ```bash
   # Add Agent configuration (e.g: Anthropic credentials) if needed
   LOG_LEVEL=INFO
   ```

2. Run the eval. Evals are in the `src/evals` directory. You can run them directly with bun:

```bash
bun src/evals/security-review.ts
```

Results are saved to `.evals/{run-id}/`

### Cleanup

Clean up all eval runs and temporary directories:

```bash
bun run clean
```
