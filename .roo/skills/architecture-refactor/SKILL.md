---
name: architecture-refactor
description: Apply DRY, SOLID, LEAN, and KISS principles to software architecture. Use when analyzing cliff-watch architecture, proposing improvements, designing new components, or reviewing module boundaries.
---

# Architecture Refactor

## When to use this skill

Use this skill when:
- Analyzing existing architecture for principle violations
- Proposing architectural improvements
- Designing new components or modules
- Reviewing module boundaries and dependencies
- Creating documentation for architectural decisions
- Ensuring architecture follows DRY, SOLID, LEAN, and KISS principles

## When NOT to use this skill

Do NOT use this skill when:
- Refactoring code at the function or class level (use `rust-code-quality` or `typescript-code-quality` skill instead)
- Debugging runtime issues (use Debug mode instead)
- The task requires system-level modifications
- Making quick code fixes without architectural consideration

## Inputs required from the user

The user should provide:
- The scope of architectural analysis (entire project, specific modules, etc.)
- The quality principles to focus on (DRY, SOLID, LEAN, KISS, or all)
- Context about requirements or constraints if designing new components

## Workflow

1. **Analyze** current architecture
   - Review module structure and boundaries
   - Identify violations of DRY, SOLID, LEAN, and KISS principles
   - Analyze dependency graph and coupling
   - Document findings

2. **Design** improvements
   - Design new module boundaries if needed
   - Apply SOLID principles at architectural level
   - Remove architectural waste (LEAN)
   - Simplify architecture (KISS)
   - Plan incremental changes

3. **Document** decisions
   - Create Architecture Decision Records (ADRs)
   - Document current and proposed architecture
   - Update roadmaps with architectural tasks
   - Provide clear rationale for all changes

4. **Validate** proposals
   - Ensure principles are better followed
   - Verify proposals are actionable
   - Check for new violations introduced
   - Update documentation

## Examples

### Analyzing architecture for violations

1. Review module structure in `crates/` and `clients/`
2. Identify modules with multiple responsibilities (Single Responsibility)
3. Look for duplicated patterns across modules (DRY)
4. Find unnecessary layers or abstractions (LEAN)
5. Document findings with clear recommendations

### Proposing module boundary improvements

1. Map current module dependencies
2. Identify tight coupling and circular dependencies
3. Design new boundaries with clear separation of concerns
4. Create ADR documenting the decision
5. Update roadmap with refactoring tasks

### Designing a new component

1. Understand requirements and integration points
2. Define component interface and responsibilities
3. Design integration with existing architecture
4. Ensure minimal coupling to existing components
5. Create ADR documenting the design decision

## Troubleshooting

- **Architecture seems too complex:** Apply KISS principle - simplify by removing unnecessary layers and abstractions
- **Cannot eliminate duplication:** Consider if duplication is actually variation that requires different implementations
- **Modules are tightly coupled:** Apply Dependency Inversion principle - depend on abstractions, not concretions
- **Proposals are too large:** Break down into smaller, incremental changes that can be validated independently

## Related resources

- Read [`.roo/rules-architecture-refactor/2_best_practices.xml`](../rules-architecture-refactor/2_best_practices.xml) for detailed architectural practices
- Read [`.roo/rules-architecture-refactor/3_common_patterns.xml`](../rules-architecture-refactor/3_common_patterns.xml) for common architectural patterns
- Read [`.roo/rules-architecture-refactor/4_decision_guidance.xml`](../rules-architecture-refactor/4_decision_guidance.xml) for decision-making criteria

## Architecture Decision Record (ADR) Template

When creating ADRs, include:

1. **Context**: What is the issue we're facing?
2. **Decision**: What did we decide?
3. **Status**: Proposed, Accepted, Deprecated, Superseded
4. **Consequences**: What becomes easier or harder?
5. **Alternatives considered**: What other options did we explore?
