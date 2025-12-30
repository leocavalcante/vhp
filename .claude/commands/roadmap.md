# Roadmap - Continuous Development Cycle

Execute the complete development workflow for roadmap items **continuously** until the roadmap is complete or the user stops.

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

1. **Planning Phase** (SKIP if plans exist in `docs/plans/planned/`) - Use the architect agent to:
   - Analyze the roadmap and identify the next uncompleted item
   - Research existing codebase patterns
   - Design a detailed implementation plan
   - Create plan file in `docs/plans/planned/`

2. **Implementation Phase** - Use the coder agent to:
   - Follow the architect's plan step-by-step
   - Implement lexer, parser, AST, and interpreter changes
   - Add comprehensive test coverage
   - Ensure code follows VHP style guidelines

3. **Build & Validation** - Build the project:
   - Run `make release` to compile the code
   - Verify the build succeeds before proceeding to QA

4. **Quality Assurance Phase** - Use the qa agent to:
   - Run `make lint` to ensure code quality
   - Run `make test` to verify all tests pass
   - Check test coverage for the new feature
   - Validate PHP/VHP compatibility
   - Report any issues found

5. **Documentation Phase** - Use the tech-writer agent to:
   - Update AGENTS.md with new features
   - Update README.md with feature documentation
   - Update docs/ folder (features.md, roadmap.md, etc.)
   - Move plan from `docs/plans/planned/` to `docs/plans/implemented/`
   - Ensure all documentation is synchronized

6. **Git Workflow** - Commit and push changes:
   - Create atomic commits following Conventional Commits
   - Group changes logically: implementation, tests, documentation
   - Sign all commits with `-s` flag
   - Push to remote branch
   - Provide summary of completed work

7. **Loop to Next Item** - After successful completion:
   - Provide brief summary of what was completed
   - Check for remaining plans in `docs/plans/planned/` or roadmap items
   - If more items exist, **automatically start the next iteration**
   - If roadmap is complete, stop and inform the user

## Instructions

Execute each stage sequentially. If any stage fails:
- Stop the workflow immediately
- Report the failure to the user with details
- Wait for user intervention before continuing
- Do NOT proceed to the next roadmap item on failure

Between stages:
- Verify the previous stage completed successfully
- Provide brief status updates
- Check for any blockers

After completing each item:
- Summarize what was implemented
- List all commits created
- Announce moving to the next roadmap item
- Continue the loop automatically

## Stopping Conditions

The loop stops when:
1. A stage fails (wait for user intervention)
2. The roadmap is complete (no more plans or roadmap items)
3. The user manually stops the process

## Important Notes

- **This command runs continuously** - it will keep implementing roadmap items until done
- **Check `docs/plans/planned/` first** - skip architect if plans already exist
- Always run agents sequentially (one at a time)
- Build the project before running QA
- Create multiple atomic commits instead of one large commit
- Follow the Conventional Commits specification
- Sign all commits with `git commit -s`
- Only proceed to next stage if current stage succeeds
- Failures stop the entire loop - do not skip failed items
