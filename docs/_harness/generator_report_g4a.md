# Sprint 4 Report (g4a) — service-planning-harness

## Strategic Decision
**REFINE** — 1라운드 신규 산출. g4a 범위(`30-functional-spec` + `31-erd`)를 실제 소스코드 정밀 분석 위에
작성. 사실 원천(codebase-facts §2~§6)과 실제 코드(`framework.rs`·`scanner.rs`·`killer.rs`·`network.rs`·
`streamer.rs`·`mod.rs`·`config.rs`·`app.rs`·`platform.rs`·`tree.rs`·`main.rs`·`ui.rs`)를 교차 검증해
역설계. 피벗 사유 없음.

## Deliverables produced (from sprint contract)
- `docs/30-functional-spec.md` — 동작·알고리즘 단일 출처(탐지 2단계 분류·스캔 2-pass·CPU delta·graceful_kill+
  force·kill_tree 역순·네트워크 파싱·로그 스트리밍·health·필터/정렬/Node-only·CLI 골격) + §8 플로우차트 5종.
- `docs/31-erd.md` — 인메모리 도메인 모델 단일 출처(DB 없음 명시 + ProcessInfo/Config(+3)/NetworkConnection/
  LogStreamer/App/Rule 전 필드 + enum 값 10종 + Config 기본값 정규표 + crow's-foot ASCII ERD + 관계 R1~R11).

## Verification I performed (실측 결과)
**30-functional-spec (S-1~S-8 전부 PASS):**
- S-1 분류: classify 3티어(FRAMEWORK→RUNTIME→config dev runner) + match_tier 매처순서(name_exact→
  command_binary→command_contains) + None=미표시 + normalize_name(16자) — §2-1 + §8-1 플로우차트. 규칙 정규표
  framework_rules.rs 원본과 1:1 대조 완료.
- S-2 스캔: pass1(runtime=Some)/pass2(ppid backfill, runtime=None)/persistent System CPU delta/포트 별도 —
  §2-3 + §8-2. scanner.rs scan() + main.rs do_scan 대조.
- S-3 종료: graceful_kill(SIGTERM→200ms 폴링→timeout(31값 10s)→TimedOut), CLI 단건 TimedOut시 force_kill
  자동 에스컬레이션, TUI KillConfirm은 send_signal 직접(폴링 없음)·ForceKillPrompt 수동 분리, kill_tree
  역순 — §2-6·2-7 + §8-3 상태도. killer.rs + main.rs cmd_kill + ui.rs dialog handler 대조.
- S-4 네트워크: lsof field-mode(`-F pcnT`)/netstat(`-ano -p TCP`)/parse_addr 3폼/명령부재→빈 HashMap —
  §2-5 + §8-4. network.rs 대조.
- S-5 로그: LOG_PATTERNS 7개/최신 수정순/SeekFrom::End/MAX_BUFFER_LINES=1000(31값)/Linux /proc/<pid>/fd/1
  fallback — §2-9 + §8-5. streamer.rs 대조.
- S-6 health: from_cpu_mem(≥90 Critical/≥80 Warning)·from_process_status(Zombie|Dead→Critical)·health() 결합
  순서 — §2-12. mod.rs 대조. **추가 정직 표기:** health()가 from_cpu_mem 2번째 인자로 RSS를 MB 환산값으로
  넘김(코드 그대로) 명시.
- S-7 시각화: §8-1~8-5 순수 ASCII(grep "mermaid" = 0).
- S-8 단일 출처: 값/정책/키/스키마를 31/33/21/32로 회부, 알고리즘만 소유.

**31-erd (E-1~E-7 전부 PASS):**
- E-1 DB 없음: §0 콜아웃 + 생애 도식(설정 파일만 디스크).
- E-2 구조체: ProcessInfo 18필드·Config+3하위·NetworkConnection 4·LogStreamer 3·App 전 필드·Rule 5 — 전부
  타입/코드근거. mod.rs/config.rs/network.rs/streamer.rs/app.rs/framework_rules.rs 대조.
- E-3 enum: Runtime8/FrameworkKind15/HealthStatus3/KillSignal(3+3)/KillResult4/GracefulResult5/DetailTab4/
  DialogKind5/FocusPanel2/SortColumn9 — Display/라벨 포함, 코드 대조.
- E-4 Config 기본값: §4 정규표(refresh=3·SIGTERM·graceful=10·confirm=true·tree=true·auto·mask=true·
  include_*=false·MAX_BUFFER_LINES=1000) — config.rs default() + streamer.rs 상수 대조.
- E-5 관계: §5 R1~R11(카디널리티·코드근거) + §1 crow's-foot ERD.
- E-6 시각화: §1 ASCII ERD + §3 표(mermaid 0).
- E-7 단일 출처: health 임계/분류 규칙/파싱/시그널 가용성/키를 30·33·21로 회부, 값만 정의.

**공통 게이트:** G-f(placeholder/TBD grep = 0), G-g(다이어그램 6+개, 외부 렌더러 0), Sprint Contract+자체
검증 양 문서 상단/하단 존재(grep 확인). 도구 호출 태그 누수 0.

## Known limitations
- `framework_version` 항상 None, Express/Fastify/Koa/Hapi 규칙 미등록 — 코드 실상으로 정직 표기, 로드맵은
  40-backlog 소유.
- health()의 메모리 인자가 백분율이 아닌 MB라는 코드 동작을 정직 기록(개선 = 40-backlog). 창작 아님.
- 키 바인딩/출력 스키마는 21/32 단일 출처 — 30은 처리 단계만, 참조로만 부기.
- 33-policy(권한 매트릭스·마스킹 규칙·fs 미독 정책 값)는 아직 미생성(g4 후속) — 30/31은 "→33" 참조로 연결만.

## How to review
- **30:** §8-1 플로우차트를 framework.rs `classify`/`match_tier`와, §8-2를 scanner.rs `scan`+main.rs `do_scan`과,
  §8-3을 killer.rs `graceful_kill`/`kill_tree`+main.rs `cmd_kill`+ui.rs dialog와 대조(S-1~S-3). §2-12 health
  임계가 30 단일 출처인지(31에 임계 미중복) 확인(S-6).
- **31:** §4 Config 기본값이 config.rs default()와 일치하는지, §2 ProcessInfo 18필드가 mod.rs와 일치하는지,
  §1 ERD에 children 자기참조 1:N + App↔ProcessInfo 1:N + runtime Option이 표현됐는지(E-2·E-4·E-5).
- **교차:** 30이 graceful_timeout/MAX_BUFFER_LINES 값을 적지 않고 "31값"으로만 인용하는지(중복정의 0, S-8/E-7).

READY_FOR_QA: generator_report_g4a.md
</content>
