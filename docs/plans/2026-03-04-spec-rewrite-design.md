# Spec Rewrite Design: Collab Platform + Language Core

**Purpose:** Full rewrite of `tests/spec/` — replacing the current 140 mechanical spec files with a cohesive, expressive test suite that also serves as the **initial living specification of the Opal language**.

---

## Two-Layer Architecture

### Layer 1: `tests/spec/core/` — The Language Core (Self-Hosting)

This folder is Opal's **self-description**: the language written in itself, as an executable specification. The goal is maximum self-hosting — using Opal's own features (macros, AST quoting, protocols, annotations, models) to describe and implement as much of Opal as possible.

This is the canonical reference for what Opal *is*:

```
tests/spec/core/
├── types.opl          # Type system: BuiltinType enum, TypeInfo, typeof, is
├── dispatch.opl       # Multiple dispatch rules written as Opal specs
├── patterns.opl       # Pattern algebra: constructing and matching patterns
├── protocols.opl      # Protocol definition and satisfaction checking
├── macros.opl         # Macro system: quoting, splicing, AST transformation
├── meta_eval.opl      # Mini Opal evaluator using ast blocks and the eval builtin
├── annotations.opl    # Annotation system: metadata on declarations
└── grammar.opl        # BNF-like grammar expressed as Opal data structures
```

### Layer 2: `tests/spec/01-10/` — The Collab Platform (Domain Examples)

A **collaboration platform** — teams, projects, bounties, and accomplishments. People form teams, post bounties for help on projects, contributors with matching skills claim bounties, and earn accomplishments ranked by tier.

Feature folders (01-09) are **excerpts from the platform domain** — each file tells a small, realistic story from the collab world that happens to exercise a specific language feature. The examples folder (10) is the **full multi-file DDD application**.

---

## Domain Model

### Entities
- **Contributor** — name, skills (list of `SkillTag` symbols), accomplishments, reputation score
- **Bounty** — description, reward amount, required skill, state (`Available | Claimed | Paid`)
- **Project** — title, status (`Draft | Open | InProgress | Completed`), team, bounty pool
- **Team** — set of contributors working toward a project
- **Accomplishment** — earned on completing bounties; tier based on reward (`Bronze | Silver | Gold`)

### Business Rules
- Bounties can only be claimed by contributors with the required `SkillTag`
- Reward tiers: `< 100` → Bronze, `100..500` → Silver, `>= 500` → Gold
- Project must be `:Open` or `:InProgress` to accept claims
- Completing all bounties transitions project to `:Completed`
- Total pool = sum of all `Available` bounty rewards

### Type Aliases and Enums
```opal
type SkillTag = :rust | :opal | :design | :docs | :testing
enum AwardTier
  Bronze
  Silver
  Gold
end
enum Status
  Draft
  Open
  InProgress
  Completed
end
enum BountyState
  Available(reward: Float)
  Claimed(reward: Float, claimer: String)
  Paid(reward: Float, claimer: String)
end
```

---

## Feature Folder Mapping (01-09)

Each folder contains 3-5 spec files. Each spec is a mini-scenario from the collab domain.

### `01-basics/`
- `contributor_profile.opl` — f-strings, let bindings, variables
- `bounty_amounts.opl` — arithmetic, compound assign, casting
- `skill_tags.opl` — symbols, type aliases, `in`/`not in`
- `reward_formatting.opl` — f-string format specs, null-safe `?.`/`??`
- `project_metadata.opl` — parallel assign, string methods

### `02-control-flow/`
- `status_transitions.opl` — match on Status enum, or-patterns
- `bounty_eligibility.opl` — suffix if, match guards
- `claim_loop.opl` — for loops with break/next
- `tier_classification.opl` — range patterns, if-expression assignment
- `pipeline_control.opl` — pattern match chains with as-bindings

### `03-collections/`
- `team_roster.opl` — sort, filter, group_by contributors by skill
- `bounty_board.opl` — comprehensions over available bounties
- `leaderboard.opl` — reduce/sort/join for ranking
- `skill_index.opl` — dict building, merge, group_by

### `05-functions/` (renaming 02-functions → 05-functions for clarity)
- `claim_dispatch.opl` — multiple dispatch by SkillTag
- `award_factory.opl` — default params, closures
- `bounty_pipeline.opl` — pipe operator, function composition
- `validation_guards.opl` — precondition dispatch

### `04-classes/`
- `bounty_enum.opl` — BountyState enum with methods
- `contributor_class.opl` — Contributor with protocols
- `project_model.opl` — model keyword with validation
- `team_protocol.opl` — Rankable/Displayable protocols
- `operator_money.opl` — operator overloading on reward amounts

### `05-errors/`
- `claim_result.opl` — Ok/Error from claim_bounty
- `project_validation.opl` — requires, try/catch, error propagation

### `06-actors/`
- `bounty_processor.opl` — actor processing claim queue

### `07-macros/`
- `platform_dsl.opl` — DSL macro for defining bounty requirements
- `audit_annotations.opl` — @[audit] annotation on claim functions

### `09-modules/`
- Multi-file import examples using collab modules

---

## Main App: `tests/spec/10-examples/collab/`

```
10-examples/collab/
├── core/
│   ├── types.opl          # Shared enums: Status, AwardTier, BountyState, SkillTag
│   └── money.opl          # Money type with validation and operator overloading
├── domain/
│   ├── contributor.opl    # Contributor model (validated), accomplishments
│   ├── bounty.opl         # Bounty class + claim logic
│   ├── project.opl        # Project with state machine
│   └── team.opl           # Team aggregation
├── services/
│   ├── claim_service.opl  # Orchestrates bounty claiming and award granting
│   ├── ranking_service.opl # Leaderboard: sort contributors by score
│   └── report_service.opl # Format and output platform reports
└── main.opl               # Full scenario: create project -> claim bounties -> rank
```

### `main.opl` Scenario Narrative
1. Create a project "Build Opal Compiler" in `:Open` status with 4 bounties
2. Register 3 contributors: Alice (rust, opal), Bob (design, docs), Carol (testing)
3. Each contributor claims the matching bounty
4. Accomplishments are auto-granted based on reward tier
5. Project auto-transitions to `:Completed` when all bounties paid
6. Print final leaderboard ranked by total reward earned

Expected output:
```
=== Collab Platform Report ===
Project: Build Opal Compiler [completed]
Bounties: 4/4 claimed

Leaderboard:
1. Alice  Gold    $750.00  [Parser Bounty, Core Types]
2. Carol  Silver  $250.00  [Test Suite]
3. Bob    Bronze  $50.00   [Documentation]

Total distributed: $1050.00
```

---

## Quality Bar for All Specs

Every spec file should:
1. Have a comment block explaining the scenario (2-3 lines)
2. Use realistic names and values — not `x = 1`, `r1`, `print("ok")`
3. Exercise at least 2-3 language features together
4. Be readable top-to-bottom like a short story
5. Have a meaningful expected output (not just `true | false | 42`)

---

## Self-Hosting Goals (core/)

Maximum self-hosting priority order:
1. **Type algebra** — `TypeInfo` as an enum, `typeof` and `is` written as Opal functions over Opal values
2. **Pattern matching specification** — The pattern algebra described as Opal data structures
3. **Dispatch table** — Multiple dispatch resolution expressed as Opal match expressions
4. **Macro transformer** — AST transformation pipeline written as Opal closures over AST values (using ast blocks)
5. **Mini evaluator** — A small subset evaluator using Opal's ast quoting as a primitive
6. **Grammar as data** — BNF rules expressed as Opal dicts/lists (not a full parser, but the structure)
