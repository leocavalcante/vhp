# Next Roadmap Item - Full Development Cycle

Execute the complete development workflow for the next roadmap item:

## Workflow Stages

1. **Planning Phase** - Use the @Architect agent to:
   - Analyze the roadmap and identify the next uncompleted item
   - Research existing codebase patterns
   - Design a detailed implementation plan
   - Create plan file in `docs/plans/planned/`

2. **Implementation Phase** - Use the @Coder agent to:
   - Follow the architect's plan step-by-step
   - Implement lexer, parser, AST, and interpreter changes
   - Add comprehensive test coverage
   - Ensure code follows VHP style guidelines

3. **Build & Validation** - Build the project:
   - Run `make release` to compile the code
   - Verify the build succeeds before proceeding to QA

4. **Quality Assurance Phase** - Use the @QA agent to:
   - Run `make lint` to ensure code quality
   - Run `make test` to verify all tests pass
   - Check test coverage for the new feature
   - Validate PHP/VHP compatibility
   - Report any issues found

5. **Documentation Phase** - Use the @Tech Writer agent to:
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

## Instructions

Execute each stage sequentially. If any stage fails:
- Stop the workflow
- Report the failure to the user with details
- Wait for user intervention before continuing

Between stages:
- Verify the previous stage completed successfully
- Provide brief status updates
- Check for any blockers

After completion:
- Summarize what was implemented
- List all commits created
- Provide the feature summary for reference

## Important Notes

- Always run agents sequentially (one at a time)
- Build the project before running QA
- Create multiple atomic commits instead of one large commit
- Follow the Conventional Commits specification
- Sign all commits with `git commit -s`
- Only proceed to next stage if current stage succeeds
- If roadmap is complete, inform the user
