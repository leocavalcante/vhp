# VHP Implementation Plans

This directory contains detailed implementation plans for PHP features to make VHP a true drop-in replacement for PHP.

## Directory Structure

```
plans/
├── planned/           # Features not yet implemented
│   ├── essential-features-index.md  # Master index of all plans
│   ├── pcre-regex.md              # PCRE regular expressions
│   ├── array-functions-extended.md # Extended array functions
│   ├── filesystem-functions.md     # Extended file system functions
│   ├── datetime-support.md         # Date/time support
│   ├── serialization-support.md     # Serialization support
│   ├── generators-execution.md     # Generator execution
│   ├── constants-support.md        # Constants support
│   └── goto-statement.md         # goto statement
└── implemented/        # Features that have been completed
    └── (moved here when completed)
```

## How to Use These Plans

### For Implementers

1. **Choose a plan** from the `planned/` directory
2. **Read the full plan** to understand requirements
3. **Follow the implementation phases** in order
4. **Create tests** as specified in the plan
5. **Update documentation** when complete
6. **Move the plan** to `implemented/` when done

### For Reviewers

Each plan includes:
- **Status**: Planned, In Progress, or Complete
- **Overview**: High-level description
- **Requirements**: Detailed feature requirements
- **Implementation Plan**: Phase-by-phase steps
- **Testing Strategy**: How to verify implementation
- **Success Criteria**: When is it considered complete?

## Current Status

### Planned Features (9)

1. [PCRE Regular Expressions](./planned/pcre-regex.md) - preg_match, preg_replace, etc.
2. [Extended Array Functions](./planned/array-functions-extended.md) - array_slice, array_splice, sort, etc.
3. [Extended File System Functions](./planned/filesystem-functions.md) - fopen, glob, scandir, etc.
4. [DateTime Support](./planned/datetime-support.md) - time(), date(), DateTime class
5. [Serialization Support](./planned/serialization-support.md) - serialize(), unserialize()
6. [Generator Execution](./planned/generators-execution.md) - yield, yield from, send()
7. [Constants Support](./planned/constants-support.md) - define(), trait constants
8. [goto Statement](./planned/goto-statement.md) - goto, labels
9. [Code Organization](./planned/code-organization.md) - Keep files under 300-500 lines

### Completed Features (0)

No features have been completed yet. Completed plans will be moved to `implemented/`.

## Implementation Priority

See [Essential Features Index](./planned/essential-features-index.md) for:
- Detailed priority breakdown
- Implementation order recommendation
- Progress tracking
- Success criteria for drop-in compatibility

## Plan Template

Each plan follows this structure:

```markdown
# Feature Name

## Status: Planned

## Overview
Brief description of the feature

## Background
Why this feature is important

## Requirements
Detailed feature requirements

## Implementation Plan
Phase-by-phase implementation steps

## Implementation Details
Code examples and technical details

## Dependencies
Required external crates or internal features

## Testing Strategy
How to test the implementation

## Success Criteria
When is the feature complete?

## References
Links to PHP documentation and RFCs
```

## Contributing

When adding new plans:

1. **Create plan file** in `planned/` directory
2. **Follow the template** structure
3. **Include detailed requirements** and implementation phases
4. **Add to the index** in `essential-features-index.md`
5. **Test against PHP behavior** where possible

## Completion Checklist

When completing a feature:

- [ ] All implementation phases are complete
- [ ] All tests pass
- [ ] Documentation is updated (AGENTS.md, README.md, docs/)
- [ ] Plan file is moved to `implemented/`
- [ ] Progress is updated in `essential-features-index.md`

## Related Documentation

- [AGENTS.md](../../AGENTS.md) - Project instructions for AI assistants
- [README.md](../../README.md) - Project overview
- [docs/roadmap.md](../roadmap.md) - Project roadmap
- [docs/features.md](../features.md) - Feature documentation

## Questions?

Refer to the main project documentation or open an issue on GitHub.
