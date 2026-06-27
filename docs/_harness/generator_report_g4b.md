# Sprint 4 Report (g4b) — service-planning-harness

## Strategic Decision
**REFINE** — 1라운드 신규 산출. g4b 범위(`32-api-spec` + `33-policy`)를 실제 소스코드 정밀 분석 위에 작성.
사실 원천(codebase-facts §0-1·§4-1·§5·§7)과 실제 코드(`cli.rs`·`main.rs`·`lib.rs`·`network.rs`·`killer.rs`·
`framework.rs`·`config.rs`·`tree.rs`·`streamer.rs`·`ui.rs`·`env_tab.rs`)를 교차 검증해 역설계. g4a 산출물
(`30-functional-spec`·`31-erd`)과 g4a 리포트의 종료 정책 발견을 정확히 반영. 피벗 사유 없음.

## Deliverables produced (from sprint contract)
- `docs/32-api-spec.md` — 외부 인터페이스 계약 단일 출처. 3종 계약(① CLI 서브커맨드/플래그/필수성/종료코드/
  출력 스키마 table7·JSON17+depth·CSV9 ② 공개 lib API `ntop::` 시그니처 ③ 의존 시스템명령 lsof/netstat/
  taskkill + 부재→빈 결과) + CLI 호출 시퀀스 다이어그램 2종 + 계약 요약. **HTTP 부재 명시.**
- `docs/33-policy.md` — 안전·프라이버시 정책 단일 출처. kill 확인 게이트 + ★정확한 종료 정책(TUI vs CLI
  경로 구분) + 시그널 권한 매트릭스 히트맵(Unix6/Win3·EPERM/ESRCH) + env 마스킹(8패턴·TUI 전용) + ★fs 미독
  프라이버시 원칙 + 시각화 3종(히트맵·플로우차트·시퀀스).

## Verification I performed (실측 결과)

**32-api-spec (A-1~A-7 전부 PASS):**
- A-1 CLI 계약: `cli.rs` Commands enum 대조 — `list{json,format}`·`kill{pid,tree,signal,all,no_confirm}`
  (`required_unless_present="all"`)·`info{pid}`·`log{pid}`·`config`·무서브=TUI. 플래그 상호작용(--json>
  --format, --signal from_str 실패→SIGTERM 폴백) 코드 확인.
- A-2 종료코드: `main()`이 `anyhow::Result` 반환, 모든 cmd_*가 실패 상황에도 `Ok(())` 반환 확인 → 0/2(clap)/
  1(런타임) 표. **"not found/cancelled/permission denied=0" 정직 표기**(자동화는 stdout 파싱 필요).
- A-3 출력 스키마: `print_table`/`print_json`/`print_csv` 코드 직접 대조 — table 7컬럼(FRAMEWORK 포함),
  JSON 17키(+depth, runtime nullable, framework_version null), CSV 9컬럼. **table/JSON/CSV 차이 정직 표기**
  (CSV는 원문 name·세미콜론 포트·% 없는 CPU·원시 byte MEMORY).
- A-4 lib API: `lib.rs` pub 모듈 5개 + 각 모듈 `pub fn` 시그니처 grep 대조 — ProcessScanner/NetworkInspector/
  ProcessKiller/KillSignal/FrameworkDetector/Config/TreeBuilder/LogStreamer 코드 그대로. 타입은 31 회부.
- A-5 시스템명령: `network.rs` L98-99(`lsof -iTCP -sTCP:LISTEN,ESTABLISHED -nP -F pcnT`)·L194-195
  (`netstat -ano -p TCP`)·`killer.rs` L219-220(`taskkill /PID`) 대조 + 부재 처리 L104/107·L200/203
  (`Err→HashMap::new()`, `!status.success()→HashMap::new()`).
- A-6 시각화: §6-1 list 시퀀스·§6-2 kill 에스컬레이션 시퀀스·§6-3 요약 + §2/§3 표(순수 ASCII).
- A-7 단일 출처: 값=31·동작=30·정책=33 참조, **HTTP/REST/URL/메서드/상태코드/DB/회원/결제 0**(§0 콜아웃).

**33-policy (P-1~P-7 전부 PASS):**
- P-1 확인 게이트: `cmd_kill`의 `!no_confirm && confirm_before_kill` 분기 + TUI 3모달 확인. 취소→exit 0.
- P-2 ★ 정확한 종료 정책(g4a 발견 반영): `ui.rs` L99(`send_signal(pid,Term)` 직접)·L102(needs_rescan)·
  L116(kill_tree Term)·L145(SignalPicker send_signal)·L187(ForceKillPrompt force_kill) vs `main.rs cmd_kill`
  L417-429(graceful_kill→TimedOut→force_kill 자동) 대조. **TUI=SIGTERM 직접/CLI 단건만 자동 에스컬레이션/
  ForceKillPrompt 수동/kill_tree 역순(killer.rs L183 `.rev()`)** 경로별 표 + TUI vs CLI 플로우차트.
- P-3 권한 매트릭스: `killer.rs` `all()`(Unix6/Win3) + L142-143(ESRCH→AlreadyDead, EPERM→PermissionDenied)
  + Windows 5/87·taskkill stderr 매핑. 히트맵 + 오류 표.
- P-4 env 마스킹: `env_tab.rs` L13-22(8패턴 PASSWORD/SECRET/TOKEN/KEY/API_KEY/PRIVATE/CREDENTIALS/AUTH)·
  L25-30(uppercase contains)·L48-49(`********`)·L42(mask_env_values). **★ TUI 전용 — `cmd_info` L503-505는
  원문 미마스킹 정직 표기**(프라이버시 갭 → 40-backlog).
- P-5 ★ fs 미독: `framework.rs classify`가 name/command/&Config만 입력(cwd/fs 미접근) — 프라이버시 + 오분류
  회피 두 이유 + 안전 연결 + 반전 금지(spec §7) 명시.
- P-6 시각화: §3-1 히트맵·§2-3 플로우차트·§4-3 시퀀스(순수 ASCII).
- P-7 단일 출처: 값=31·동작=30·계약=32·지속가능성=00 참조, 결제/DB/REST 0(§6).

**공통 게이트:** G-f(placeholder/TBD/TODO grep = 0), G-g(다이어그램 32:3종+표 / 33:3종, 외부 렌더러 0 —
"mermaid 0"은 검증 문구일 뿐 실제 Mermaid 없음, 30/31과 동일 컨벤션), Sprint Contract(상단)+자체 검증(하단)
양 문서 grep 확인. **도구 호출 태그(`</content>`·`</invoke>`·`</antml`) 누수 0**(tail + grep 확인).

## Known limitations
- 종료코드가 결과(성공/미발견)를 구분 안 함 → 자동화는 stdout 파싱 필요 — 코드 실상 정직 기록(개선=40-backlog).
- CLI `ntop info` env 미마스킹(TUI만 마스킹) — 프라이버시 갭 정직 기록(개선=40-backlog).
- 마스킹 패턴 부분일치로 과마스킹 가능(`KEY` 등) — 정밀화 여지(40-backlog).
- 키 바인딩 정규 출처는 21-screen-spec(미생성, g3) — 32/33은 "→21" 참조로만 연결. 본 작업은 33이 종료/마스킹
  "정책"을, 32가 "계약"을 소유하고 키 매핑은 21에 회부.

## How to review
- **32:** §2-7 종료코드 표를 `main.rs`의 cmd_* 반환(`Ok(())` 패턴)과, §3 스키마를 `print_table/json/csv`와,
  §4-1 시그니처를 각 모듈 `pub fn`과, §5 시스템명령을 `network.rs` L98/194·`killer.rs` L219와 대조(A-2~A-5).
  HTTP/REST/DB가 부정문으로만 등장하는지(§0) 확인.
- **33:** §2-1 경로별 종료 표를 `ui.rs` L99/116/145/187 vs `main.rs cmd_kill` L417-429와 대조(P-2, g4a
  발견 일치 확인). §3-1 매트릭스를 `KillSignal::all()`과, §4 마스킹을 `env_tab.rs` SENSITIVE_PATTERNS와,
  §4-2 CLI 미마스킹을 `cmd_info` L503과 대조(P-3·P-4). §5 fs 미독이 `classify` 입력 시그니처와 일치하는지.
- **교차:** 32/33이 `graceful_timeout=10`·`confirm_before_kill=true`·`mask_env_values=true`를 값으로 재정의
  하지 않고 "31값"으로만 인용하는지(중복정의 0, A-7/P-7). 32와 33의 종료 정책 기술이 일치하는지(32 §6-2
  ↔ 33 §2-1).

READY_FOR_QA: 32-api-spec.md, 33-policy.md
