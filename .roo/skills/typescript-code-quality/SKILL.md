---
name: typescript-code-quality
description: Refactor TypeScript code to follow DRY, SOLID, LEAN, and KISS principles. Use when working with TypeScript code in cliff-watch-witness VS Code extension, refactoring existing modules, or reviewing code quality.
---

# TypeScript Code Quality

## When to use this skill

Use this skill when:
- Refactoring existing TypeScript code to eliminate duplication (DRY)
- Applying SOLID principles to TypeScript modules and classes
- Removing unnecessary complexity and waste (LEAN)
- Simplifying over-engineered code (KISS)
- Writing new TypeScript code with quality principles in mind
- Reviewing TypeScript code for adherence to quality principles

## When NOT to use this skill

Do NOT use this skill when:
- Working with Rust code (use `rust-code-quality` skill instead)
- Making architectural decisions (use `architecture-refactor` skill instead)
- Debugging runtime issues (use Debug mode instead)
- The task requires system-level modifications

## Inputs required from the user

The user should provide:
- The specific TypeScript files or modules to refactor
- The quality principles to focus on (DRY, SOLID, LEAN, KISS, or all)
- Context about the codebase if working on unfamiliar areas

## Workflow

1. **Analyze** code
   - Read target TypeScript files
   - Identify violations of DRY, SOLID, LEAN, and KISS principles
   - Understand existing patterns in the codebase

2. **Plan** refactoring
   - Determine which principles to apply
   - Plan incremental changes to avoid breaking tests
   - Consider impact on related modules

3. **Apply** refactoring
   - Extract duplicated code into reusable functions/interfaces
   - Apply SOLID principles (single responsibility, open/closed, etc.)
   - Remove unnecessary abstractions and complexity (LEAN)
   - Simplify over-engineered code (KISS)
   - Ensure type safety without over-engineering

4. **Validate** changes
   - Run TypeScript compiler (`tsc`) to check for errors
   - Run linter to check for warnings
   - Run tests to ensure all tests pass
   - Verify refactoring improves code quality

## Examples

### Refactoring DRY violations

**Before:**
```typescript
function processUserA(user: User): Result<string> {
  if (user.isActive && user.hasPermission) {
    return { success: true, data: `User ${user.name} is valid` };
  } else {
    return { success: false, error: "Invalid user" };
  }
}

function processUserB(user: User): Result<string> {
  if (user.isActive && user.hasPermission) {
    return { success: true, data: `User ${user.name} is valid` };
  } else {
    return { success: false, error: "Invalid user" };
  }
}
```

**After:**
```typescript
function validateUser(user: User): Result<string> {
  if (user.isActive && user.hasPermission) {
    return { success: true, data: `User ${user.name} is valid` };
  } else {
    return { success: false, error: "Invalid user" };
  }
}

function processUserA(user: User): Result<string> {
  return validateUser(user);
}

function processUserB(user: User): Result<string> {
  return validateUser(user);
}
```

### Applying SOLID principles

**Before:** Single class with multiple responsibilities

**After:** Separate classes, each with a single responsibility, using interfaces for polymorphism

### Improving type safety

**Before:**
```typescript
function processValue(value: any): string {
  return value.toUpperCase();
}
```

**After:**
```typescript
function isString(value: unknown): value is string {
  return typeof value === "string";
}

function processValue(value: unknown): string {
  if (isString(value)) {
    return value.toUpperCase();
  }
  throw new Error("Value is not a string");
}
```

## Troubleshooting

- **Tests fail after refactoring:** Review changes incrementally, identify what broke tests, and adjust refactoring approach
- **TypeScript errors:** Address each error; they often indicate type safety issues that need to be resolved
- **Code becomes more complex:** Step back and reconsider if abstraction is necessary (LEAN principle)
- **Cannot eliminate duplication:** Consider if duplication is actually variation that requires different implementations

## Related resources

- Read [`.roo/rules-typescript-code-quality/2_best_practices.xml`](../rules-typescript-code-quality/2_best_practices.xml) for detailed TypeScript-specific practices
- Read [`.roo/rules-typescript-code-quality/3_common_patterns.xml`](../rules-typescript-code-quality/3_common_patterns.xml) for common refactoring patterns
- Read [`.roo/rules-typescript-code-quality/4_decision_guidance.xml`](../rules-typescript-code-quality/4_decision_guidance.xml) for decision-making criteria
