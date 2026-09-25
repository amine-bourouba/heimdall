# ADR 0008: Provider abstraction removal

- **Status:** Accepted (user answer 3)
- **Date:** 2026-09-25

## Context

`Protocols/ProviderDescribing.swift` has exactly one implementation, `Services/Providers/ClaudeCodeProvider.swift` (plus a test mock). It mixes SF Symbol names, a SwiftUI `Color`, placeholder capability counts and a macOS CLI path. Because the one implementation enables every capability, it never hides anything. The user wants to focus on Claude Code now and consider other CLIs later.

## Decision

1. Heimdall hard-codes Claude Code as the only provider. There is no provider trait in Rust and no provider store in Vue.
2. What the provider supplied moves to where it is used:
   - Navigation items, order and labels (`Models/Provider/NavigationItem.swift`) → `src/router/routes.ts` (M2-06), with `NavigationItemTests` carried over as a Vitest spec.
   - The model catalog → `heimdall-core/src/models/catalog/` (M1-10).
   - The CLI path → PATH search (M1-31) and the `claudeCodePath` preference (ADR 0007).
3. `ClaudeCodeProviderTests` is dropped; `ProviderDescribingMock` goes with it.
4. **Reintroduction trigger:** when a second CLI provider is actually in scope, a new ADR designs the abstraction from two real implementations.

## Consequences

- Less code and no dead indirection.
- Supporting a second CLI later means touching the loaders and navigation, which a new ADR will scope.

## Related

Parity matrix row X2 · ADR 0002 · Tasks M0-06, M1-10, M1-31, M2-06
