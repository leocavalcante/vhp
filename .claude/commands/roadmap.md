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
- **Missing context**: Research the codebase or check `docs/learnings.md`

## Continuous Loop

This command runs in a loop:
1. Complete one roadmap item (plan → implement → review → validate → document → commit)
2. Automatically proceed to the next item
3. Repeat until roadmap is complete or user intervention

## Pre-Check: Learnings & Existing Plans

Before each iteration:

1. **Check `docs/learnings.md`** for relevant patterns and pitfalls
2. **Check `docs/plans/planned/`** for existing plans:
   - If plans exist, **skip the Planning Phase** and proceed to Implementation
   - Pick the next feasible plan (prioritize by roadmap order or simplicity)
   - Only run the architect agent if no planned plans exist

## Enhanced Workflow Stages (Per Item)

```
┌─────────────┐
│  Stage 1    │ Planning (architect) - SKIP if plan exists
└──────┬──────┘
       ↓
┌─────────────┐
│  Stage 2    │ Implementation (coder)
└──────┬──────┘
       ↓
┌─────────────┐
│  Stage 3    │ Build & Validation
└──────┬──────┘
       ↓
┌─────────────┐
│  Stage 4    │ Code Review (reviewer) - NEW
└──────┬──────┘
       ↓
┌─────────────┐
│  Stage 5    │ Quality Assurance (qa)
└──────┬──────┘
       ↓
┌─────────────┐
│  Stage 6    │ Documentation (tech-writer)
└──────┬──────┘
       ↓
┌─────────────┐
│  Stage 7    │ Learnings Capture
└──────┬──────┘
       ↓
┌─────────────┐
│  Stage 8    │ Git Workflow
└──────┬──────┘
       ↓
┌─────────────┐
│  Stage 9    │ Loop to Next Item
└─────────────┘
```

### Stage 1: Planning Phase (SKIP if plans exist in `docs/plans/planned/`)

Use the architect agent to:
- Check `docs/learnings.md` for relevant patterns
- Analyze the roadmap and identify the next uncompleted item
- Research existing codebase patterns
- Design a detailed implementation plan with:
  - User Documentation Draft
  - Test Strategy with explicit test cases
  - Implementation Phases (for complex features)
  - Potential Pitfalls section
- Create plan file in `docs/plans/planned/`

### Stage 2: Implementation Phase

Use the coder agent to:
- Read `docs/learnings.md` for relevant patterns
- Follow the architect's plan step-by-step
- Implement ALL test cases from the Test Strategy
- Follow Implementation Phases if defined
- Use the Checklist for Coder to verify completeness
- Refactor if needed to maintain code quality
- Verify compilation with `cargo build --release`

**On implementation errors**: The coder agent should fix issues autonomously. Do not stop for minor problems.

### Stage 3: Build & Validation

Build the project:
- Run `make release` to compile the code
- Verify the build succeeds before proceeding

**On build failures**: Delegate to coder to fix. Retry up to 3 times.

### Stage 4: Code Review Phase (NEW)

Use the reviewer agent to:
- Verify implementation matches the plan
- Check code quality (clean, idiomatic Rust)
- Verify test quality (tests actually test what they claim)
- Review design decisions (appropriate complexity)
- Delegate any fixes to coder agent

**On review issues**: Reviewer delegates fixes to coder, then re-reviews.

### Stage 5: Quality Assurance Phase

Use the qa agent to:
- Run `cargo clippy -- -D warnings`
- Run `cargo build --release`
- Run `make test`
- Perform root cause analysis on any failures
- Delegate fixes to coder agent
- Track patterns for `docs/learnings.md`

**On QA failures**:
- Analyze root cause
- Delegate fix to coder
- Retry up to 3 times
- Update learnings if new pattern discovered

### Stage 6: Documentation Phase

Use the tech-writer agent to:
- Update AGENTS.md (features, roadmap)
- Update README.md (features)
- Update docs/features.md (full documentation)
- Update docs/roadmap.md (mark complete)
- Add examples to docs/examples.md
- Move plan from `docs/plans/planned/` to `docs/plans/implemented/`
- Verify consistency across all docs

### Stage 7: Learnings Capture

Ensure learnings are captured:
- Review QA report for any new patterns discovered
- Review reviewer report for any design learnings
- If new learnings exist, verify they were added to `docs/learnings.md`
- If not added, add them using this format:

```markdown
### [Category]: [Brief Title]

**Date**: [Current date]
**Feature**: [Feature name]
**Issue**: [What was learned]
**Prevention**: [How to apply this going forward]
```

### Stage 8: Git Workflow

Commit and push changes:
- Review all changes: `git status`, `git diff`
- Group changes logically:
  - Implementation code
  - Tests
  - Documentation
  - Learnings (if updated)
- Create separate atomic commits:
  - `feat(scope): description` for implementation
  - `test(scope): description` for tests
  - `docs: description` for documentation
- Sign all commits with `-s` flag
- Push to remote branch

### Stage 9: Loop to Next Item

After successful completion:
- Provide brief summary of what was completed
- List all commits created
- Note any learnings captured
- Check for remaining plans in `docs/plans/planned/` or roadmap items
- If more items exist, **automatically start the next iteration**
- If roadmap is complete, stop and inform the user

## Error Recovery Protocol

When errors occur, follow this protocol:

1. **First attempt**: Analyze the error and apply an obvious fix
2. **Second attempt**: Research similar patterns in codebase or `docs/learnings.md`
3. **Third attempt**: Try an alternative approach
4. **After 3 failures**: Consider rollback, then stop and report

For each error type:
- **Compilation errors**: Fix syntax/type issues based on error messages
- **Test failures**: Debug test, fix assertion or implementation
- **Lint errors**: Apply automatic fixes or manual corrections
- **Review issues**: Delegate to coder, re-review
- **Git conflicts**: Resolve using ours/theirs based on context

## Rollback Protocol

If a feature causes critical issues after 3 fix attempts:

1. **Revert commits**:
   ```bash
   git revert --no-commit <commit-hash>
   git commit -s -m "revert(feature): rollback [FEATURE] due to [REASON]"
   ```

2. **Document in learnings**:
   ```markdown
   ### Rollback: [Feature Name]
   **Date**: [Date]
   **Reason**: [Why rollback was needed]
   **Root Cause**: [What went wrong]
   **Prevention**: [How to avoid this in future]
   ```

3. **Move plan back** to `docs/plans/planned/`
4. **Continue to next item** (don't stop the loop)

## Stopping Conditions

The loop stops ONLY when:
1. A stage fails after 3 automatic fix attempts AND rollback is not possible
2. The roadmap is complete (no more plans or roadmap items)
3. The user manually stops the process

**DO NOT stop for:**
- Minor questions or clarifications
- Implementation choices
- Commit message wording
- Documentation phrasing
- Fixable errors (try 3 times first)

## Important Notes

- **This command runs continuously and autonomously**
- **NEVER ask questions** - figure it out from context or learnings
- **Check `docs/learnings.md` first** - learn from past issues
- **Check `docs/plans/planned/` first** - skip architect if plans already exist
- **Always run reviewer** before QA (catches design issues early)
- **Always capture learnings** - improve the system over time
- Always run agents sequentially (one at a time)
- Build the project before running QA
- Create multiple atomic commits instead of one large commit
- Follow the Conventional Commits specification
- Sign all commits with `git commit -s`
- Only proceed to next stage if current stage succeeds
- Attempt automatic fixes before stopping on failures
- Trust the plan files - they contain all needed context
- Use rollback as last resort before stopping
