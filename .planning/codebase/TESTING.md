# Testing Patterns

**Analysis Date:** 2026-03-18

## Test Framework

**Runner:**
- No frontend test framework configured
- No Vitest, Jest, or other test runner in `package.json` or config files
- Rust: built-in `cargo test` with `#[cfg(test)]` modules

**Run Commands:**
```bash
# Frontend — no test commands exist
# package.json scripts: "dev", "build", "preview", "tauri"

# Backend (Rust)
cd src-tauri && cargo test          # Run all Rust tests
cd src-tauri && cargo test -- --nocapture  # With stdout
```

## Test File Organization

**Frontend:**
- No test files found in the codebase
- No `__tests__/`, `test/`, or `*.test.*` / `*.spec.*` files

**Backend (Rust):**
- Inline `#[cfg(test)]` modules within source files
- Tests in `collection.rs`, `evaluation.rs`, `app_config.rs`

## Test Structure

**Rust Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // arrange
        let input = create_test_input();

        // act
        let result = function_under_test(input);

        // assert
        assert_eq!(result, expected);
    }
}
```

## Mocking

**Frontend:**
- N/A — no mocking setup

**Backend (Rust):**
- In-memory SQLite for database tests
- No external mocking framework

## Coverage

**Requirements:**
- No coverage tool or targets configured
- No CI enforcement

## Test Types

**Unit Tests (Rust):**
- Inline tests for core logic (collection, evaluation, config)
- In-memory SQLite for data layer tests

**Integration Tests:**
- None present

**E2E / Component Tests:**
- None present
- Manual testing via `hooks/smoke_test.sh`
- `MANUAL_TEST_CHECKLIST.md` referenced for critical paths

## Recommendations

To introduce frontend testing:
1. Add Vitest (Vite-native) as test runner
2. Add `"test": "vitest"` to `package.json`
3. Place tests alongside source (e.g., `utils.test.ts`) or in `__tests__/`
4. Mock `@tauri-apps/api` and `fetch` for API/hook tests
5. Use React Testing Library for component tests

---
*Testing analysis: 2026-03-18*
*Update when test patterns change*
