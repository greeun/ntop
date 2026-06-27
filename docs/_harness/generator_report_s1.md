# Sprint 1 Report — service-planning-harness (ntop)

## Strategic Decision

**REFINE** (first round — establishing the S1 baseline within the current direction).
There is no prior `critique.md` and no `REDIRECT`, so per the hard rules I REFINE within the
spec/playbook direction rather than pivot. Concrete edits made this round to maximize first-pass
quality:

1. Identified **10 real named competitors** (well past the ≥3 bar) and added the manual baseline
   (`lsof`/`ps|grep`/`kill`) so the "도구 없는 대체재" is explicit.
2. Built the **✓/△/✗ comparison matrix** with an extra "탐지 범위(scope)" column to make the
   pm2/systemd "deep control but narrow scope" nuance visible, and annotated every △ with its reason.
3. Drew the **positioning quadrant** on the exact two axes the prompt requested
   (런타임/프레임워크 의미 인지 × 프로세스 제어 깊이) and placed all tools.
4. Wrote the **full ecosystem analysis as role·incentive·value-exchange·dependency tables** across
   all 7 participant categories, plus an **ASCII node-edge ecosystem map** with labeled value flows.
5. Made differentiation an **explicit named comparison + a 3-part moat argument** (data-driven rule
   table × process-local-only detection × compounding extension structure) — no "more convenient".

## Deliverables produced (from sprint contract)

- `/Users/uni4love/project/workspace/211-withwiz/ntop/docs/_harness/research-s1.md` — the S1
  market·competition·ecosystem research (Korean). Self-proposed Sprint Contract at top, content,
  self-verification table at bottom.
- `/Users/uni4love/project/workspace/211-withwiz/ntop/docs/_harness/generator_report_s1.md` — this report.

## Verification I performed

Each Sprint Contract check verified against the actual file (concrete observed results):

| Check | Observed result | Where |
|---|---|---|
| C-1 실명 경쟁자 ≥3 | PASS — 10 named + baseline | §2 |
| C-2 비교 축(하는/안 하는) | PASS — per-tool profile + 8-axis table | §2-0/§2-1 |
| C-3 매트릭스(✓/△/✗) | PASS — 11 rows × 8 axes + scope col, ntop row `▶`, △ reasons given | §3 |
| C-4 포지셔닝 쿼드런트 | PASS — 2-axis ASCII scatter, all tools placed | §4 |
| C-5 시장/세그먼트 정성(매출 0) | PASS — 4 trends + funnel segments, no revenue sizing | §5 |
| C-6 생태계 참여자 全 | PASS — 7 categories, role/incentive/value/dependency | §6 |
| C-7 생태계 맵(렌더러 0) | PASS — ASCII node-edge, no Mermaid | §7 |
| C-8 갭/기회 + moat | PASS — strength/weakness tables + 3 moat arguments | §8/§9 |

Gate self-checks: **G-c** PASS (explicit named comparison: "RT·FW·PORT 동시 ✓는 ntop뿐"; no
generalities). **G-g** PASS (matrix + quadrant + ecosystem map + segment funnel all ASCII, zero
external renderers, no empty diagrams).

Fact-grounding verification: I re-read `Cargo.toml` and `src/process/framework_rules.rs` against the
dossier — dependency versions (clap4/crossterm0.28/ratatui0.29/sysinfo0.33/tokio1/nix0.29/
windows-sys0.59) and the registered FRAMEWORK_RULES/RUNTIME_RULES match the dossier exactly. Every
ntop claim in the doc carries a dossier §-reference. No payment/settlement/DB/REST invented; the only
monetization mention (GitHub Sponsors) is labeled "가설" with no sizing.

## Known limitations

- Some competitor sub-features vary by version (glances ports plugin, btop Windows maturity, gotop
  tree/grouping). I judged these **conservatively** (toward ✗/△) per the prompt's instruction; an
  Evaluator wanting stricter values can have them adjusted.
- Scope is S1 only (market/competition/ecosystem). Persona specificity is S2; feature↔problem
  traceability is S4's `10-prd`. The ecosystem/competitor analysis here is the substrate that
  `02-market-competition.md` will formalize in S4.
- No image-generation tier used; visuals are tier-③ ASCII (md always carries this tier per 4-V).

## How to review

- **G-c (named differentiation):** read §3 matrix → confirm RT·FW·PORT columns are ✗ for all general
  monitors and that only the ntop row has all three ✓; then §9 for the moat argument (must NOT be
  "more convenient"). 
- **G-g (infographic-first):** confirm §3 matrix, §4 quadrant, §7 ecosystem map, §5 funnel are all
  present, non-empty, ASCII, and contain no Mermaid/external-renderer syntax.
- **≥3 competitors + axes:** §2 lists 10 real tools each with "한다/안 한다"; §2-0 defines the axes.
- **Ecosystem completeness:** §6 covers supply/demand/platform/complements/substitutes/institutions/
  sponsorship-distribution; cross-check §7 map shows code·feedback·sponsorship·distribution·dependency
  flows.
- **No fabrication:** grep for payment/settlement/take-rate/DB/REST terms → should be absent; the
  single "가설" monetization note is in §6-7.

READY_FOR_QA: generator_report_s1.md
