# Sprint 3 Report — service-planning-harness (ntop)

## Strategic Decision

**REFINE** (first round of S3 within the current direction).
There is no S3 `critique.md` and no `REDIRECT`, so per the hard rules I REFINE within the
spec/playbook direction rather than pivot. S3 is the heaviest research sprint (foundation for
12-ia / 13-user-flows / 20-wireframes / 21-screen-spec). Concrete work this round, built directly on
research-s1 (semantic-gap thesis) and research-s2 (PRB-1~8 + personas P1/P2/P3):

1. Built the **TUI+CLI IA tree** (§2) as one ASCII tree with a code-grounded color/role legend
   (`status_bar.rs`/`empty_state.rs` `Color::*`), every node carrying a screen ID + entry key + PRB.
2. Drew **6 core user flows** (§3) as ASCII flowcharts — F1 startup→2-pass scan→select→4-tab
   inspect→quit, F2 filter/sort/Node-only, F3 single kill graceful→force, F4 tree kill, F5
   SignalPicker, F6 CLI `list --json`→`kill --tree` — each step labeled with screen ID·key·PRB.
3. Produced the **kill graceful→force state machine + sequence diagram** (§4), modeling
   `GracefulResult` (Terminated/TimedOut/AlreadyDead/PermissionDenied) and the TUI(DLG-FORCE prompt)
   vs CLI(auto) escalation difference.
4. Enumerated the **full screen inventory** (§5, 21 rows: 16 TUI + 5 CLI) with IDs, widget files,
   entry keys, PRB links — the canonical screen list for `21-screen-spec`.
5. Mapped **4 states (empty/loading/normal/error)** per key view (§6), documenting the verified fact
   that ntop **merges empty and loading into one `empty_state` widget** (spinner `tick_count`), with
   the ★ error triggers from the playbook (EPERM→PermissionDenied, lsof/netstat absent→empty result).
6. Copied the **key-binding catalog** faithfully (§7, 5 groups) from dossier §6 **+ verified code**
   (ui.rs line refs, the context-sensitive bottom bar from `status_bar.rs`) — single source for `21`.
7. Drew **12 low-fi ASCII wireframes** (§8) — main view, 4 detail tabs, 5 dialogs, filter mode, empty
   state — under an explicit `HUMAN_CHECKPOINT_REQUIRED` banner (P-1).
8. Wrote the **design-quality note** (§9, 4-D): confirmed no naive defect (no manual mode/role wall —
   `n` is a toggle; no account-derived re-input — ports/runtime auto-resolved; no stacked steps —
   live filter), with 3 honest boundary-case flags.

**Key finding surfaced (not a pivot):** dossier §6 says the main layout is left/right
(`process_list 좌 55% | detail_panel 우 45%`), but the **actual code `src/tui/ui.rs` L37–44 uses
`Layout::vertical([55%,45%])` = top/bottom**. Since spec line 5 / playbook line 8 authorize
"codebase-facts.md **+ 실제 소스코드**" as co-equal fact sources and code is ground truth, I adopted
top/bottom and flagged the dossier discrepancy for the human checkpoint (research-s3 §0 note + §10).

## Deliverables produced (from sprint contract)

- `/Users/uni4love/project/workspace/211-withwiz/ntop/docs/_harness/research-s3.md` — the S3
  UX/UI flow·screen research (Korean). Self-proposed Sprint Contract at top, content (§1–§10),
  self-verification table at bottom.
- `/Users/uni4love/project/workspace/211-withwiz/ntop/docs/_harness/generator_report_s3.md` — this report.

## Verification I performed

Each Sprint Contract check verified against the actual file (concrete observed results):

| Check | Observed result | Where |
|---|---|---|
| E-1 IA tree (TUI+CLI, role/color) | PASS — color/role legend (code Color refs) + unified ASCII tree with screen IDs/entry keys/PRB | §2 |
| E-2 6 flows + screen/key/PRB links | PASS — F1~F6 all ASCII; every step labeled screen ID·key·PRB; branches `◇` | §3 |
| E-3 kill graceful→force seq + state | PASS — state machine (Terminated/TimedOut/EPERM/AlreadyDead) + sequence (User↔App↔Killer↔OS↔Process); TUI/CLI escalation difference noted | §4 |
| E-4 screen inventory (all + empty/error) | PASS — 21-row table (16 TUI + 5 CLI), ID·role·widget file·entry·PRB | §5 |
| E-5 key views 4 states | PASS — matrix (V-LIST/NET/LOG/ENV/KILL), ★EPERM·★lsof/netstat triggers, empty=loading merge fact, transition diagram | §6 |
| E-6 key-binding catalog (5 groups) | PASS — global/list/detail/filter/dialog tables + context-sensitive bottom bar (status_bar.rs) | §7 |
| E-7 wireframes 1/screen → HUMAN_CHECKPOINT | PASS — 12 ASCII frames; `HUMAN_CHECKPOINT_REQUIRED` banner + 5 review points | §8 |
| E-8 design-quality note (4-D) | PASS — 4-D check table (all ✅) + zero naive defects + 3 boundary flags | §9 |

Gate self-checks: **G-g** PASS (IA tree, 6 flowcharts, state machine, sequence, 4-state matrix +
transition, 12 wireframes — all pure ASCII; zero Mermaid/external renderers; no empty diagrams; every
screen carries a visual). **P-1** PASS (HUMAN_CHECKPOINT_REQUIRED banner on §8 + review points).
**C1 (flow↔problem consistency)** PASS (every flow step links a PRB from research-s2 §3).

Fact-grounding: every screen/key/flow carries a dossier §-ref **or** a file:line (ui.rs
L19/L37/L239/L245/L275/L281/L286/L290; status_bar.rs; empty_state.rs; process_list.rs;
detail_panel.rs), cross-referenced with research-s2 PRB-1~8. I independently re-read the source
(`status_bar.rs`, `process_list.rs`, `ui.rs`, `empty_state.rs`, `detail_panel.rs`) to confirm: top
bar string, list columns ([✓]PID·NAME·PORT·THR·CPU·MEM·USER·STATUS·UPTIME — note: the TUI list has no
dedicated FRAMEWORK column; framework lives in TAB-INFO and the CLI `list` table), the context-
sensitive bottom-bar hint sets, the empty/loading merge, and the vertical layout split. No
payment/settlement/DB/REST/login/settings-wizard screens invented; only implemented TUI/CLI surfaces
are listed.

## Known limitations

- **Layout discrepancy (most important):** dossier §6 (left/right) vs code (top/bottom vertical,
  ui.rs L37–44). I adopted the code (top/bottom) per the "dossier + source code" fact mandate and
  flagged it in research-s3 §0 + §10 for the P-1 human checkpoint. If the Evaluator insists on the
  dossier's left/right wording, this is a one-line re-orientation of the wireframes — but the
  code-faithful choice is the defensible one for "ASCII layout IS the real fidelity."
- **TUI list columns differ from the s2/spec prose:** research-s2 and spec §4-2 mention a "NAME/
  FRAMEWORK column" in the left list; the actual TUI list (process_list.rs) has no FRAMEWORK column
  (framework is shown in TAB-INFO and in the CLI `list` table). research-s3 documents the code-true
  column set and notes where framework actually surfaces. Flag for S4 `21`/`32` consistency.
- **Wireframe width/alignment** is low-fidelity (≈78-col example); real rendering is terminal-width
  elastic (ratatui truncates/grows). Visual quality is P-1 review scope (excluded from scoring).
- **CLI output schemas** (table columns / JSON fields+depth / CSV 9 columns) are referenced here but
  fully specified in S4 `32-api-spec`; this sprint covers them at the IA/flow level only.
- Scope is S3 only (IA·flows·screens·keys·wireframes). Canonical feature IDs + feature↔problem
  traceability table are S4 `10-prd`.

## How to review

- **E-1 / IA (§2):** confirm the tree covers BAR-TOP/BOTTOM, V-MAIN, V-LIST, V-DETAIL + 4 tabs,
  V-FILTER, V-EMPTY, 5 dialogs, and the 5 CLI subcommands, each with a role and color legend.
- **E-2 / flows (§3):** verify all 6 required flows exist, each step labeled screen ID·key·PRB; check
  F3 shows the graceful→force escalation branch and F4 shows reverse-order tree kill.
- **E-3 / kill session (§4):** confirm both a state machine AND a sequence diagram, with the TUI
  (DLG-FORCE prompt) vs CLI (auto-escalate) distinction and the GracefulResult enum branches.
- **E-4 / inventory (§5):** count screens (16 TUI + 5 CLI); confirm empty (V-EMPTY) and error states
  are present and that no non-existent screen is invented.
- **E-5 / 4 states (§6):** confirm the empty=loading merge is documented and the ★ EPERM / ★ lsof-
  netstat-absent error triggers are mapped to specific views (DLG-KILL/FORCE, TAB-NET).
- **E-6 / keys (§7):** spot-check against dossier §6 — global (q/Ctrl+C, /, s, r, +/-, x, K, H, S, e,
  n), list (↑↓/jk, PgUp/Dn, Home/End, Enter, →/l, ←/h, Space, Tab, Esc), detail (Tab/→/l next,
  BackTab/←/h prev, scroll), filter (Enter apply / Esc clear), dialogs per §7-5.
- **E-7 / wireframes (§8):** confirm 12 ASCII frames + the `HUMAN_CHECKPOINT_REQUIRED` banner; this
  is a STEP 7 human-review trigger (visual quality not machine-scored).
- **E-8 / design quality (§9):** confirm the 4-D table (no manual mode wall, no account-derived
  re-input, no stacked steps) and that the 3 flagged items are boundary cases, not defects.
- **G-g:** grep the file for Mermaid/```mermaid → should be absent; confirm all diagrams are non-empty ASCII.
- **No fabrication:** grep for payment/settlement/take-rate/DB/REST/로그인/회원/요금 → should be absent.
- **Layout note:** read §0 and §10 — confirm the dossier-vs-code discrepancy is flagged transparently
  rather than silently resolved.

READY_FOR_QA: generator_report_s3.md
</content>
