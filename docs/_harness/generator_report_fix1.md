# Fix Round 1 Report — Blocking #1 (kill→force escalation 역전파)

## Strategic Decision

**REFINE** — `30`/`32`/`33`/`40`/`41`의 **코드-진실 kill 모델을 정본**으로 삼아, 모순 문서 4종(`11`/`13`/`20`/`21`)을
정렬하고 Non-Blocking 2건을 동반 정리했다. 새 사실/수치/기능 창작 0, 기존 ID(F·V/TAB/DLG/BAR/CLI·PRB) 보존.

코드-진실(직접 확인): TUI `KillConfirm`(Enter) = `send_signal(pid, SIGTERM)` **1회 직접** + `needs_rescan=true`
(폴링·자동 force 없음, `ui.rs` L95-108) / `kill_in_progress`는 선언·`None` 초기화뿐 **미대입 죽은 필드**(grep
`kill_in_progress =` 0건) / `ForceKillPrompt`는 `app.dialog=Some(...)`로 **설정되는 경로 없음**(KillConfirm·
KillTreeConfirm·Help·SignalPicker 4종만 설정) → 미도달 / `graceful_kill`(폴링+자동 SIGKILL)은 `main.rs` L418
**CLI 단건 경로에서만** 호출.

5개 구체 편집:
1. **`11-user-stories` US-2.1** — 수용 기준에서 "200ms 폴링 / timeout 후 TUI ForceKillPrompt(DLG-FORCE)" 서술
   제거. `x`→KillConfirm→`Enter` = **SIGTERM 1회 직접 전송 + needs_rescan**으로 교정, 강제 종료 자동 승격은
   **CLI `ntop kill` 단건 전용**임을 참고 블록으로 명시. (추가로 §1 "수용 기준 예시" 줄·Epic 트리 라벨 정정.)
2. **`13-user-flows`** — UF-3 흐름도의 "200ms 폴링→graceful_timeout→DLG-FORCE 자동" 분기를 제거하고 TUI=SIGTERM
   1회로 교정, 자동 force는 CLI 경로로 분리. §8 인트로 / §8-1 상태기계 / §8-2 시퀀스를 **경로 ①(TUI)·②(CLI)
   2-lane**으로 재작도해 `30` §8-3·`33` §2-3와 일치. 흐름 인덱스 표·플로우 맵·추적 표(2행)·L-5 게이트 노트 정정.
3. **`20-wireframes` W-9** — DLG-FORCE 헤더 "진입: graceful_timeout(10s) 초과 시 자동" → "(현재 코드에서 미도달
   — 어떤 키/경로로도 트리거되지 않음)"으로 정정, 푸터에 죽은 코드·CLI 전용 자동 승격 명시.
4. **`21-screen-spec` §3-12·§3-15** — §3-12에서 "kill_in_progress 기록 / 폴링" 제거(SIGTERM 1회·즉시 닫힘으로
   교정, `App.kill_in_progress`는 미사용 필드로 표기). §3-15 헤더/상태/예외에 **트리거 경로 현재 없음(미도달)**
   명시. §2-5 키 카탈로그의 DLG-FORCE 핸들러 행은 **유지**(코드에 존재). §3-16 직전 4상태 매트릭스 행 정정.
5. **`40-backlog`** — §3-3 신설 + **M1**("ForceKillPrompt/kill_in_progress 미도달 죽은 코드 — 연결 또는 제거"),
   **M2**("`12`/`13` NAME 인라인 배지 표기 보강") 추가. 카운트 문장 2곳을 "기능/수정 로드맵 10개 + 정합 항목
   M1·M2 별도"로 정정. **(Non-Blocking #2)** `12-ia` §0·`13-user-flows` §0 정정 박스에 "목록 NAME 컬럼이
   인라인 배지(`(Next.js)`/`[Python]`)로 프레임워크 노출" 한 줄 보강.

## Deliverables produced

- **`docs/11-user-stories.md`** — §1 수용 기준 예시 줄(L45-46), Epic 트리 라벨(US-2.1), **US-2.1 수용 기준 본문**.
- **`docs/13-user-flows.md`** — §0 코드-진실 박스(NAME 배지), UF 인덱스 표(UF-3 행), 플로우 맵 라벨, **UF-3
  §4(헤더·플로우차트·TUI/CLI 차이)**, §8 인트로, **§8-1 상태기계 재작도**, **§8-2 시퀀스 재작도**, 안전성 프로즈,
  추적 표(UF-3 2행), L-5 게이트 노트.
- **`docs/20-wireframes.md`** — **W-9 DLG-FORCE** 헤더 + 푸터.
- **`docs/21-screen-spec.md`** — **§3-12 DLG-KILL**(상호작용·상태·연결화면·연결데이터), **§3-15 DLG-FORCE**
  (헤더·목적·상호작용·상태·예외·연결데이터), §3-16 직전 4상태 매트릭스 행. (§2-5 키 카탈로그 DLG-FORCE 행 유지.)
- **`docs/40-backlog.md`** — §0·§3-2 카운트 문장 2곳, **§3-3 신설(M1·M2 표 + 설명)**.
- **`docs/12-ia.md`** — §0 정정 박스 하단 NAME 인라인 배지 정정 블록.

## Verification I performed

- **코드 진실 직접 확인(Grep/Read):**
  - `sed -n '88,120p' src/tui/ui.rs` → `KillConfirm` Enter = `ProcessKiller::send_signal(pid, KillSignal::Term)`
    1회 + `dialog=None; needs_rescan=true`. 폴링 없음 — **확인됨**.
  - `grep -n 'kill_in_progress' src/tui/app.rs` → L166 선언 / L216 `None` 초기화 2건뿐. `grep -rn
    'kill_in_progress =' src/` → **0건**(미대입 죽은 필드) — **확인됨**.
  - `grep -rn 'dialog = Some' src/` → KillConfirm/KillTreeConfirm/Help/SignalPicker 4종만. `ForceKillPrompt`
    설정 경로 **0건** → 미도달 — **확인됨**. (핸들러 `ui.rs:183`·위젯 `kill_dialog.rs:122`·`status_bar.rs:77`
    존재.)
  - `grep -rn 'graceful_kill' src/` → 호출은 `main.rs:418`(CLI 단건)뿐 — **확인됨**. `main.rs` L410-425에서
    `TimedOut` → `force_kill` 자동 호출 확인.
- **CHECK 1** `grep -rn '200ms\|폴링\|kill_in_progress 기록\|초과 시 자동' docs/11 docs/13 docs/20 docs/21` →
  잔존 히트는 전부 (a) "TUI는 폴링 없음" 부정문, (b) `CLI 단건`/`graceful_kill` 명시 라벨, (c) 용어집/값-출처
  메타뿐. **TUI 자동 에스컬레이션 주장 0건** — 각 히트 개별 검수 완료.
- **CHECK 2** `grep -rlE '</content>|</invoke>' docs/*.md` → **EMPTY**(Write 꼬리 누출 0).
- **원본 모순 문자열 제거 확인** `grep -rn '여부를 다시\|이후 200ms 간격으로 생존을 폴링\|매 Tick is_alive
  200ms 폴링\|graceful_timeout(10s) 초과 시 자동\|SIGTERM 전송→kill_in_progress 기록\|kill_in_progress 동안
  폴링\|kill_in_progress "폴링 대기"' …` → **EMPTY**(critique가 지목한 4개 문자열 전부 소거).
- **잔존 DLG-FORCE/ForceKillPrompt 검수** → 11/13/20/21의 모든 잔존 언급이 "미도달/수동 핸들러"로 프레이밍됨.
  §2-5 키 카탈로그·모달 5종 enum·status_bar 힌트(코드에 실재)만 중립 유지.
- **30/33 대조(verbatim 의미 일치):** `30` §2-6·§8-3 / `33` §2-1(경로별 표)·§2-3(TUI vs CLI 플로우차트) 재독 →
  13 §8-1/§8-2와 21 §3-12/§3-15가 "CLI 단건만 폴링+자동 force / TUI는 SIGTERM 1회 / ForceKillPrompt 수동"로
  **일치**. 13/21이 추가한 "현재 미도달" 표기는 Recommended Fix #3/#4가 명시 요구한 것이며 30/33의 "수동 별도
  모달"과 **모순 아님**(둘 다 자동 트리거 부재를 뜻함), 갭은 `40` M1에 로드맵화 → 단일 출처 위반 해소.
- **배너/구조 보존:** `20` L4·`21` L15-16 HUMAN_CHECKPOINT 배너 intact. F·V/TAB/DLG/BAR/CLI·PRB ID 변경/삭제 0.

## Known limitations

- **13 §8-1 상태기계는 ASCII 박스 정렬이 완벽하지 않다**(한글 전각폭 + 2-lane 분기). 의미(TUI ① / CLI ②
  분기)는 명확하나 픽셀 정렬은 STEP 7 사람 검수에서 다듬을 수 있다. (단 `13`은 와이어프레임 HUMAN_CHECKPOINT
  대상이 아니며 사실 정확성만 채점 대상이라 게이트엔 영향 없음.)
- **30/33 자체는 손대지 않았다.** critique Non-Blocking #1이 지적한 "30 §2-6·33 §2-1의 ForceKillPrompt 미세
  과장(수동 도달 가능 인상)"은 단일 출처(30이 kill 정본)를 흔들지 않기 위해 본문 정정 대신 **`40` M1 로드맵
  항목**으로 가시화했다. 코드 연결/제거 결정은 메인테이너 몫.
- **M2(NAME 배지)는 `13` §0에 반영했고 `12` §0에도 정정 블록을 추가**했으나, 본문 다른 곳의 "Info 탭에서만"
  관용 표현까지 전수 치환하진 않았다(부하 큰 사실 "별도 FRAMEWORK 컬럼 없음"은 일관·정확하므로 비차단).

## How to review (C3 회복 재확인 포인트)

Evaluator는 아래를 코드/`30`과 대조해 C3(통합일관성) 회복을 확인하면 된다:
1. **`13` §8-1·§8-2** ↔ `30` §8-3 / `33` §2-3 — 폴링·자동 SIGKILL이 **CLI 경로 ②에만**, TUI 경로 ①은 SIGTERM
   1회로 끝나는지. (`30` §2-6 "TUI는 graceful_kill 폴링/자동 에스컬레이션을 쓰지 않는다"와 일치하는지.)
2. **`21` §3-12·§3-15** ↔ `33` §2-1 표 — §3-12에 kill_in_progress/폴링 부재, §3-15에 "트리거 경로 현재 없음"
   명시, §2-5 핸들러 행 유지가 모두 충족되는지.
3. **`11` US-2.1** ↔ `30`/`33` — 수용 기준이 SIGTERM 1회 + "자동 승격 CLI 전용"으로 코드와 일치하는지.
4. **`20` W-9** — DLG-FORCE 진입이 "미도달"로 표기됐는지(자동 진입 주장 제거).
5. **`40` §3-3 M1·M2** — 죽은 코드/배지 갭이 코드 근거와 함께 로드맵화됐는지(창작 0).
6. (선택) **단일 출처:** spec §4-3 "kill 로직 정규 출처 = 30"에 대해 11/13/20/21이 더 이상 30과 상반되지 않는지.
