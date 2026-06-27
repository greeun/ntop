# Critique — 최종 통합 패스 (Mode B, 16종 Mode 5 패키지)

## Verdict: **FAIL**

> 사유 한 줄: 16종 문서의 교차 추적성·단일 출처·사실 정확성은 대부분 모범적이나, **핵심 안전 동작(TUI kill→force 에스컬레이션)에 대해 g2/g3 문서(11·13·20·21)가 코드 및 자매 문서(30·32·33·40·41)와 정면으로 모순**한다. 이는 평가 브리프가 명시적으로 지목한 사실 정확성 점검 항목이자 spec §4-3 단일 출처 위반이며, 2× 가중 축 C3(통합일관성)를 4 미만으로 끌어내려 verdict 규칙상 FAIL이다. (수정 범위가 좁고 명확해 재수정 후 PASS 도달은 용이하다.)

---

## Rubric Scores

| Criterion | Score | Weight | Justification | Evidence |
|-----------|-------|--------|---------------|----------|
| **C2 차별화·독창성** | **5/5** | 2× | 실명 경쟁자 10종을 9축 매트릭스(✓/△/✗)+포지셔닝 쿼드런트로 비교하고, "RT·FW·PORT 동시 ✓는 ntop뿐"이라는 명시 축 + 3중 해자(데이터 주도 규칙 테이블 × 프로세스-로컬 전용 탐지 × 복리 확장구조) 논거까지 제시 — 캘리브레이션 5/5 앵커를 충족·초과. | `02-market-competition` §3-2 매트릭스(htop/btop/top/glances/gotop/Activity Monitor/ctop/pm2/systemd/launchd) + §7 "ntop의 해자 = ① × ② × ③" |
| **C3 실현가능성·통합일관성** | **3/5** | 2× | MVP/Out-of-scope 분리·우선순위 근거는 5/5 수준이나, **TUI kill→force 에스컬레이션이라는 핵심 안전 동작에 대해 4개 문서가 코드 및 5개 자매 문서와 모순**(아래 Blocking #1) — 통합일관성 절반이 핵심 흐름에서 깨짐. | (정상) `01` §5·`10` §3-3 MVP 경계 / (모순) `11` US-2.1 ↔ `30` §2-6 ↔ `src/tui/ui.rs` L95-108 |
| **C1 문제정의 명확성** | **5/5** | 1× | P1 박서연(월 1~2회 오kill·세션 끊김 10~15분), P2 정태호(새벽 2시 인시던트·월 2~3회 2차 재호출), P3 이준(npx MCP 10+개 cwd 제각각)까지 상황·빈도·결과로 구체화 — 5/5 앵커 충족. | `03-personas` §3·§4·§5 + §2 PRB-1~8 앵커표 |
| **C4 성공지표 측정가능성** | **5/5** | 1× | 제품 품질 지표는 수치+측정도구+주기+판정(scan<3s·CPU>0.0%·graceful 10s→force·10모듈 pass)으로, OSS 채택 지표는 전부 "가설" 라벨로 엄격히 분리. | `01-service-plan` §6-1/§6-2 |

**Verdict 규칙 적용:** 2× 가중 축 C3 = 3 < 4 → **FAIL**(핵심 규칙). C1·C4·C2는 모두 ≥4.

---

## §8 Gate Results (G-a ~ G-g) — 전부 PASS (binary)

| Gate | 결과 | 근거 |
|---|---|---|
| **G-a** 기능↔문제 추적 | **PASS** | `10-prd` §4 추적표(F1~F21 전부 PRB-1~8 매핑) + §4-1 양방향 커버리지 + §6-3 매트릭스. 미매핑 기능 0, PRB 전부 ≥1 커버. `11`§9·`13`§9도 일치. |
| **G-b** 타깃 구체성 | **PASS** | `03` 3 페르소나 상황·빈도·결과 구체(추상 인구통계만 0). |
| **G-c** 실명 차별화 비교 | **PASS** | `02` §3 실명 경쟁자 ≥3(실제 10종) + 비교 축 + moat 논거. |
| **G-d** 지표 측정가능성 | **PASS** | `01` §6 수치+도구+주기+판정, 채택 목표=가설 라벨. |
| **G-e** MVP/Out-of-scope 분리 | **PASS** | `01` §5(포함/제외/근거) + `10` §3-3 + `40` §3-1 영구 제외 박스. |
| **G-f** 플레이스홀더 0 | **PASS** | 16종 grep 결과 TBD/미정/lorem/빈칸 0건. |
| **G-g** 인포그래픽 적합 | **PASS** | 모든 구조 문서에 ASCII 다이어그램(생태계 맵·ERD crow's-foot·플로우차트·상태기계·시퀀스·간트·매트릭스). **Mermaid 등 외부 렌더러 0**(grep 확인). 화면 문서 per-screen 시안 충족(20: W-1~W-12 / 21: §3-1~§3-16 + V-LIST·TAB-LOG·TAB-NET 4상태 세트). |

> 비고: §8 게이트는 전부 PASS다. FAIL은 게이트 실패가 아니라 **2× 축 C3(통합일관성)의 핵심-흐름 모순**에서 발생한다.

---

## Integration Check Results (Mode B 핵심)

### 1) 교차 추적성 — 대체로 우수, 1개 핵심 모순
- **기능 ID(F1~F21):** `10`(소유) → `11`/`13`/`21`/`30`/`40`/`41` 전반 일관. F-ID 충돌 없음. ✔
- **화면 ID(V-/TAB-/DLG-/BAR-/CLI-):** `12`(소유) → `13`/`20`/`21` 일관. 21종 화면 ID 정합. ✔
- **PRB-1~8:** `03` §2(앵커) → `10` §4 → `11`/`13`/`41` 전반 인용 일관. ✔
- **❌ 단, kill→force 흐름:** `11`/`13`/`20`/`21`이 `30`/`32`/`33`/`40`/`41` 및 코드와 **모순**(Blocking #1).

### 2) 단일 출처(spec §4-3) — 1건 위반
- 돈/지속가능성=00, 가치/MVP/지표=01, 경쟁/생태계=02, 페르소나=03, 기능 ID/우선순위=10, 구조체/Config 기본값=31, 인터페이스 계약=32, 안전/프라이버시=33, 키 바인딩=21 — **소유/참조 경계 준수**. 같은 수치(`graceful_timeout=10`·`refresh_interval=3`·`MAX_BUFFER_LINES=1000`)는 31에만 값으로 정의되고 타 문서는 "31값"으로 인용 — **중복 값 정의 0**. ✔
- **❌ 위반:** spec §4-3은 **kill 처리 로직의 정규 출처를 `30-functional-spec`**으로 지정한다. 그런데 `13`(§8-1 상태기계)·`21`(§3-12·§3-15)이 30과 **상반된 kill 동작**을 기술 → 단일 출처 위반(Blocking #1).

### 3) 창작(fabrication) 스캔 — 0
- 결제/정산/take-rate/회원 DB/REST 엔드포인트 언급은 전부 **부정·면책·치환 맥락**(예: `00` 서두 면책, `31` §0 "DB 없음", `32` §0 "HTTP API 아님", `33` §6 "결제 파트너 없음"). grep으로 부정 맥락 외 출현 0건 확인. ✔

### 4) 코드 대비 사실 정확성 — 1건 부정확(핵심), 나머지 정확
- **세로 레이아웃**(목록 위 55%/상세 아래 45%, `ui.rs` L37-44): 전 문서 일관·정확. ✔
- **TUI 목록에 별도 FRAMEWORK 컬럼 없음:** 정확. 추가 확인 — 코드(`process_list.rs` L121/L127)는 NAME에 **인라인 배지**(`next-server (Next.js)` / `uvicorn [Python]`)를 붙인다. `20`/`21`이 이를 정확히 반영. ✔ (단, `12`/`13`의 "프레임워크는 Info 탭에서만 확인" 표현은 인라인 배지를 빠뜨려 약간 과장 — Non-Blocking 참고.)
- **8 런타임 / 시그널(Unix 6·Win 3) / Config 기본값:** 전 문서 일관·정확. ✔
- **❌ graceful 에스컬레이션 = CLI 단건 전용(TUI KillConfirm은 SIGTERM 직접):** 코드로 확정(`ui.rs` L95-108, `main.rs` L418, `kill_in_progress` 미사용·`ForceKillPrompt` 미도달). `30`/`32`/`33`/`40`/`41`은 정확, **`11`/`13`/`20`/`21`은 부정확**(Blocking #1).

### 5) 정직 발견 일관성 — PASS
- `health()`가 `from_cpu_mem`에 메모리를 %가 아니라 **RSS의 MB 환산값**으로 넘기는 버그: 코드 확인(`mod.rs` L302 `self.memory_rss as f32 / 1_048_576.0`). `30` §2-12·`40` R6·`41` TC-11.6에서 **버그/로드맵/테스트 갭**으로 정직 기록 — 타 문서에서 "정상 동작"으로 위장 0. ✔
- `framework_version` 항상 `None`: 코드 확인(`mod.rs` L165, Some 할당 없음). `31`/`32`/`20`/`21`에서 "항상 -/null"로 표기, `40` R7 로드맵화. ✔
- CLI `info` env 미마스킹(`33` §4-2·`40` R8), 종료코드 항상 0(`32` §2-7·`40` R9), 과마스킹(`40` R10) — 모두 정직 기록·일관. ✔

---

## Wireframe Checkpoint

- `20-wireframes.md` 및 `21-screen-spec.md`는 **화면별 ASCII 터미널 시안(per-screen mockup)** 을 포함한다(20: W-1~W-12, 21: §3-1~§3-16 + 핵심 3화면 4상태 세트). 두 문서 모두 상단에 명시적 **🟡 HUMAN_CHECKPOINT_REQUIRED 배너** 보유.
- **→ `HUMAN_CHECKPOINT_REQUIRED`**: 시각 품질(폭·정렬·문자 충실도)은 채점에서 제외하며 **STEP 7 메인테이너/디자이너 검수 대상**이다. 위치: `docs/20-wireframes.md` 상단 배너 + §2 검수 포인트 6항, `docs/21-screen-spec.md` L14-17 배너.
- (사실 정확성은 채점했고, 화면 문자열·컬럼·다이얼로그 문구는 코드와 대체로 일치. 단 §3-15 DLG-FORCE 트리거 표기는 Blocking #1에 해당.)

---

## Blocking Issues

### ❌ Blocking #1 — TUI kill→force 에스컬레이션: 4개 문서가 코드 및 자매 문서와 모순 (핵심 안전 흐름)

- **무엇이 문제인가:** ntop의 가장 위험한 동작(프로세스 종료)의 TUI 동작이 문서마다 다르게, 그리고 **코드와 다르게** 기술됨.
- **코드 진실(소스 직접 확인):**
  - `src/tui/ui.rs` L95-108 — `KillConfirm`의 `Enter`는 `ProcessKiller::send_signal(pid, KillSignal::Term)` **1회 직접 전송** 후 `dialog=None; needs_rescan=true`. **폴링 없음.**
  - `src/tui/app.rs` — `kill_in_progress` 필드는 L166 선언 + L216 `None` 초기화뿐, **어디에서도 대입되지 않음**(죽은 필드, grep `kill_in_progress =` → 0건).
  - `ForceKillPrompt`는 `app.dialog = Some(...)`로 **설정되는 경로가 없음**(설정되는 다이얼로그는 KillConfirm/KillTreeConfirm/Help/SignalPicker 4종뿐) → **현재 도달 불가능한 죽은 코드**.
  - `graceful_kill`(폴링+자동 force)은 `src/main.rs` L418의 **CLI 단건 경로에서만** 호출.
  - ⇒ **TUI는 SIGTERM 1회만 보내고, 자동 폴링·자동 SIGKILL 에스컬레이션이 없다. 자동 graceful→force는 CLI 단건 전용.**
- **정확하게 기술한 문서(코드 일치):**
  - `30-functional-spec` §2-6 — "TUI KillConfirm(Enter)은 send_signal(pid, SIGTERM)을 직접 보내고 … TUI는 graceful_kill 폴링/자동 에스컬레이션을 쓰지 않는다."
  - `32-api-spec` §6-2 주석 — "TUI KillConfirm 경로는 이 폴링/자동 force를 쓰지 않음 — send_signal(SIGTERM) 직접 1회."
  - `33-policy` §2-1·§2-3 — "TUI 단건은 SIGTERM만 … 자동 에스컬레이션은 CLI 단건에서만."
  - `40-backlog` §2-1(F7) "CLI 자동 에스컬레이션·TUI 수동 2단계", `41-qa` TC-3.8(자동 force를 CLI에만 귀속).
- **모순(부정확)한 문서 — 기대 vs 실제:**
  | 문서·위치 | 기술된 내용(실제) | 코드 진실(기대) |
  |---|---|---|
  | `11-user-stories` US-2.1(L172-176) | "이후 200ms 간격으로 생존을 폴링한다 … timeout까지 살아 있으면 **TUI는 ForceKillPrompt(DLG-FORCE)로** … 다시 묻고" | TUI는 폴링 안 함·DLG-FORCE 자동 표시 안 함 |
  | `13-user-flows` UF-3(§4)·§8-1(상태기계)·§8-2(시퀀스) | "(이후 매 Tick is_alive 200ms 폴링)" → "graceful_timeout 안에 죽었나?" → "**[DLG-FORCE]**", 상태기계에 "TUI: [DLG-FORCE] 확인" | TUI 폴링/자동 DLG-FORCE 분기 없음 |
  | `20-wireframes` W-9(L302) | "DLG-FORCE … **진입: graceful_timeout(10s) 초과 시 자동**" | TUI에서 자동 진입 경로 없음(도달 불가) |
  | `21-screen-spec` §3-12(L450-451)·§3-15(L504) | "Enter=SIGTERM 전송→**kill_in_progress 기록**" / "kill_in_progress 동안 **폴링**" / DLG-FORCE "**진입: … 초과 시 자동**" | kill_in_progress 미기록(죽은 필드)·폴링 없음·자동 진입 없음 |
- **영향:** (a) 단일 출처 위반 — spec §4-3은 kill 로직 정규 출처를 `30`으로 지정하나 `13`/`21`이 30과 상반. (b) 사실 부정확 — 평가 브리프가 명시 지목한 "graceful escalation is CLI-single-kill (TUI KillConfirm sends SIGTERM directly)" 점검에서 4개 문서가 탈락. (c) 안전 오해 — 사용자/기여자가 "TUI가 10초 뒤 자동으로 강제 종료한다"고 잘못 믿게 됨(파괴적 동작에 대한 위험한 오해).
- **심각도:** **Blocking** — 핵심 안전 흐름의 교차 문서 모순 + 코드 불일치. (2× 축 C3를 4 미만으로 끌어내림.)

---

## Non-Blocking Notes

1. **(정확한 문서들도) ForceKillPrompt/kill_in_progress의 "도달 가능성" 미세 과장.** `30` §2-6·`33` §1-1/§2-1은 ForceKillPrompt를 "별도 수동 모달(별도 진입)"로 기술해 **수동으로는 도달 가능**한 듯한 인상을 준다. 그러나 코드상 어떤 키/경로로도 트리거되지 않는 **완전한 죽은 코드**이며 `kill_in_progress`도 미사용 필드다. → `40-backlog`에 "ForceKillPrompt·kill_in_progress = 현재 미도달 죽은 코드(연결 또는 제거)" 로드맵 항목 추가 권장. (Blocking #1 수정 시 함께 정리.)
2. **"프레임워크는 Info 탭에서만" 표현의 미세 부정확.** `03`/`10`/`11`/`12`/`13`은 "프레임워크 정체는 상세 Info 탭/CLI 테이블에서만 확인"으로 적었으나, 실제 목록 NAME 컬럼은 인라인 배지(`(Next.js)`/`[Python]`)로 프레임워크를 노출한다(`20`/`21`은 정확히 반영). 부하가 큰 사실("별도 FRAMEWORK 컬럼 없음")은 일관·정확하므로 비차단. → `12`/`13` §0 정정 박스에 "NAME 인라인 배지" 한 줄 보강 권장.
3. **INDEX.md·HTML 미생성:** 현재 `docs/`에 INDEX.md·`NN-name.html`·`index.html`·`overview.html`이 없다. 단 sprint-playbook은 HTML(g6)·INDEX(g7)를 **md 문서 PASS + STEP 7 휴먼 체크포인트 이후**로 명시 시퀀스 → 이 게이트 시점에서의 부재는 **정상(후속 단계)**이며 verdict에 불리하게 계산하지 않음. (다만 spec §8 DoD의 INDEX/HTML 항목은 추후 별도 검증 필요.)

---

## Recommended Fixes (재수정 가이드 — Blocking #1)

`30`/`32`/`33`/`40`/`41`의 **코드-진실 모델을 정본**으로 삼아 `11`/`13`/`20`/`21`을 정렬:

1. **`11-user-stories` US-2.1** — "200ms 폴링/타임아웃 후 TUI ForceKillPrompt" 서술 제거. 수용 기준을 "`x`→KillConfirm→`Enter`로 **SIGTERM 1회 전송**, 이후 재스캔으로 종료 반영. (강제 종료 자동 승격은 **CLI `ntop kill` 단건 전용**)"으로 교정.
2. **`13-user-flows`** — UF-3 흐름도에서 TUI의 "200ms 폴링→graceful_timeout→DLG-FORCE 자동" 분기를 CLI 경로로 이동. §8-1 상태기계/§8-2 시퀀스의 "TUI: [DLG-FORCE] 확인"을 "TUI: SIGTERM 직접 1회(폴링·자동 force 없음)"로 교정(`30` §8-3·`33` §2-3과 일치시킴).
3. **`20-wireframes` W-9** — DLG-FORCE 헤더 "진입: graceful_timeout 초과 시 자동"을 "(현재 코드에서 미도달 — 자동 graceful→force는 CLI 전용)"으로 정정하거나, 죽은 코드임을 명시.
4. **`21-screen-spec` §3-12·§3-15** — "kill_in_progress 기록/폴링", DLG-FORCE "초과 시 자동 진입" 제거. §2-5 키 카탈로그의 DLG-FORCE 핸들러 자체는 코드에 존재하므로 유지하되, **트리거 경로가 현재 없음**을 §3-15 상태/예외에 명시.
5. **(선택)** `40-backlog`에 "ForceKillPrompt/kill_in_progress 미도달 죽은 코드 — 연결 또는 제거" + "`12`/`13` NAME 인라인 배지 표기 보강" 항목 추가.

→ 위 정렬 완료 시 C3는 4~5로 회복되어 PASS 도달 가능.

---

## Iteration Quality Note

- 기술/실행 계층(`30`·`32`·`33`·`40`·`41`)은 **g4a 코드 정밀 분석으로 kill 동작 오해를 명시적으로 바로잡았다**(`33` §2 "흔한 오해를 바로잡는다(g4a 발견)"). 이는 매우 강한 반복 품질이다. 문제는 그 정정이 **선행 g2/g3 문서(11·13·20·21)로 역전파되지 않은** 데 있다. 즉 결함은 분석 부족이 아니라 **정정의 미전파**이며, 그래서 수정 범위가 좁고 명확하다.

## Recommended Next Focus

1. **(최우선) Blocking #1 역전파** — g4의 kill 정본을 g2/g3 4개 문서에 반영(위 Recommended Fixes 1~4). 이 한 가지가 C3 회복의 전부다.
2. Non-Blocking #1·#2 동반 정리(죽은 코드 명시 + NAME 인라인 배지 표기).
3. 이후 STEP 7 휴먼 체크포인트(20/21 시각 검수) → g6 HTML → g7 INDEX 순으로 진행.

---

FINAL VERDICT: FAIL
