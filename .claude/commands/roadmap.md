# Roadmap - Fully Autonomous Development Cycle

Execute the complete development workflow for roadmap items **continuously and autonomously** until the roadmap is complete or the user stops.

## Autonomous Behavior

**This command is FULLY AUTONOMOUS.** You must:
- NEVER ask questions or wait for user input
- NEVER use AskUserQuestion tool
- Make all decisions independently based on context and best practices
- Handle errors by attempting fixes automatically before reporting
- Continue working until the roadmap is complete or you truly cannot proceed

## Decision-Making Guidelines

When you encounter ambiguity or choices:
- **Plan selection**: Pick the simplest/smallest plan first to build momentum
- **Implementation choices**: Follow existing codebase patterns
- **Error handling**: Attempt to fix errors automatically (up to 3 times) before stopping
- **Test failures**: Fix the failing tests or implementation, don't ask for guidance
- **Merge conflicts**: Resolve automatically using standard git practices
- **Missing context**: Research the codebase to find the answer

## Continuous Loop

This command runs in a loop:
1. Complete one roadmap item (plan → implement → validate → document → commit)
2. Automatically proceed to the next item
3. Repeat until roadmap is complete or user intervention

## Pre-Check: Existing Plans

Before each iteration, check `docs/plans/planned/`:
- If plans exist, **skip the Planning Phase** and proceed directly to Implementation
- Pick the next feasible plan from the planned directory (prioritize by roadmap order or simplicity)
- Only run the architect agent if no planned plans exist

## Workflow Stages (Per Item)

### 1. Planning Phase (SKIP if plans exist in `docs/plans/planned/`)

Use the architect agent to:
- Analyze the roadmap and identify the next uncompleted item
- Research existing codebase patterns
- Design a detailed implementation plan
- Create plan file in `docs/plans/planned/`

### 2. Implementation Phase

Use the coder agent to:
- Follow the architect's plan step-by-step
- Implement lexer, parser, AST, and interpreter changes
- Add comprehensive test coverage
- Ensure code follows VHP style guidelines

**On implementation errors**: The coder agent should fix issues autonomously. Do not stop for minor problems.

### 3. Build & Validation

Build the project:
- Run `make release` to compile the code
- Verify the build succeeds before proceeding to QA

**On build failures**: Analyze the error, fix the code, and retry. Only stop after 3 failed attempts.

### 4. Quality Assurance Phase

Use the qa agent to:
- Run `make lint` to ensure code quality
- Run `make test` to verify all tests pass
- Check test coverage for the new feature
- Validate PHP/VHP compatibility

**On QA failures**:
- Lint errors: Fix automatically and re-run
- Test failures: Analyze, fix implementation or test, and retry
- Only stop after 3 failed fix attempts

### 5. Documentation Phase

Use the tech-writer agent to:
- Update AGENTS.md with new features
- Update README.md with feature documentation
- Update docs/ folder (features.md, roadmap.md, etc.)
- Move plan from `docs/plans/planned/` to `docs/plans/implemented/`
- Ensure all documentation is synchronized

### 6. Git Workflow

Commit and push changes:
- Create atomic commits following Conventional Commits
- Group changes logically: implementation, tests, documentation
- Sign all commits with `-s` flag
- Push to remote branch
- Provide summary of completed work

**Commit message decisions**: Use these conventions automatically:
- New features: `feat(scope): description`
- Bug fixes: `fix(scope): description`
- Tests: `test(scope): description`
- Docs: `docs(scope): description`

### 7. Loop to Next Item

After successful completion:
- Provide brief summary of what was completed
- Check for remaining plans in `docs/plans/planned/` or roadmap items
- If more items exist, **automatically start the next iteration**
- If roadmap is complete, stop and inform the user

## Error Recovery Protocol

When errors occur, follow this protocol:

1. **First attempt**: Analyze the error and apply an obvious fix
2. **Second attempt**: Research similar patterns in codebase for guidance
3. **Third attempt**: Try an alternative approach
4. **After 3 failures**: Stop and report the issue with full context

For each error type:
- **Compilation errors**: Fix syntax/type issues based on error messages
- **Test failures**: Debug test, fix assertion or implementation
- **Lint errors**: Apply automatic fixes or manual corrections
- **Git conflicts**: Resolve using ours/theirs based on context

## Autonomous Execution Instructions

Execute each stage sequentially with full autonomy:
- Make decisions without asking
- Fix problems without waiting for input
- Research codebase when context is needed
- Continue automatically to next stage on success

Between stages:
- Verify the previous stage completed successfully
- Provide brief status updates (informational, not questions)
- Proceed immediately to next stage

After completing each item:
- Summarize what was implemented
- List all commits created
- Announce moving to the next roadmap item
- Continue the loop automatically (no confirmation needed)

## Stopping Conditions

The loop stops ONLY when:
1. A stage fails after 3 automatic fix attempts
2. The roadmap is complete (no more plans or roadmap items)
3. The user manually stops the process

**DO NOT stop for:**
- Minor questions or clarifications
- Implementation choices
- Commit message wording
- Documentation phrasing

## Important Notes

- **This command runs continuously and autonomously**
- **NEVER ask questions** - figure it out from context
- **Check `docs/plans/planned/` first** - skip architect if plans already exist
- Always run agents sequentially (one at a time)
- Build the project before running QA
- Create multiple atomic commits instead of one large commit
- Follow the Conventional Commits specification
- Sign all commits with `git commit -s`
- Only proceed to next stage if current stage succeeds
- Attempt automatic fixes before stopping on failures
- Trust the plan files - they contain all needed context
