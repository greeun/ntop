# Sprint 2 Report — service-planning-harness (ntop)

## Strategic Decision

**REFINE** (first round of S2 within the current direction).
There is no S2 `critique.md` and no `REDIRECT`, so per the hard rules I REFINE within the
spec/playbook direction rather than pivot. Concrete edits made this round to maximize first-pass
quality, building directly on `research-s1.md`:

1. Stated the **core problem as the semantic gap** (§1) with an explicit who/what/why + an ASCII
   "what the monitor sees vs what I need to know" contrast, then decomposed it into **6 sub-problems
   (§2-1~2-6)** each written as 누가/무엇이/왜 + **빈도 + 결과** + the exact ntop behavior (dossier §)
   that fills it.
2. Built a **problem anchor table PRB-1~8 (§3)** with IDs, primary persona, the filling ntop behavior,
   and a provisional `F-*` feature handle so S4's `10-prd` has a ready G-a trace start point.
3. Wrote **3 personas (§4)**; made **P1 (폴리글랏 풀스택) and P3 (npx/MCP heavy)** fully concrete
   (name·role·situation·day-in-the-life·pains-with-frequency/consequence·workaround·desired outcome) —
   exceeding the "≥1 concrete" bar — and added **P2 (operator)** for the headless/automation axis.
4. Mapped **each persona's desired outcome → ntop's 3 capability groups (identify/port/safe-kill)**
   in §5, every row carrying a PRB and dossier §.
5. Produced the required visuals (G-g, all ASCII, zero external renderer): **persona journey map**
   with per-stage action/emotion (§6-1), **problem→persona matrix** (§6-2), and a **pain-frequency
   chart** (§6-3).

## Deliverables produced (from sprint contract)

- `/Users/uni4love/project/workspace/211-withwiz/ntop/docs/_harness/research-s2.md` — the S2
  problem·target·persona research (Korean). Self-proposed Sprint Contract at top, content,
  self-verification table at bottom.
- `/Users/uni4love/project/workspace/211-withwiz/ntop/docs/_harness/generator_report_s2.md` — this report.

## Verification I performed

Each Sprint Contract check verified against the actual file (concrete observed results):

| Check | Observed result | Where |
|---|---|---|
| D-1 who-what-why + sub-problems(freq/consequence) | PASS — §1 core (who/what/why) + 6 sub-problems each as 누가/무엇이/왜+빈도+결과+ntop동작 | §1, §2 |
| D-2 personas ≥2, ≥1 concrete | PASS — 3 personas; P1·P3 fully concrete (day-in-the-life, freq/consequence, workaround, desired outcome); no demographics-only | §4 |
| D-3 desired outcome → ntop capability | PASS — mapping diagram + 3-capability summary, each row has PRB + dossier § | §5 |
| D-4 problem list (anchor) PRB-IDs | PASS — PRB-1~8 table with primary persona + ntop behavior + provisional F-handle | §3 |
| D-5 persona journey map (action/emotion) | PASS — 띄움→모름→포트역추적/오kill→(대안)→ntop 식별→안전 종료, per-stage action/touchpoint/emotion/pain/ntop | §6-1 |
| D-6 problem→persona matrix + pain-frequency | PASS — 8×3 matrix (●/◐/○) + P1 pain-frequency bar chart | §6-2, §6-3 |

Gate self-checks: **C1** PASS (semantic gap defined sharply, each sub-problem has frequency +
consequence). **G-a** PASS (PRB-1~8 carry IDs + primary persona + ntop behavior + provisional feature
handle → S4 `10-prd` can trace back). **G-b** PASS (P1·P3 concrete to situation/day/frequency/
consequence; abstract-demographics-only failure mode avoided). **G-g** PASS (gap diagram, journey map,
matrix, frequency chart, mapping diagram all ASCII; zero Mermaid/external renderers; no empty diagrams).

Fact-grounding verification: every ntop behavior cited carries a dossier §-reference
(§2 detection rules / §3 ProcessInfo·Config / §4-1 NetworkInspector / §4-2 phys_footprint / §5 killer /
§6 keybindings·TUI / §7 CLI), cross-referenced with research-s1.md (§3 matrix / §5 segments / §9 moat).
No payment/settlement/take-rate/DB/REST invented; personas are users (developers/operators), never
paying customers, per dossier §0-1. Persona frequencies are labeled as plausible usage-context
estimates (not in the dossier), while every cited ntop capability is dossier-grounded.

## Known limitations

- Persona-level numbers (주 10회, 월 1~2회) are **usage-context plausibility estimates**, not dossier
  facts — they characterize the frequency of the problem ntop targets. The Evaluator can request
  stronger "추정" labeling. All ntop capability claims remain dossier-grounded.
- Provisional feature handles (`F-분류`, etc.) are anchors that **S4 `10-prd` will normalize into real
  feature IDs**; no feature is invented here — each maps to an actual behavior in dossier §2~§7.
- Scope is S2 only (problem/target/persona). User flows/screens are S3; the completed feature↔problem
  traceability table is S4 `10-prd`. The PRB↔F-handle map in §3/§5 is its substrate.

## How to review

- **C1 / D-1 (problem clarity):** read §1 (semantic-gap one-liner with who/what/why) → §2 confirm all
  6 sub-problems carry 누가/무엇이/왜 + 빈도 + 결과; no vague problem statements.
- **G-b / D-2 (target specificity):** read §4 P1 and P3 — confirm each has situation, a day-in-the-life
  moment, pains with explicit frequency + consequence, current workaround, desired outcome. A
  demographics-only persona would FAIL; verify these are not that.
- **G-a / D-4 (trace start point):** read §3 PRB-1~8 — confirm each problem has an ID, a primary
  persona, the ntop behavior (dossier §), and a provisional feature handle that S4 can trace back to.
- **D-3 (outcome→capability):** read §5 — confirm every persona desired outcome maps to identify/port/
  safe-kill with a PRB and dossier §.
- **G-g (infographic-first):** confirm §1 gap diagram, §6-1 journey map, §6-2 matrix, §6-3 frequency
  chart, §5 mapping diagram are present, non-empty, ASCII, and contain no Mermaid/external-renderer
  syntax.
- **No fabrication:** grep for payment/settlement/take-rate/DB/REST/요금/구매 → should be absent;
  personas framed as users not customers.

READY_FOR_QA: generator_report_s2.md
