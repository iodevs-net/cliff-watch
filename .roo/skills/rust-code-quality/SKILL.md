---
name: rust-code-quality
description: Refactor Rust code to follow DRY, SOLID, LEAN, and KISS principles. Use when working with Rust code in cliff-watch crates, refactoring existing modules, or reviewing code quality.
---

# Rust Code Quality

## When to use this skill

Use this skill when:
- Refactoring existing Rust code to eliminate duplication (DRY)
- Applying SOLID principles to Rust modules and structs
- Removing unnecessary complexity and waste (LEAN)
- Simplifying over-engineered code (KISS)
- Writing new Rust code with quality principles in mind
- Reviewing Rust code for adherence to quality principles

## When NOT to use this skill

Do NOT use this skill when:
- Working with TypeScript code (use `typescript-code-quality` skill instead)
- Making architectural decisions (use `architecture-refactor` skill instead)
- Debugging runtime issues (use Debug mode instead)
- The task requires system-level modifications

## Inputs required from the user

The user should provide:
- The specific Rust files or modules to refactor
- The quality principles to focus on (DRY, SOLID, LEAN, KISS, or all)
- Context about the codebase if working on unfamiliar areas

## Workflow

1. **Analyze the code**
   - Read the target Rust files
   - Identify violations of DRY, SOLID, LEAN, and KISS principles
   - Understand the existing patterns in the codebase

2. **Plan the refactoring**
   - Determine which principles to apply
   - Plan incremental changes to avoid breaking tests
   - Consider the impact on related modules

3. **Apply the refactoring**
   - Extract duplicated code into reusable functions/traits
   - Apply SOLID principles (single responsibility, open/closed, etc.)
   - Remove unnecessary abstractions and complexity (LEAN)
   - Simplify over-engineered code (KISS)
   - Ensure type safety without over-engineering

4. **Validate the changes**
   - Run `cargo clippy` to check for warnings
   - Run `cargo fmt` to format the code
   - Run `cargo test` to ensure all tests pass
   - Verify the refactoring improves code quality

## Examples

### Refactoring DRY violations

**Before:**
```rust
fn process_user_a(user: &User) -> Result<String> {
    if user.is_active() && user.has_permission() {
        Ok(format!("User {} is valid", user.name))
    } else {
        Err("Invalid user".into())
    }
}

fn process_user_b(user: &User) -> Result<String> {
    if user.is_active() && user.has_permission() {
        Ok(format!("User {} is valid", user.name))
    } else {
        Err("Invalid user".into())
    }
}
```

**After:**
```rust
fn validate_user(user: &User) -> Result<String> {
    if user.is_active() && user.has_permission() {
        Ok(format!("User {} is valid", user.name))
    } else {
        Err("Invalid user".into())
    }
}

fn process_user_a(user: &User) -> Result<String> {
    validate_user(user)
}

fn process_user_b(user: &User) -> Result<String> {
    validate_user(user)
}
```

### Applying SOLID principles

**Before:** Single struct with multiple responsibilities

**After:** Separate structs, each with a single responsibility, using traits for polymorphism

## Troubleshooting

- **Tests fail after refactoring:** Review the changes incrementally, identify what broke the tests, and adjust the refactoring approach
- **Clippy warnings:** Address each warning; they often indicate potential bugs or non-idiomatic Rust code
- **Code becomes more complex:** Step back and reconsider if the abstraction is necessary (LEAN principle)
- **Cannot eliminate duplication:** Consider if the duplication is actually variation that requires different implementations

## Related resources

- Read [`.roo/rules-rust-code-quality/2_best_practices.xml`](../rules-rust-code-quality/2_best_practices.xml) for detailed Rust-specific practices
- Read [`.roo/rules-rust-code-quality/3_common_patterns.xml`](../rules-rust-code-quality/3_common_patterns.xml) for common refactoring patterns
- Read [`.roo/rules-rust-code-quality/4_decision_guidance.xml`](../rules-rust-code-quality/4_decision_guidance.xml) for decision-making criteria
