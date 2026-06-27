# Critique — Final Integrated Pass (Round 2, post kill-fix)

## Verdict: **PASS**

> 사유 한 줄: 라운드 1의 단일 Blocking(#1, TUI kill→force 에스컬레이션 모순)이 **코드 직접 확인 기준으로 완전히 해소**됐다. 모순 문서 4종(`11`/`13`/`20`/`21`)이 모두 `30`/`32`/`33`/`40`/`41` 및 소스코드(`ui.rs` L95-108·`main.rs` L418)와 정렬됐고, 잔존 "200ms/폴링/자동" 표현은 전수 검수 결과 **부정문·CLI-경로 명시 라벨·용어집/값-출처 메타뿐**(살아남은 TUI-자동승격 주장 0건). 단일 출처(spec §4-3, kill 정본=`30`) 위반 해소, 신규 창작 0, 교차 ID(F·V/TAB/DLG/BAR/CLI·PRB) 무손상. 2× 축 C3가 3→5로 회복되어 verdict 규칙(전 축 ≥4 ∧ 2×축 ≥4 ∧ 프로브 클린 ∧ §8 전부 PASS)을 충족한다.

---

## Rubric Scores

| Criterion | Score | Weight | Justification | Evidence |
|-----------|-------|--------|---------------|----------|
| **C2 차별화·독창성** | **5/5** | 2× | 라운드 1과 동일(수정이 손대지 않은 영역). 실명 경쟁자 10종 9축 매트릭스 + 3중 해자 논거 유지. 회귀 없음. | `02-market-competition` §3-2·§7 (변경 없음) |
| **C3 실현가능성·통합일관성** | **5/5** | 2× | **핵심 회복 축.** kill→force 흐름이 이제 4개 문서·5개 자매 문서·코드에서 **단일 모델**로 일치. `13` §8을 **경로 ①(TUI=SIGTERM 1회)·②(CLI=graceful_kill 폴링+자동 SIGKILL) 2-lane**으로 재작도, `11` US-2.1·`20` W-9·`21` §3-12/§3-15가 "미도달 죽은 코드/CLI 전용 자동 승격"으로 코드 정렬. 죽은 코드·표기 갭은 `40` M1·M2로 **코드 근거와 함께** 로드맵화(창작 0). MVP/Out-of-scope 경계는 라운드 1의 5/5 유지. | `13` §4·§8-1·§8-2 ↔ `30` §2-6/§8-3 ↔ `33` §2-1(P-2 표 L114) ↔ `ui.rs` L99·L102 / `main.rs` L418-424 |
| **C1 문제정의 명확성** | **5/5** | 1× | 라운드 1 유지(P1~P3 상황·빈도·결과 구체). 수정 무영향. | `03-personas` §3~§5 (변경 없음) |
| **C4 성공지표 측정가능성** | **5/5** | 1× | 라운드 1 유지(수치+도구+주기+판정 / 채택=가설 라벨). 수정 무영향. | `01-service-plan` §6 (변경 없음) |

**Verdict 규칙 적용:** 전 축 ≥4 ✓ · 2× 가중 축 C2=5·C3=5 ≥4 ✓ · 프로브(leak/placeholder/probe-1) 클린 ✓ · §8 게이트 전부 PASS ✓ → **PASS.**

---

## §8 Gate Results (G-a ~ G-g)

| Gate | 결과 | 근거 |
|---|---|---|
| **G-a** 기능↔문제 추적 | **PASS** | `10` §4 추적표(F1~F21 전부 PRB-1~8 매핑) 무손상. F-ID·PRB 범위 재확인: `grep` 결과 F1~F21 연속·PRB-1~8 전부 존재. `13` §9-1 추적표에 신규 행 "**CLI 단건(§7) \| 자동 graceful→force \| (graceful_kill) \| (timeout 초과 시 자동) \| F7 \| 5**"가 일관 추가(F7 재사용, 신규 ID 0). |
| **G-b** 타깃 구체성 | **PASS** | `03` 3 페르소나 변경 없음. |
| **G-c** 실명 차별화 비교 | **PASS** | `02` 변경 없음. |
| **G-d** 지표 측정가능성 | **PASS** | `01` §6 변경 없음. |
| **G-e** MVP/Out-of-scope 분리 | **PASS** | `40` §3-1 영구 제외 박스 유지 + §3-3 신설(M1·M2)은 "정합 항목"으로 기능 로드맵과 **분리** 기록. |
| **G-f** 플레이스홀더 0 | **PASS** | `grep -rniE 'TBD\|미정\|lorem\|placeholder\|XXX' docs/*.md` → EMPTY. |
| **G-g** 인포그래픽 적합 | **PASS** | `13` §8-1 상태기계·§8-2 시퀀스가 **2-lane ASCII 다이어그램**으로 재작도(외부 렌더러 0). `20` W-9·`21` §3-15 ASCII 모달 시안 유지. 텍스트 벽/빈 다이어그램 0. (정렬 미세 흠은 아래 Wireframe Checkpoint·Non-Blocking.) |

> 비고: §8 게이트는 라운드 1에서도 전부 PASS였고, 수정이 건드린 G-a(추적)·G-g(인포그래픽)·단일 출처(§4-3)를 재검한 결과 모두 PASS 유지·회복.

---

## Integration Check Results (kill 흐름 집중)

### 1) 교차 추적성 — 회귀 0, 핵심 모순 해소
- **기능 ID(F1~F21):** `grep` 결과 `10-prd`에 F1~F21 연속 존재, 삭제/충돌 0. `11`/`13`/`20`/`21` 편집부가 모두 **기존 F7**(안전 종료)을 재사용 — 신규 F-ID 창작 0. ✔
- **화면 ID:** `12-ia` 기준 V-MAIN/V-LIST/V-DETAIL/V-EMPTY/V-FILTER · TAB-INFO/LOG/NET/ENV · DLG-KILL/KILLTREE/SIGNAL/FORCE/HELP · BAR-TOP/BOTTOM · CLI-LIST/KILL/INFO/LOG/CONFIG — 전 패밀리 정합(`grep` 확인). DLG-FORCE는 카탈로그에 **유지**되되 트리거 부재가 명시됨. ✔
- **PRB-1~8:** 전 문서 출현 일관(`grep` PRB-1~8 전부). UF-3 트레이스 PRB-3·5 유지. ✔
- **❌→✔ kill 흐름:** 라운드 1에서 `30`/`32`/`33`/`40`/`41`과 상반됐던 `11`/`13`/`20`/`21`이 이제 동일 모델로 수렴. (상세는 아래 §4.)

### 2) 단일 출처(spec §4-3) — 위반 해소
- spec §4-3은 **kill 처리 로직 정규 출처 = `30-functional-spec`**. 편집된 4개 문서가 이제 모두 "**정규 출처 `30` §2-6·§8-3 / 정책 `33` §2-1**"을 **명시 인용하며 그 모델을 재기술(재정의 아님)**: `11` US-2.1 참고 블록(L177), `13` §4 인용(L186)·§8 인트로(L291)·§8-1 각주(L329-332), `21` §3-12 연결데이터(L450)·§3-15(L529). 30/33과 상반되는 서술 0. → **단일 출처 위반 소거.** ✔
- 동일 수치(`graceful_timeout=10`·폴링 200ms)는 여전히 `31`/`33`을 값 출처로 인용("31값")하고 편집 문서는 의미만 인용 — 중복 값 정의 0. ✔

### 3) 창작(fabrication) 스캔 — 0
- `40` §3-3 신설 항목 **M1·M2는 코드 검증 사실에만 근거**: M1은 `app.dialog=Some(...)` 4종만 설정(`ui.rs`)·`kill_in_progress =` grep 0건, M2는 `process_list.rs` L121·L127 인라인 배지. 둘 다 `[제안]`/Low로 라벨, 근거 컬럼에 코드 위치 명시. 근거 없는 기능/수치 창작 0. ✔
- 결제/정산/take-rate/REST 등 도메인 금지어 신규 출현 0(편집 범위가 kill/배지 한정). ✔

### 4) 코드 대비 사실 정확성 — kill 흐름 전수 재확인(코드 직접 열람)
**소스 직접 확인 결과(Generator 주장 비의존):**
- `src/tui/ui.rs` L95-108 — `KillConfirm` `Enter` = `ProcessKiller::send_signal(pid, KillSignal::Term)` **1회**(L99) → `app.dialog=None`(L101)·`app.needs_rescan=true`(L102). **폴링/대기/자동 force 없음.** ✔
- `src/tui/app.rs` L166 `kill_in_progress: Option<(u32,Instant)>` 선언 / L216 `None` 초기화 — `grep -rn 'kill_in_progress' src/` 결과 그 2건뿐, **대입 0건**(죽은 필드). ✔
- `grep -rn 'dialog = Some' src/` → `KillConfirm`(ui.rs:264)·`KillTreeConfirm`(:270)·`Help`(:275)·`SignalPicker`(:281) **4종만**. `ForceKillPrompt` 설정 경로 0 → **미도달**. (핸들러 `ui.rs:183`·위젯 `kill_dialog.rs:122`·`status_bar.rs:77` 존재하나 트리거 없음.) ✔
- `src/main.rs` L418 `graceful_kill(target_pid, timeout)` → L421-424 `TimedOut` 시 `force_kill` **자동** 호출 = **CLI 단건 경로 전용**. ✔
- `src/tui/widgets/process_list.rs` L121-127 — framework≠Generic → `"{display_name} ({framework})"` (예: `(Next.js)`), Node-generic → 배지 없음, 그 외 런타임 → `"{display_name} [{rt}]"` (예: `[Python]`). → `40` M2·`12` §0 주장 **정확**. ✔

**편집 문서 ↔ 코드 대조:**
| 문서·위치 | 현재 기술 | 코드 진실 | 판정 |
|---|---|---|---|
| `11` US-2.1 (L173-180) | "SIGTERM 1회 직접 전송(TUI 폴링 없음)" + "자동 승격은 CLI `ntop kill` 단건 전용" + "ForceKillPrompt 미도달 죽은 코드(40 M1)" | 일치 | ✔ |
| `13` §4(L182-211)·§8-1(L294-332)·§8-2(L334-362) | 2-lane: ①TUI SIGTERM 1회 / ②CLI graceful_kill 폴링+자동 SIGKILL. 폴링/timeout/force_kill 분기는 전부 경로 ②에 배치 | 일치 | ✔ |
| `20` W-9 (L301-318) | 헤더 "현재 미도달 죽은 코드" + 진입 "(미도달 — 어떤 키/경로로도 트리거되지 않음)" + 푸터 "자동 승격 CLI 전용" | 일치 | ✔ |
| `21` §3-12(L450)·§3-15(L506-531) | §3-12 SIGTERM 1회·폴링 없음·kill_in_progress 미사용 필드 표기 / §3-15 "트리거 경로 현재 없음·미도달" + §2-5 핸들러 행 유지 | 일치 | ✔ |

### 5) 정직 발견 일관성 — PASS(유지)
- `health()` MB-as-% 버그·`framework_version` 항상 None·CLI info env 미마스킹·종료코드 0 등 라운드 1 정직 기록 유지. 이번 수정은 **새로운 정직 항목(미도달 죽은 코드·미사용 필드)** 을 `40` M1로 추가 가시화 — 정직성 강화. ✔

---

## Wireframe Checkpoint

- **`HUMAN_CHECKPOINT_REQUIRED`** — `20-wireframes.md`(상단 배너·L4 intact)·`21-screen-spec.md`(L15-16 배너 intact)는 화면별 ASCII 터미널 시안(per-screen mockup)을 포함하므로 **시각 품질(폭·정렬·문자 충실도)은 채점 제외, STEP 7 메인테이너/디자이너 검수 대상**. 수정으로 추가/변경된 `20` W-9·`21` §3-15 모달 시안도 동일 처리.
- **추가 체크포인트:** `13-user-flows.md` §8-1 상태기계는 **와이어프레임 문서가 아님**(사실 정확성만 채점)이나, 2-lane 분기 + 한글 전각폭으로 **ASCII 박스 정렬이 부분적으로 어긋난다**(Generator도 Known limitation으로 자진 신고). 의미(경로 ①/②)는 명확 → 채점 무영향, 정렬은 STEP 7에서 다듬기 권장.
- 텍스트 파트(헤더/푸터/명세 산문)는 정상 채점했고 코드와 일치.

---

## Blocking Issues

**없음.** 라운드 1 Blocking #1(TUI kill→force 모순)은 코드 직접 확인 기준으로 **완전 해소**. 신규 Blocking 0.

검증 프로브 결과(전수):
- `grep '200ms|폴링|kill_in_progress 기록|초과 시 자동'` (11/13/20/21) → 잔존 히트 전부 (a) "TUI 폴링 없음" 부정문 / (b) "CLI 단건 전용"·"graceful_kill" 명시 라벨(경로 ②) / (c) 용어집 정의·"31값"·트레이스표 메타뿐. **살아남은 TUI-자동승격 주장 0건** — 개별 인용 검수 완료.
- 원본 모순 문자열 6종(`TUI는 ForceKillPrompt`/`이후 200ms 간격으로 생존을 폴링`/`매 Tick is_alive 200ms 폴링`/`SIGTERM 전송→kill_in_progress 기록`/`kill_in_progress 동안…폴링`/`graceful_timeout(10s) 초과 시 자동`) → `grep` **EMPTY**(전부 소거).
- `grep -rlE '</content>|</invoke>' docs/*.md` → **EMPTY**(누출 태그 0).

---

## Non-Blocking Notes

1. **`13` §8-1 상태기계 ASCII 정렬 미세 흠.** 2-lane + 한글 전각폭으로 일부 박스 경계가 어긋남(예: L307-323). 의미 전달엔 지장 없음 → STEP 7 시각 검수에서 정렬 보정 권장. (사실 정확성·게이트 무영향.)
2. **M2(NAME 배지) 본문 잔존 관용표현.** `12` §0·`13` §0에 정정 블록은 추가됐으나, 다른 문서 본문의 "프레임워크는 Info 탭/CLI 테이블에서만" 관용 표현까지 전수 치환하진 않음. 부하 큰 사실("별도 FRAMEWORK 컬럼 없음")은 일관·정확하므로 비차단. `40` M2로 가시화됨.
3. **`30` §2-6·`33` §1-1의 "ForceKillPrompt = 수동 별도 모달" 표현.** 라운드 1 Non-Blocking #1과 동일 — 코드상 어떤 경로로도 트리거되지 않는 완전 죽은 코드인데 "수동 도달 가능" 인상을 약간 남김. Generator는 단일 출처(30=정본) 안정성을 위해 본문 정정 대신 `40` M1 로드맵으로 처리. 합리적 트레이드오프, 비차단. (코드 연결/제거는 메인테이너 몫.)
4. **INDEX.md·HTML 미생성:** 라운드 1과 동일 — sprint-playbook 시퀀스상 md PASS + STEP 7 휴먼 체크포인트 이후 단계이므로 이 게이트 시점 부재는 정상. verdict 불리 계산 안 함.

---

## Iteration Quality Note

수정은 **정확히 지목된 범위에만, 코드 정본 방향으로** 가해졌다 — Recommended Fixes 1~5를 모두 이행하면서 신규 사실/수치/기능 창작 0, 기존 ID 무손상. 특히 `13` §8을 단순 문구 교정이 아니라 **2-lane 상태기계+시퀀스로 재구조화**해 "TUI=SIGTERM 1회 / CLI=폴링+자동 force"의 차이를 시각적으로 명확화한 점, 그리고 죽은 코드·미사용 필드를 숨기지 않고 `40` M1으로 **정직하게 로드맵화**한 점이 고품질이다. 라운드 1에서 진단한 결함의 본질("g4 정정의 g2/g3 미전파")이 정확히 해소됐다 — 분석이 아니라 전파의 문제였고, 전파가 완료됐다.

## Recommended Next Focus

1. **(최우선) STEP 7 휴먼 체크포인트** — `20`/`21` 화면 시안 + `13` §8-1 상태기계 ASCII 정렬을 사람이 시각 검수.
2. 이후 g6 HTML(`NN-name.html`·`index.html`·`overview.html`) → g7 INDEX.md 순서로 진행(현재 미생성은 정상 후속 단계). HTML 생성 시 spec §8 DoD의 "페이지당 인라인 SVG 다수·ASCII→SVG 변환·외부 렌더러 0"을 별도 검증 필요.
3. (선택) Non-Blocking #2·#3 동반 정리 — 본문 관용표현 치환 / `30`·`33`의 ForceKillPrompt "수동 도달" 인상 미세 보정. 모두 로드맵에 가시화돼 있어 비긴급.

---

FINAL VERDICT: PASS
