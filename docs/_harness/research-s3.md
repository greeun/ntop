# research-s3.md — UX/UI 플로우·화면 리서치 (ntop, S3)

> 본 문서는 PLANNER가 발행한 `spec.md`·`sprint-playbook.md`, **사실의 원천**인
> `docs/_harness/codebase-facts.md`(이하 "도시에") **+ 실제 소스코드**(`src/tui/**`, `src/cli.rs`,
> `src/main.rs`), 그리고 선행 산출물 `research-s1.md`(시장·경쟁·생태계)·`research-s2.md`(문제·페르소나)를
> 근거로 작성한 **S3 리서치 산출물**이다. ntop은 **이미 v0.2.0까지 구현된 MIT 오픈소스 도구**이므로,
> 이 리서치는 신규 화면 설계가 아니라 **이미 구현된 TUI/CLI를 역설계해 정보구조(IA)·유저 플로우·화면
> 인벤토리·키 바인딩·와이어프레임으로 정리**하는 작업이다.
>
> ntop은 **터미널 도구**다 — "화면(screen)"은 웹/GUI 페이지가 아니라 **TUI 뷰·패널·탭·다이얼로그 +
> CLI 명령 출력**을 뜻한다. 따라서 **ASCII 레이아웃이 곧 실제 충실도**다(spec §4-0). 본 문서의 모든
> 화면/플로우/키는 도시에 §6·§7 **또는 실제 소스코드**에 역추적된다(§-번호 / 파일:라인 부기).
>
> **용어 풀이(첫 등장):** **TUI**(Text User Interface) = 터미널 안에서 키보드로 조작하는 전체화면 텍스트
> UI / **CLI**(Command-Line Interface) = 명령 한 줄로 실행해 결과를 출력하는 방식 / **패널(panel)** =
> 화면을 나눈 영역 / **탭(tab)** = 한 패널 안에서 전환되는 하위 화면 / **모달 다이얼로그(modal dialog)** =
> 떠서 다른 입력을 막고 결정을 받는 작은 창 / **포커스(focus)** = 지금 키 입력을 받는 패널 / **graceful
> 종료(정상 종료)** = "정리하고 꺼져라"라고 **요청**(SIGTERM)하는 것, 강제 종료(SIGKILL)와 다름 /
> **에스컬레이션(escalation)** = 정상 요청이 안 통하면 강제로 **단계 올림** / **IA**(Information
> Architecture, 정보구조) = 화면·기능이 어떻게 묶이고 이동하는지의 뼈대.

---

## Sprint Contract (self-proposed checks)

이번 스프린트(S3)에서 만족시킬 **관찰 가능한 체크**(playbook §S3 + spec §4-0/§4-2에서 도출). Evaluator는
승인 또는 보강 후 이 기준으로 평가한다. 본 산출물은 후속 `12-ia`·`13-user-flows`·`20-wireframes`·
`21-screen-spec`의 토대이며, 게이트 **G-g**(인포그래픽 우선)·**P-1**(와이어프레임 휴먼 체크포인트)에 공급된다.

| # | 체크 | 어떻게 충족하나 | 본문 위치 |
|---|---|---|---|
| **E-1** | **정보구조(IA)** — TUI 레이아웃 계층 + **CLI 명령 트리**를 **ASCII IA 트리**로(역할/색 주기) | TUI(상단바/리스트/상세4탭/다이얼로그5종/필터모드/빈상태) + CLI(`ntop`+`list`/`kill`/`info`/`log`/`config`) 트리 + 색·역할 범례 | §2 |
| **E-2** | **핵심 유저 플로우**(6종) ASCII 플로우차트 + 각 단계에 **화면 ID·키·PRB**(research-s2) 링크 | F1 기동→스캔(2-pass)→선택→상세4탭→종료 / F2 필터·정렬·Node-only / F3 kill 단건 graceful→force / F4 kill 트리 / F5 SignalPicker / F6 CLI `list --json`→`kill --tree` | §3 |
| **E-3** | **핵심 세션 시퀀스 + 상태 다이어그램** — kill graceful→force 에스컬레이션(ASCII) | 상태기계(SIGTERM→폴링→Terminated/TimedOut→SIGKILL) + 시퀀스(User↔App↔ProcessKiller↔OS↔Process), TUI/CLI 차이 표기 | §4 |
| **E-4** | **화면/뷰 인벤토리** — 모든 TUI 뷰/패널/탭/다이얼로그 + 빈/에러 상태 열거(화면 ID 부여) | 전 화면 표(ID·역할·위젯파일·진입키·연결 PRB) | §5 |
| **E-5** | **핵심 화면 4상태**(empty/loading/normal/error) 식별 — empty=프로세스 없음, loading=스캔중 스피너 tick_count, error=권한거부 EPERM/명령부재 lsof | 핵심 뷰별 4상태 매트릭스 + 트리거·근거 | §6 |
| **E-6** | **키 바인딩 카탈로그**(전역/리스트/상세/필터/다이얼로그) — 도시에 §6 + 코드 권위값 그대로(`21`의 정규 출처) | 5그룹 키표(키·동작·코드근거) + 컨텍스트 의존 바텀바 | §7 |
| **E-7** | **저충실도 와이어프레임** — 화면당 1 ASCII 터미널 프레임(메인·상세4탭·다이얼로그5·필터·빈상태) → **HUMAN_CHECKPOINT_REQUIRED** | §8 (12종 시안) | §8 |
| **E-8** | **디자인 품질 노트**(4-D) — 수동 모드선택 벽·중복 단계·계정파생 입력 강요 없음 확인, 결함 플래그 | UX 기본기 점검 + 경계 사례 정직 플래그 | §9 |

> **사실 정정 노트(중요, P-1 휴먼 체크포인트로 회부):** 도시에 §6은 메인 레이아웃을
> "main: [process_list(좌 55%) | detail_panel(우 45%)]"(좌/우 가로 분할)로 적었으나, **실제 코드
> `src/tui/ui.rs` L37–44는 `Layout::vertical([55%, 45%])`로 process_list를 위(55% 높이)·detail_panel을
> 아래(45% 높이)에 두는 상/하 세로 분할**이다(코드 주석: "Vertical split: 55% process list (top), 45%
> detail panel (bottom)"). 본 문서는 spec/playbook이 "도시에 **+ 실제 소스코드**"를 공동 사실 원천으로
> 규정(spec line 5, playbook line 8)하고 **코드를 지상 진실로** 보므로, **상/하 세로 분할을 채택**한다.
> 도시에 §6의 좌/우 표기는 부정확 → 메인테이너가 도시에를 정정하도록 §10에서 다시 플래그한다.

---

## 1. 한눈에 보는 결론 (Executive snapshot)

ntop의 UX는 **단일 TUI 한 화면 + 5종 모달 + 5종 CLI 서브커맨드**로 끝난다. 별도 설정 화면·로그인·모드
선택 마법사가 **없다** — 이것이 의도된 저마찰 설계다(§9).

- **IA는 평평하다(flat).** 메인 뷰(상단바 + 위 프로세스 목록 + 아래 상세 4탭 + 하단바) 하나에서 거의
  모든 일이 끝나고, **위험한 동작(종료·시그널)만 모달**로 한 단계 확인을 둔다. 이동 깊이가 얕아
  research-s2의 PRB-8(GUI↔터미널 왕복)을 구조적으로 없앤다.
- **플로우의 척추는 "식별 → 점검 → 안전 종료"** 다(research-s2 §5의 3대 능력군과 1:1). 스캔은
  **2-pass**(분류 서버 + 트리 부모 backfill)로 트리를 만들고, 종료는 **graceful→force 에스컬레이션**과
  **트리 역순 종료**로 안전을 보장한다.
- **상태 설계가 검소하다.** ntop은 빈 상태와 로딩 상태를 **하나의 위젯**(`empty_state`, 스피너
  `tick_count` 애니메이션)으로 합쳐, "스캔 중"과 "서버 없음"을 같은 자리에서 보여준다(§6). 에러는
  대개 **빈 결과로 수렴**(lsof/netstat 부재 → 포트 빈칸)하거나 **다이얼로그 결과 메시지**(EPERM →
  PermissionDenied)로 노출된다.
- **키 바인딩은 컨텍스트 의존.** 같은 키라도 포커스(리스트/상세)·모드(필터)·모달(다이얼로그)에 따라
  의미가 갈리고, **하단바가 그 순간 쓸 수 있는 키만** 실시간으로 보여준다(§7, `status_bar.rs`).

---

## 2. 정보구조(IA) — ASCII IA 트리 (E-1)

> ntop의 인터페이스는 **두 진입점**으로 갈린다: 서브커맨드 없이 `ntop` → **TUI**, 서브커맨드 있으면
> **CLI 일회성 출력**(`src/main.rs` clap 분기). 아래는 두 갈래를 합친 IA 트리다. 색/역할은 범례 참조
> (색은 `src/tui/widgets/status_bar.rs`·`empty_state.rs`의 실제 `Color::*` 사용에 근거).

### 2-1. 색·역할 범례 (코드 근거)

```
역할 표기                                    │ 색(코드 근거)
─────────────────────────────────────────────┼──────────────────────────────────────
[크롬]  상시 노출 막대(상단/하단 바)          │ 배경 DarkGray · 글자 White (status_bar.rs)
[모드]  키 입력 해석을 바꾸는 상태(필터)      │ 강조 Yellow (filter_text 커서 █)
[패널]  포커스 가능 영역(리스트/상세)         │ 활성 테두리 강조
[탭]    상세 패널 하위 화면(Info/Log/Net/Env) │ 활성 탭 Yellow 강조
[모달]  입력 막는 확인창(다이얼로그 5종)      │ 위험=Red(kill), 선택=Yellow
[빈/로딩] 프로세스 0건 안내 + 스피너          │ 스피너 Cyan (empty_state.rs)
색 의미: Cyan=브랜드/스피너 · Green=정상(CPU<50) · Yellow=강조/주의(CPU>50·키힌트·Node-only)
         Red=위험(CPU>80·파괴적 kill) · DarkGray=크롬배경/비활성 (status_bar.rs 임계값 실측)
```

### 2-2. IA 트리 (TUI + CLI 통합)

```
ntop (바이너리 진입점 · src/main.rs clap 분기)
│
├─[A] 서브커맨드 없음 → TUI 기동 (run_tui)  ........................ 화면 ID 접두 V-/BAR-/TAB-/DLG-
│   │   (터미널 셋업: raw_mode → AlternateScreen → 루프; 해제 시 복원 — 도시에 §7)
│   │
│   ├─ BAR-TOP  [크롬] 상단 상태바 (1줄, status_bar.rs render_top_bar)
│   │     └ "ntop v{버전} | CPU:x% | MEM:used/totalMB | Servers/Nodes:n [Node-only] | Refresh:s | [H]elp"
│   │       (CPU 색: >80 Red · >50 Yellow · else Green)
│   │
│   ├─ V-MAIN  메인 콘텐츠 (main_content, ui.rs L19) ── 둘 중 하나로 렌더:
│   │   │
│   │   ├─ (flat_list 비었으면) V-EMPTY  [빈/로딩] empty_state.rs
│   │   │       └ 스피너(Cyan, tick_count) + "Scanning…" + "No server processes found. Waiting…"
│   │   │
│   │   └─ (아니면) 상/하 세로 분할 (ui.rs L38 Layout::vertical 55/45) ── ※§0 정정 노트
│   │       │
│   │       ├─ V-LIST  [패널·포커스1] 프로세스 목록(트리)  (위 55%, process_list.rs)
│   │       │     └ 컬럼: [✓/health] PID·NAME·PORT·THR·CPU·MEM·USER·STATUS·UPTIME
│   │       │       (정렬 표시자 ↑↓ · 트리 들여쓰기 ▾/▸ · Space 체크)
│   │       │       └ PRB-1 식별 · PRB-2 포트 · PRB-4 멀티런타임/Node-only
│   │       │
│   │       └─ V-DETAIL  [패널·포커스2] 상세 패널 " Details "  (아래 45%, detail_panel.rs)
│   │             └ 탭바(Tabs) + 콘텐츠. 활성 탭 순환 Info→Log→Net→Env
│   │               ├─ TAB-INFO  PID/Framework/Port/CPU/Memory/Uptime…  (info_tab.rs)  PRB-1·6
│   │               ├─ TAB-LOG   감지 로그파일 실시간 tail            (log_tab.rs)
│   │               ├─ TAB-NET   LISTEN 포트 + 활성 TCP 연결          (net_tab.rs)    PRB-2
│   │               └─ TAB-ENV   환경변수(민감값 마스킹)             (env_tab.rs)    PRB-6
│   │
│   ├─ BAR-BOTTOM  [크롬] 하단 키힌트 바 (1줄, render_bottom_bar) ── 컨텍스트 의존(§7)
│   │     └ 리스트포커스 / 상세포커스 / 필터활성 / 다이얼로그 별로 다른 힌트
│   │
│   ├─ V-FILTER  [모드] 필터 입력 (filter_active, 하단바에 " /{텍스트}█" 표시)  PRB-1·4
│   │     └ 입력마다 rebuild_view() 실시간 갱신 (도시에 §6)
│   │
│   └─ 모달 다이얼로그 (DialogKind, ui.rs 위에 오버레이) ── 한 번에 하나만
│         ├─ DLG-KILL      [모달·위험] KillConfirm     ← x      → SIGTERM        PRB-3·5
│         ├─ DLG-KILLTREE  [모달·위험] KillTreeConfirm ← K      → kill_tree      PRB-5
│         ├─ DLG-SIGNAL    [모달]      SignalPicker    ← S      → 임의 시그널    PRB-5
│         ├─ DLG-FORCE     [모달·위험] ForceKillPrompt ← graceful timeout 후     PRB-5
│         └─ DLG-HELP      [모달]      Help            ← H      → 키 도움말
│
└─[B] 서브커맨드 있음 → CLI 일회성 (scan_blocking, 도시에 §7) ......... 화면 ID 접두 CLI-
    ├─ CLI-LIST   `ntop list [--json] [--format <table|csv|json>]`   분류 서버 목록   PRB-1·7
    │     └ table: PID·NAME·FRAMEWORK·PORT·CPU·MEM·UPTIME / json: 전필드+depth / csv: 9컬럼
    ├─ CLI-KILL   `ntop kill <PID> [--tree] [--signal <SIG>] [--all] [--no-confirm]`  PRB-5·7
    │     └ graceful(Term→timeout→force) / tree(재귀수집→확인→각각) / all(전 서버)
    ├─ CLI-INFO   `ntop info <PID>`        단일 프로세스 상세(메트릭/포트/health/env/net)  PRB-1·6
    ├─ CLI-LOG    `ntop log <PID>`         cwd 로그 자동감지 → 200ms 폴링 스트림
    └─ CLI-CONFIG `ntop config`            설정 파일 경로 + [general][display][filter] 출력
```

**IA가 말하는 것:** 깊이 2~3단계로 끝나는 **얕은 트리**다. TUI는 메인 한 화면에 정보를 모으고
파괴적 동작만 모달 한 겹으로 감싼다. CLI는 TUI와 **동일 코어**(ProcessScanner+NetworkInspector)를
재사용하는 평면 5개 명령이다(도시에 §7). 별도 라우팅·페이지 전환·설정 마법사가 없어 PRB-8(도구 왕복)을
구조적으로 제거한다.

---

## 3. 핵심 유저 플로우 (E-2) — ASCII 플로우차트

> 각 플로우는 **단계 → 화면 ID → 키 → PRB(research-s2)** 로 라벨링한다. 분기(branch)는 `◇`로 표기.
> PRB 매핑은 research-s2 §3(PRB-1~8)·§5(능력군) 기준.

### 3-1. F1 — 기동 → 스캔(2-pass) → 선택 → 상세 4탭 점검 → 종료  (PRB-1·2·6·8)

```
[기동]                                          [스캔: 2-pass + 포트]
$ ntop ──▶ 터미널 셋업(raw/AltScreen) ──▶ 이벤트 루프 ──Tick──▶ do_scan()
  CLI-TUI    (main.rs run_tui)                  (event.rs)        │
                                                                  ▼
                                  Pass1: 전체 열거→classify→서버(runtime=Some) 수집
                                  Pass2: 서버의 부모(launchd/claude/shell) backfill(runtime=None)
                                  포트: NetworkInspector LISTEN만 PID별 채움 (도시에 §4·§4-1)
                                                                  │
                          ┌──────────────◇ flat_list 비었나? ─────┤
                          │ 예                                  아니오
                          ▼                                       ▼
                  V-EMPTY (스피너 "Scanning…")          V-LIST 트리 표시 (first_load면 전체 확장)
                  PRB-1 대기                            상단바 Servers:n  ← PRB-1 의미 식별 시작
                          │  (서버 뜨면 다음 Tick에)              │
                          └───────────────────────────────────────┤
                                                                   ▼
[선택]  V-LIST 포커스 ─ ↑↓/jk 이동 · Enter 트리 토글 · →/l 확장 · ←/h 축소 · Space 다중선택
                                                                   │  Tab
                                                                   ▼
[상세 점검]  V-DETAIL 포커스 ─ Tab/→/l 다음탭 · BackTab/←/h 이전탭
        ┌─────────────┬─────────────┬─────────────┬─────────────┐
        ▼             ▼             ▼             ▼
     TAB-INFO      TAB-LOG       TAB-NET       TAB-ENV
     프레임워크·    실시간 로그   LISTEN 포트·  환경변수
     CPU·메모리     tail          TCP 연결      (마스킹)
     PRB-1·6                      PRB-2         PRB-6
        │             │             │             │
        └─────────────┴──────┬──────┴─────────────┘   (Esc → V-LIST 복귀)
                             ▼
[종료]  q 또는 Ctrl+C ──▶ should_quit ──▶ 터미널 복원(raw 해제·AltScreen 나감·커서 표시)  PRB-8 일원화
```

### 3-2. F2 — 필터(`/`) / 정렬(`s`,`r`) / Node-only(`n`) 토글  (PRB-1·4)

```
 V-LIST (normal)
   │
   ├── / ─▶ V-FILTER [모드] ─ Char 입력마다 rebuild_view() 실시간 ─┐
   │          하단바 " /{텍스트}█  [Enter]Apply [Esc]Cancel"      │
   │          매칭: name·command·pid·framework·runtime·ports 부분일치(대소문자무시)
   │                                                              │
   │        ◇ 종료 방식                                           │
   │        ├ Enter ─▶ 필터 유지하고 normal 복귀  (좁혀진 목록 유지)
   │        └ Esc   ─▶ filter_text 비우고 normal 복귀 (전체로)    │
   │                                                  PRB-1 식별 좁힘 ◀┘
   │
   ├── s ─▶ sort_column 순환 (PID→NAME→PORT→THR→CPU→MEM→USER→STATUS→UPTIME) ─▶ rebuild_view()
   ├── r ─▶ sort_ascending 반전 (↑/↓) ─────────────────────────────────────▶ rebuild_view()
   │
   └── n ─▶ node_only 토글 ─▶ rebuild_view()
            (Node 서버만 + 트리부모 유지; Deno/Bun은 숨김) 상단바 라벨 Servers↔Nodes·[Node-only]
                                                          PRB-4 멀티런타임↔Node집중
```

### 3-3. F3 — kill 단건 (`x`, KillConfirm → SIGTERM → graceful → force)  (PRB-3·5)

```
 V-LIST/V-DETAIL ─ x ─▶ DLG-KILL (KillConfirm) [모달·위험]
                          │ "대상 프로세스/포트 표시 + [Enter]Confirm [Esc]Cancel"
              ┌───────────◇ 사용자 결정
              │ Esc                         │ Enter
              ▼                             ▼
          취소(원상)              send_signal(pid, SIGTERM)  + kill_in_progress=(pid, now)
                                            │
                                            ▼  (이후 Tick마다 is_alive 폴링)
                              ◇ graceful_timeout(기본10s) 안에 죽었나?
                       ┌──────────────┬──────────────────┬──────────────────┐
                       │ 예(Terminated)│ EPERM            │ 아니오(TimedOut)  │
                       ▼              ▼                  ▼
                    정리 완료     PermissionDenied    DLG-FORCE (ForceKillPrompt) [모달·위험]
                    (목록 갱신)   메시지 노출           │ "안 죽음 — 강제 종료?"
                                  (권한 부족)           ◇ Enter→force_kill(SIGKILL) / Esc→취소
                                                        ▼
                                                   SIGKILL(못 잡힘) → 즉시 종료 → 정리
   ↳ 안전장치: 확인 게이트(PRB-3 오kill 방지) + graceful 우선(PRB-5 데이터 유실 방지)
```

### 3-4. F4 — kill 트리 (`K`, KillTreeConfirm)  (PRB-5)

```
 V-LIST ─ K ─▶ DLG-KILLTREE (KillTreeConfirm) [모달·위험]
                 │ "부모+자식 전체 트리 PID 표시 + [Enter]Confirm [Esc]Cancel"
        ┌────────◇ 결정
        │ Esc                  │ Enter
        ▼                      ▼
     취소              kill_tree(pids, SIGTERM)
                          │  pids를 역순(자식부터) 시그널 → Vec<(pid, KillResult)>
                          ▼
                  자식 먼저 정리 → 부모 정리 ⇒ 고아 프로세스 없음 (PRB-5 고아 방지)
```

### 3-5. F5 — SignalPicker (`S`)로 임의 시그널  (PRB-5)

```
 V-LIST/V-DETAIL ─ S ─▶ DLG-SIGNAL (SignalPicker) [모달]
                          │ 시그널 목록(KillSignal::all):
                          │   Unix 6종: Term · Kill · Hup · Int · Usr1 · Usr2
                          │   Windows 3종: Term · Kill · Int   (HUP/USR1/USR2 없음 — 도시에 §12)
                          │   각 항목에 설명(예 Hup="reload", Usr1="Node 디버거 활성")
                  ┌───────◇ 조작
                  │ Up/k·Down/j 이동                │ Esc 취소
                  ▼                                 ▼
            Enter ─▶ send_signal(pid, 선택시그널)  원상
                     (예 SIGHUP=설정 리로드, SIGUSR1=디버거 — graceful/force 대신 정밀 제어)
```

### 3-6. F6 — CLI: `list --json` → `kill --tree <PID>`  (PRB-7 헤드리스·자동화)

```
[자동화/SSH 헤드리스]                              (research-s2 P2 운영자 시나리오)
$ ntop list --json
  CLI-LIST ─▶ scan_blocking()(sleep+scan 1회) ─▶ classify ─▶ NetworkInspector
            ─▶ JSON 출력(모든 필드 + depth)
                          │  jq 등으로 파싱 → 대상 PID 추출
                          ▼
$ ntop kill --tree <PID> [--no-confirm]
  CLI-KILL ─▶ 재귀로 트리 PID 전부 수집 ─▶ ◇ 확인 프롬프트(--no-confirm이면 생략)
            ─▶ 각 PID graceful(Term→timeout→force) ─▶ 종료 결과 출력 → 종료코드
   ↳ 동일 코어(ProcessScanner+NetworkInspector)를 TUI와 공유 → 스크립트에 그대로 연결(PRB-7)
```

**플로우가 말하는 것:** 6개 플로우 모두 research-s2의 **식별→포트→안전 종료** 가치 루프 위에 있다.
F1·F2는 식별(PRB-1·2·4·6·8), F3·F4·F5는 안전 종료(PRB-3·5), F6은 헤드리스 자동화(PRB-7)를 각각
완결한다. **위험한 분기(kill·signal)에는 반드시 모달 확인 또는 graceful 우선**이 끼어 있어, 단계가
"군더더기"가 아니라 **안전 게이트**다(§9).

---

## 4. 핵심 세션 — kill graceful→force 시퀀스/상태 다이어그램 (E-3)

> ntop의 가장 위험하고 중요한 세션 = **종료 에스컬레이션**. TUI와 CLI가 **같은 의미**(SIGTERM 우선 →
> 안 죽으면 SIGKILL)지만 **에스컬레이션을 트리거하는 방식이 다르다**: CLI는 timeout 후 **자동**으로
> force_kill, TUI는 timeout 후 **DLG-FORCE로 사용자 확인**을 받는다(도시에 §5·§7, killer.rs).

### 4-1. 상태 다이어그램 (state machine)

```
                       (x / kill <PID>)
        ┌─────────┐  사용자 종료 의도   ┌──────────────┐
        │  Idle   │ ──────────────────▶ │  Confirming  │  TUI=DLG-KILL / CLI=확인 프롬프트
        └─────────┘                     └──────┬───────┘
             ▲  Esc/취소 ◀──────────────────────┤ Enter/y (또는 --no-confirm)
             │                                  ▼
             │                         ┌─────────────────────┐  send_signal(SIGTERM)
             │                         │   Sending SIGTERM   │  + kill_in_progress 기록
             │                         └──────────┬──────────┘
             │                                    ▼
             │                         ┌─────────────────────────────┐
             │                         │  Polling (is_alive, 200ms)  │◀─┐ 아직 살아있고
             │                         │  until graceful_timeout(10s)│  │ timeout 전
             │                         └──┬───────┬───────────┬──────┘──┘
             │            (죽음)          │       │(EPERM)    │ (timeout 초과, 아직 살아있음)
             │   ┌────────────────────────┘       ▼           ▼
             │   ▼                          ┌──────────────┐  ┌─────────────────────────┐
             │ ┌────────────┐               │PermissionDenied│ │   TimedOut (escalate)   │
             │ │ Terminated │               │ (권한거부 표시) │ └───────────┬─────────────┘
             │ │ (정리완료) │               └──────┬─────────┘             │
             │ └─────┬──────┘                      │                       ▼
             │       │                             │            TUI: DLG-FORCE 확인  │ CLI: 자동
             │       │   AlreadyDead(ESRCH)도 여기 │            ┌──────────◇─────────┘
             │       ▼                             │            │ Enter/자동      │ Esc(TUI만)
             └───────┴─────────────────────────────┴───────┐    ▼                 ▼
                              (목록 갱신/needs_rescan)       │ force_kill(SIGKILL)  취소→여전히 실행
                                                            │      │ (못 잡힘 = 즉사)
                                                            └──────┤
                                                                   ▼
                                                              Terminated → Idle
```

`GracefulResult` enum 매핑(killer.rs, 도시에 §5): **Terminated**(timeout 내 사망) · **TimedOut**(살아남음→
에스컬레이션) · **AlreadyDead**(이미 죽음, ESRCH) · **PermissionDenied**(EPERM) · **Error**(기타).

### 4-2. 시퀀스 다이어그램 (actor 간 메시지)

```
 User      App(ui.rs/main.rs)     ProcessKiller(killer.rs)     OS(nix kill / taskkill)   Process
  │              │                        │                            │                   │
  │── x ────────▶│ DLG-KILL 표시          │                            │                   │
  │── Enter ────▶│ send_signal(pid,Term) ▶│ signal::kill(pid,SIGTERM) ▶│── SIGTERM ───────▶│ (정리 시도)
  │              │ kill_in_progress=now   │◀ KillResult::Success ──────│                   │
  │              │                        │                            │                   │
  │              │ (Tick 루프 200ms 간격)  │                            │                   │
  │              │── is_alive(pid)? ──────▶│ signal::kill(pid, 0) ─────▶│── 존재확인 ──────▶│
  │              │◀── alive=true/false ───│◀───────────────────────────│                   │
  │              │                        │                            │                   │
  │   [경우 A: timeout 내 사망]            │                            │                   │
  │◀ 목록서 사라짐(Terminated) ───────────│                            │              (종료됨)✗
  │                                                                                          │
  │   [경우 B: graceful_timeout 초과·생존 → 에스컬레이션]                                      │
  │◀ DLG-FORCE("강제 종료?") ─────────────│  (CLI는 이 단계 자동, 사용자 확인 없음)            │
  │── Enter ────▶│ force_kill(pid) ───────▶│ signal::kill(pid,SIGKILL) ▶│── SIGKILL(못잡힘)─▶│ (즉사)✗
  │◀ 정리완료/needs_rescan ────────────────│◀ KillResult::Success ──────│                   │
  │                                                                                          │
  │   [경우 C: 권한거부]   send_signal → EPERM ─▶ KillResult::PermissionDenied ─▶ 사용자에 표시 │
  │   [Windows]            Term/Int→`taskkill /PID` · Kill→OpenProcess+TerminateProcess (도시에 §5)
```

**이 세션이 안전한 이유(PRB-3·5 해소):** ① **확인 게이트**가 오발사(PRB-3)를 막고, ② **SIGTERM 우선 +
200ms 폴링 + timeout 대기**가 프로세스에 정리 시간을 줘 데이터 유실(PRB-5)을 막으며, ③ **TUI는 강제
종료(SIGKILL, 못 잡힘)를 한 번 더 확인**(DLG-FORCE)해 "되돌릴 수 없는 즉사"를 의식적 결정으로 만든다.
트리 종료(F4)는 **자식부터 역순**이라 고아 프로세스가 안 남는다.

---

## 5. 화면/뷰 인벤토리 (E-4)

> 모든 TUI 뷰/패널/탭/다이얼로그 + 빈/에러 상태를 화면 ID와 함께 열거한다. 이 표가 후속
> `21-screen-spec`의 화면 목록 정규 출처가 된다. "연결 PRB"는 research-s2 §3 기준.

| 화면 ID | 종류 | 역할(한 줄) | 위젯/코드 | 진입(키/조건) | 연결 PRB |
|---|---|---|---|---|---|
| **BAR-TOP** | 크롬 | 시스템 CPU/MEM·서버 수·refresh·[H]elp 상시 표시 | `status_bar.rs::render_top_bar` | 상시 | PRB-6(시스템 부하 맥락) |
| **BAR-BOTTOM** | 크롬 | 그 순간 쓸 수 있는 키힌트(컨텍스트 의존) | `status_bar.rs::render_bottom_bar` | 상시 | 전반(발견성) |
| **V-MAIN** | 합성뷰 | 상단바+목록+상세+하단바 조합 화면 | `ui.rs::render` | TUI 기동 | PRB-1·2·6·8 |
| **V-LIST** | 패널(포커스1) | 분류된 서버 프로세스 트리 목록 | `process_list.rs` | 기본 포커스 | PRB-1·2·4 |
| **V-DETAIL** | 패널(포커스2) | 선택 프로세스 상세(4탭 컨테이너) | `detail_panel.rs` | `Tab`(리스트→상세) | PRB-1·2·6 |
| **TAB-INFO** | 탭 | PID/Framework/Port/CPU/Memory/Uptime 메트릭 | `info_tab.rs` | 상세 기본 탭 | PRB-1·6 |
| **TAB-LOG** | 탭 | 감지된 로그파일 실시간 tail | `log_tab.rs` | 탭 순환 | (점검) |
| **TAB-NET** | 탭 | LISTEN 포트 + 활성 TCP 연결 | `net_tab.rs` | 탭 순환 | PRB-2 |
| **TAB-ENV** | 탭 | 환경변수(민감값 자동 마스킹) | `env_tab.rs` | 탭 순환 | PRB-6 |
| **V-FILTER** | 모드 | 부분일치 필터 입력(실시간 갱신) | `ui.rs::handle_filter_key`+하단바 | `/` | PRB-1·4 |
| **V-EMPTY** | 빈/로딩 | 프로세스 0건 + 스피너("Scanning…") | `empty_state.rs` | `flat_list` 비었을 때 | PRB-1(대기) |
| **DLG-KILL** | 모달·위험 | 단건 종료(SIGTERM) 확인 | `kill_dialog.rs` | `x` | PRB-3·5 |
| **DLG-KILLTREE** | 모달·위험 | 트리 종료(부모+자식) 확인 | `kill_dialog.rs` | `K` | PRB-5 |
| **DLG-SIGNAL** | 모달 | 임의 시그널 선택 전송 | `signal_picker.rs` | `S` | PRB-5 |
| **DLG-FORCE** | 모달·위험 | graceful 실패 후 강제(SIGKILL) 확인 | `kill_dialog.rs` | graceful timeout 후 | PRB-5 |
| **DLG-HELP** | 모달 | 키 바인딩 도움말(스크롤) | `help_dialog.rs` | `H` | (발견성) |
| **CLI-LIST** | CLI 출력 | 분류 서버 목록(table/json/csv) | `main.rs cmd_list` | `ntop list …` | PRB-1·7 |
| **CLI-KILL** | CLI 출력 | 종료(graceful/tree/all) | `main.rs cmd_kill` | `ntop kill …` | PRB-5·7 |
| **CLI-INFO** | CLI 출력 | 단일 프로세스 상세 | `main.rs cmd_info` | `ntop info <PID>` | PRB-1·6 |
| **CLI-LOG** | CLI 출력 | 로그 스트림(폴링) | `main.rs cmd_log` | `ntop log <PID>` | (점검) |
| **CLI-CONFIG** | CLI 출력 | 설정 경로 + 값 출력 | `main.rs cmd_config` | `ntop config` | (설정 확인) |

**합계:** TUI = 크롬 2 + 패널/뷰 4(V-MAIN/LIST/DETAIL + 탭 컨테이너) + 탭 4 + 모드 1 + 빈/로딩 1 +
모달 5 = **고유 화면 12종(탭 4 별도 집계 시 16종)**; CLI = **5종**. 별도 설정/로그인/온보딩 화면 **없음**.

---

## 6. 핵심 화면 4상태 (E-5) — empty / loading / normal / error

> playbook §S3: empty=프로세스 없음, loading=스캔 중 스피너(tick_count), normal, error=권한거부
> (EPERM)/명령부재(lsof·netstat). **중요 사실(코드 검증):** ntop은 **empty와 loading을 하나의 위젯
> `empty_state.rs`로 합쳤다** — `flat_list`가 비면 스피너(`tick_count`, Braille `⠋⠙⠹…`)가 돌며
> "Scanning for server processes… / No server processes found. Waiting…"를 같이 보여준다. 즉 "스캔 중"과
> "결과 없음"이 **같은 자리·같은 위젯**이고, 스피너 애니메이션이 살아있음(loading)을 표현한다.

### 6-1. 4상태 매트릭스 (핵심 뷰별)

```
 화면      │ empty(프로세스 없음)      │ loading(스캔 중)          │ normal                  │ error
 ──────────┼──────────────────────────┼──────────────────────────┼─────────────────────────┼───────────────────────────
 V-LIST    │ V-EMPTY로 대체:          │ V-EMPTY와 동일 위젯,      │ 트리 목록(컬럼·정렬·    │ (스캔 자체 실패는 빈
 (목록)    │ "No server processes     │ 스피너 tick_count 회전 +  │ 확장/선택) 정상 렌더     │ 결과로 수렴 → empty와
           │ found. Waiting…"         │ "Scanning…"              │                         │ 동일 화면)
 ──────────┼──────────────────────────┼──────────────────────────┼─────────────────────────┼───────────────────────────
 TAB-NET   │ "LISTEN 포트 없음"       │ 직전 스캔 값 유지         │ 포트 + TCP 연결 목록     │ ★lsof/netstat 부재 →
 (포트)    │ (서버지만 안 듣는 중)    │ (포트는 별도 채움)        │                         │ 빈 결과(포트 0건)
           │                          │                          │                         │ (도시에 §12)
 ──────────┼──────────────────────────┼──────────────────────────┼─────────────────────────┼───────────────────────────
 TAB-LOG   │ "로그파일 미감지"        │ tail 시작 대기            │ 새 줄 실시간 tail        │ 파일 접근 실패/권한 →
 (로그)    │ (cwd glob 0건)           │                          │ (MAX_BUFFER 1000줄)     │ fallback(/proc/<pid>/fd/1)
 ──────────┼──────────────────────────┼──────────────────────────┼─────────────────────────┼───────────────────────────
 TAB-ENV   │ "환경변수 없음"          │ —                        │ KEY=값(민감값 마스킹)    │ 권한 부족 시 일부 미수집
 (환경)    │                          │                          │ mask_env_values         │
 ──────────┼──────────────────────────┼──────────────────────────┼─────────────────────────┼───────────────────────────
 DLG-KILL/ │ —                        │ kill_in_progress 동안     │ 확인 프롬프트            │ ★EPERM →
 DLG-FORCE │                          │ "종료 대기…"(폴링)        │ (대상·포트 표시)        │ PermissionDenied 메시지
 ──────────┴──────────────────────────┴──────────────────────────┴─────────────────────────┴───────────────────────────
 ★ = playbook이 명시한 핵심 에러 트리거(권한거부 EPERM · 명령부재 lsof/netstat).
```

### 6-2. 상태 전이 (V-MAIN 레벨)

```
 (앱 시작)
     │
     ▼
  [loading]  V-EMPTY 스피너 "Scanning…"  ──첫 스캔 완료──┐
     │                                                  │
     │  서버 0건                                서버 ≥1건 │
     ▼                                                  ▼
  [empty]  V-EMPTY "No server… Waiting…" ◀──서버 모두 종료── [normal]  V-LIST + V-DETAIL
     │  (스피너 계속 회전)            서버 다시 뜸 ──────▶    │
     └───────────────────────────────────────────────────────┤
                                                              │ 부분 에러(lsof 부재/EPERM)
                                                              ▼
                                                    [error(부분)]  화면은 normal,
                                                    해당 탭/다이얼로그에만 빈 결과·메시지
```

**상태 설계의 의미:** ntop은 "전체 에러 화면"을 거의 두지 않는다. 에러는 **빈 결과로 수렴**(lsof 없으면
포트가 그냥 빈칸)하거나 **국소 메시지**(EPERM → PermissionDenied)로 노출돼, 사용자는 항상 메인 뷰에
머문다. 이는 research-s2 PRB-8(문맥 전환)을 상태 차원에서도 줄이는 선택이다.

---

## 7. 키 바인딩 카탈로그 (E-6) — `21-screen-spec`의 정규 출처

> 도시에 §6 **+ 실제 코드**(`src/tui/ui.rs`, `status_bar.rs`) 권위값을 그대로 옮긴다. **이 §7이 키
> 바인딩의 단일 출처(single source)** 이며 `13`/`30`/`41`은 여기를 참조한다(spec §4-3). Ctrl+C는
> 어디서든 종료(ui.rs 최우선 처리).

### 7-1. 전역 (normal 모드 — 포커스 무관 공통)

| 키 | 동작 | 코드 근거 |
|---|---|---|
| `q`, `Ctrl+C` | 종료(should_quit) | ui.rs handle_normal_key / Ctrl+C 최우선 |
| `/` | 필터 모드 진입(filter_text 초기화) | ui.rs L239 |
| `s` | 정렬 컬럼 순환(toggle_sort) | ui.rs L245 |
| `r` | 정렬 방향 반전(sort_ascending) | ui.rs L249 |
| `+` | refresh 간격 +1s (최대 60s) | 도시에 §6 |
| `-` | refresh 간격 −1s (최소 1s) | 도시에 §6 |
| `x` | DLG-KILL(KillConfirm) 열기 | 도시에 §6 |
| `K` | DLG-KILLTREE(KillTreeConfirm) 열기 | 도시에 §6 |
| `H` | DLG-HELP 열기 | ui.rs L275 |
| `S` | DLG-SIGNAL(SignalPicker) 열기 | ui.rs L281 |
| `e` | 전체 확장/축소 토글(toggle_expand_all) | ui.rs L286 |
| `n` | Node-only 뷰 토글(toggle_node_only) | ui.rs L290 |

### 7-2. 리스트 포커스 (V-LIST · FocusPanel::ProcessList)

| 키 | 동작 |
|---|---|
| `Up`/`k` · `Down`/`j` | 위/아래 이동 |
| `PageUp` · `PageDown` | 10칸 위/아래 |
| `Home` · `End` | 처음 / 끝 |
| `Enter` | 트리 노드 확장/축소 토글 |
| `Right`/`l` | 확장(expand_selected) |
| `Left`/`h` | 축소(collapse_selected) |
| `Space` | 다중 선택 토글(selected_pids) |
| `Tab` | V-DETAIL로 포커스 이동 |
| `Esc` | 종료 |

### 7-3. 상세 포커스 (V-DETAIL · FocusPanel::DetailPanel)

| 키 | 동작 |
|---|---|
| `Tab` / `Right` / `l` | 다음 탭(Info→Log→Net→Env 순환) |
| `BackTab` / `Left` / `h` | 이전 탭 |
| `Up`/`k` · `Down`/`j` | 스크롤 ↑/↓ (detail + log) |
| `PageUp` · `PageDown` | −10 / +10 스크롤 |
| `Home` | 맨 위 |
| `Esc` | V-LIST로 복귀 |
| `x` · `S` | (상세에서도) 종료 / 시그널 — 하단바 노출(status_bar.rs) |

### 7-4. 필터 모드 (V-FILTER · filter_active)

| 키 | 동작 |
|---|---|
| `Char(c)` | 입력(매칭: name·command·framework·pid·ports, 대소문자 무시 부분일치) |
| `Backspace` | 한 글자 삭제 |
| `Enter` | 필터 **유지**하고 모드 종료(Apply) |
| `Esc` | 필터 **비우고** 모드 종료(Cancel) |
| (입력마다) | rebuild_view() 실시간 갱신 |

### 7-5. 다이얼로그 (모달별)

| 다이얼로그 | 키 → 동작 |
|---|---|
| **DLG-KILL** (KillConfirm) | `Enter`=SIGTERM 전송 · `Esc`=취소 |
| **DLG-KILLTREE** (KillTreeConfirm) | `Enter`=kill_tree(SIGTERM, 역순) · `Esc`=취소 |
| **DLG-SIGNAL** (SignalPicker) | `Up`/`k`·`Down`/`j`=이동 · `Enter`=선택 시그널 전송 · `Esc`=취소 |
| **DLG-FORCE** (ForceKillPrompt) | `Enter`=force_kill(SIGKILL) · `Esc`=취소 |
| **DLG-HELP** (Help) | `Esc`/`H`/`q`=닫기 · `Up`/`k`·`Down`/`j`·`PageUp`/`Down`·`Home`·`End`=스크롤 |

### 7-6. 컨텍스트 의존 하단바 (BAR-BOTTOM, render_bottom_bar 실측)

> 같은 화면이라도 **상황별로 다른 키힌트**가 뜬다. 발견성(discoverability)을 위해 그 순간 유효한 키만 보여준다.

```
 리스트 포커스 : [q]Quit | [Up/Down]Navigate | [PgUp/Dn]Page | [Enter]Expand | [Tab]Details
                 | [Space]Select | [/]Filter | [s]Sort | [x]Kill | [S]Signal
 상세 포커스   : [Esc]List | [Tab]Next Tab | [S-Tab]Prev Tab | [Up/Down]Scroll | [PgUp/Dn]Page
                 | [x]Kill | [S]Signal
 필터 활성     :  /{입력텍스트}█   | [Enter]Apply | [Esc]Cancel
 Kill 계열 모달:  [Enter]Confirm | [Esc]Cancel
 SignalPicker  :  [Up/Down]Select | [Enter]Send | [Esc]Cancel
 Help 모달     :  [Esc]Close
```

---

## 8. 저충실도 와이어프레임 (E-7) — HUMAN_CHECKPOINT_REQUIRED

> ⚠️ **HUMAN_CHECKPOINT_REQUIRED (P-1):** 와이어프레임의 시각 품질은 LLM 한계 영역이므로 **STEP 7
> 사용자 검수 대상**이다(채점 제외). 아래는 화면당 1개 **저충실도 ASCII 터미널 시안**으로, 컬럼/탭/키힌트
> 구성은 §2·§5·§7과 코드(process_list.rs 컬럼, status_bar.rs 바, detail_panel.rs 탭)에 근거한다.
> 레이아웃은 **상/하 세로 분할**(목록 위 55% · 상세 아래 45%, ui.rs L37–44 — §0 정정 노트)을 반영한다.
> 폭은 가독성을 위해 약 78칸 기준의 예시이며, 실제는 터미널 폭에 맞춰 ratatui가 신축·절단한다.

### 8-1. V-MAIN (메인 뷰: 트리 + 선택행 + 상세 Info 탭)

```
 ntop v0.2.0 | CPU: 23.4%  MEM: 8124/16384MB | Servers: 5 | Refresh: 3s | [H]elp     ← BAR-TOP
┌ Processes ──────────────────────────────────────────────────────────────────────┐
│   PID    NAME           PORT   THR  CPU↓   MEM     USER     STATUS  UPTIME         │
│ ▾ 932    claude         —      18   1.2%   210MB   seoyeon  Run     3h 5m          │ ← 트리부모(runtime None)
│ ▸▌41822  next-server    :3000  11   2.1%   128MB   seoyeon  Run     1h 2m  ◀═선택   │ ← 선택행 강조(▌)
│   41835  vite           —       7   0.4%   96MB    seoyeon  Run     1h 2m          │
│ ▸ 42013  uvicorn        :8000   5   5.0%   180MB   seoyeon  Run     58m            │
│ □ 42090  celery         —       3   0.0%   140MB   seoyeon  Run     58m           │ ← □ Space 다중선택칸
└──────────────────────────────────────────────────────────────────────────────────┘
┌ Details ── [Info] Log  Net  Env ──────────────────────────────────────────────────┐ ← V-DETAIL(아래45%)
│ PID        41822            Framework  Next.js (Node)                               │
│ Port       3000 (LISTEN)    CPU        2.1%        Memory  128.0 MB (phys_footprint)│
│ Uptime     1h 2m 5s         Threads    11          User    seoyeon   PPID  932      │
│ Status     Run · Healthy    Command    node node_modules/.bin/next dev              │
└──────────────────────────────────────────────────────────────────────────────────┘
 [q]Quit | [Up/Down]Navigate | [Tab]Details | [Space]Select | [/]Filter | [s]Sort | [x]Kill | [S]Signal  ← BAR-BOTTOM
```

### 8-2. TAB-INFO (상세 Info 탭)

```
┌ Details ── [Info] Log  Net  Env ──────────────────────────────────────────────────┐
│ PID         41822                                                                   │
│ Framework   Next.js  (Runtime: Node)                                               │
│ Port        3000 (LISTEN)                                                          │
│ CPU         2.1 %                                                                   │
│ Memory      128.0 MB   (macOS: phys_footprint = Activity Monitor 일치)            │
│ Threads     11          Open FDs  64                                               │
│ Uptime      1h 2m 5s    User      seoyeon                                          │
│ Status      Run         Health    Healthy                                          │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 8-3. TAB-LOG (상세 Log 탭)

```
┌ Details ──  Info [Log] Net  Env ──────────────────────────────────────────────────┐
│ source: /proj/web/.next/server/app/page.log   (가장 최근 수정 파일 tail)          │
│ 12:01:03  ✓ Compiled /app in 412ms                                                │
│ 12:01:09  GET /  200 in 34ms                                                       │
│ 12:01:11  GET /api/health 200 in 3ms                                              │
│ 12:01:18  ▌(새 줄 실시간 추가… 버퍼 최대 1000줄, Up/Down 스크롤)                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 8-4. TAB-NET (상세 Net 탭)

```
┌ Details ──  Info  Log [Net] Env ──────────────────────────────────────────────────┐
│ LISTEN                                                                              │
│   0.0.0.0:3000        (이 프로세스가 듣는 포트)                                    │
│ ESTABLISHED                                                                         │
│   127.0.0.1:3000  ←→  127.0.0.1:51544                                              │
│   127.0.0.1:3000  ←→  127.0.0.1:51547                                              │
│ ───────────────────────────────────────────────────────────────────────────────  │
│ ※ lsof(Unix)/netstat(Win) 부재 시 이 목록은 비어 보일 수 있음(error→빈 결과)      │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 8-5. TAB-ENV (상세 Env 탭)

```
┌ Details ──  Info  Log  Net [Env] ─────────────────────────────────────────────────┐
│ NODE_ENV       development                                                          │
│ PORT           3000                                                                 │
│ DATABASE_URL   postgres://****                    ← 민감값 마스킹                  │
│ API_SECRET     ********                            ← (mask_env_values 기본 true)    │
│ AWS_TOKEN      ********                                                             │
│ PATH           /usr/local/bin:/usr/bin:…                                           │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 8-6. DLG-KILL (KillConfirm 모달)

```
            ┌──────────── Confirm Kill ─────────────┐
            │                                        │
            │  Kill this process?                    │
            │    PID 41822  next-server              │
            │    Next.js  ·  :3000                   │
            │                                        │
            │  → SIGTERM (graceful). 10s 내 안 죽으면 │
            │    강제 종료 여부를 다시 묻습니다.      │
            │                                        │
            │     [Enter] Confirm     [Esc] Cancel   │
            └────────────────────────────────────────┘
```

### 8-7. DLG-KILLTREE (KillTreeConfirm 모달)

```
            ┌────────── Confirm Kill Tree ──────────┐
            │  Kill the whole process tree?          │
            │    ▾ 41822  next-server  (부모)         │
            │       41835  vite        (자식)         │
            │       41850  worker      (자식)         │
            │  자식부터 역순으로 SIGTERM → 고아 없음. │
            │     [Enter] Confirm     [Esc] Cancel   │
            └────────────────────────────────────────┘
```

### 8-8. DLG-SIGNAL (SignalPicker 모달)

```
            ┌──────────── Send Signal ──────────────┐
            │  대상: PID 41822 next-server           │
            │  > SIGTERM   Graceful termination       │ ← 현재 선택(Up/Down 이동)
            │    SIGKILL   Force kill (cannot catch)  │
            │    SIGHUP    Hangup / reload config     │
            │    SIGINT    Interrupt (like Ctrl+C)    │
            │    SIGUSR1   Node.js: activate debugger │
            │    SIGUSR2   User-defined signal        │
            │  (Windows는 TERM/KILL/INT 3종만)        │
            │   [Up/Down] Select  [Enter] Send  [Esc] │
            └────────────────────────────────────────┘
```

### 8-9. DLG-FORCE (ForceKillPrompt 모달)

```
            ┌─────────── Force Kill? ────────────────┐
            │  PID 41822 next-server 가 SIGTERM 후    │
            │  10초가 지나도 살아 있습니다.           │
            │                                        │
            │  강제 종료(SIGKILL)? — 못 잡히는 즉사,  │
            │  정리 작업 없이 종료됩니다.             │
            │                                        │
            │     [Enter] Force Kill   [Esc] Cancel  │
            └────────────────────────────────────────┘
```

### 8-10. DLG-HELP (Help 모달)

```
       ┌──────────────────── Help — Key Bindings ────────────────────┐
       │ Global   / filter   s sort   r reverse   +/- refresh         │
       │          x kill     K tree   S signal    e expand   n node   │
       │ List     ↑↓/jk move  Enter toggle  →/l expand  ←/h collapse  │
       │          Space select   Tab → details                        │
       │ Detail   Tab/→ next tab   S-Tab/← prev   ↑↓ scroll           │
       │ Quit     q / Ctrl+C                                           │
       │                                          (↑↓ scroll · [Esc])  │
       └──────────────────────────────────────────────────────────────┘
```

### 8-11. V-FILTER (필터 모드)

```
 ntop v0.2.0 | CPU: 23.4%  MEM: 8124/16384MB | Servers: 2 | Refresh: 3s | [H]elp
┌ Processes ──────────────────────────────────────────────────────────────────────┐
│   PID    NAME           PORT   THR  CPU    MEM     USER     STATUS  UPTIME         │
│   41822  next-server    :3000  11   2.1%   128MB   seoyeon  Run     1h 2m          │ ← "next" 매칭만
│   41835  vite(next dep) —       7   0.4%   96MB    seoyeon  Run     1h 2m          │   실시간 좁혀짐
└──────────────────────────────────────────────────────────────────────────────────┘
┌ Details ── [Info] Log  Net  Env ──────────────────────────────────────────────────┐
│ PID 41822  Next.js (Node)  :3000  2.1%  128.0 MB  1h 2m 5s                          │
└──────────────────────────────────────────────────────────────────────────────────┘
 /next█    | [Enter]Apply  | [Esc]Cancel                                            ← 필터 입력(BAR-BOTTOM)
```

### 8-12. V-EMPTY (빈 상태 = 로딩 상태, 스피너)

```
 ntop v0.2.0 | CPU: 5.1%  MEM: 6100/16384MB | Servers: 0 | Refresh: 3s | [H]elp
┌─────────────────────────────── ntop ──────────────────────────────────────────────┐
│                                                                                    │
│                                                                                    │
│                       ⠹  Scanning for server processes...                          │ ← 스피너(Cyan, tick_count)
│                                                                                    │
│                       No server processes found. Waiting...                        │
│                                                                                    │
│                Start a server and it will appear here automatically.               │
│                                                                                    │
└────────────────────────────────────────────────────────────────────────────────────┘
 [q]Quit | [Up/Down]Navigate | [/]Filter | [s]Sort | [x]Kill | [S]Signal
```

> **검수 포인트(STEP 7):** ① 상/하 분할이 맞는지(정정 노트), ② process_list 컬럼 순서/폭(코드:
> [✓]PID·NAME·PORT·THR·CPU·MEM·USER·STATUS·UPTIME), ③ 상단/하단바 문구가 실측 문자열과 맞는지,
> ④ 마스킹·스피너·탭 강조 표현, ⑤ CLI 출력 시안(별도 — `32-api-spec`에서 table/json/csv 스키마로 상술).

---

## 9. 디자인 품질 노트 (E-8) — 4-D UX 기본기 점검

> spec/generator-prompt 4-D: ① 수동 모드/역할 선택 벽 금지(계정·맥락으로 자동 분기) ② 계정 파생
> 정보 재입력 금지 ③ 중복 단계/입력 최소화. ntop은 계정·로그인이 없는 로컬 도구이므로, 이 원칙을
> **"맥락으로 결정되는 것을 사람에게 묻지 않는다"** 로 번역해 점검한다.

| 4-D 원칙(ntop 번역) | ntop의 실제 설계 | 판정 |
|---|---|---|
| **수동 모드 선택 벽 없음** | 런타임/프레임워크를 **자동 분류**(규칙 테이블)해 표시. "어느 런타임 볼래?" 같은 선택 화면 없음. Node만 보려면 **`n` 토글 한 키**(모드 마법사 아님) | ✅ 모범 |
| **맥락 파생 정보 재입력 금지** | 포트·프레임워크·메모리·소유자·트리 부모를 **스캔으로 server-side 해결**, 사용자에게 묻지 않음(PRB-2 포트도 자동 매핑) | ✅ 모범 |
| **중복 단계/마찰 최소** | 필터는 **입력마다 실시간**(rebuild_view, 별도 Apply 누름 불필요) · 메인 한 화면에 식별+점검+종료 집약 · CLI는 동일 코어 평면 5명령 | ✅ 모범 |
| **불필요 화면 없음** | 설정은 파일(`~/.config/ntop/config.toml`) + `ntop config`로 표시, **TUI 설정 화면을 따로 두지 않음**(CLI 관행) | ✅ 합리적 트레이드오프 |
| **위험 동작에만 의도적 마찰** | kill/signal에만 모달 확인(`confirm_before_kill`), graceful→force는 **2단계 확인**이지만 "되돌릴 수 없는 SIGKILL"을 의식적 결정으로 만드는 **정당한 안전 마찰**(중복 아님) | ✅ 모범 |

**도입할 순진한 결함(naive defect) 없음 — 확인 결과:**
- 수동 역할/모드 선택 게이트 ❌ 없음 (Node-only는 토글, 분류는 자동).
- 계정/맥락으로 알 수 있는 값을 묻는 입력 ❌ 없음 (포트·런타임·소유자 자동 해결).
- 쌓인 중복 단계 ❌ 없음 (필터 실시간, 단일 메인 뷰).

**정직한 경계 사례(결함은 아니나 기록):**
1. **하단바가 컨텍스트별로 키 목록이 달라 일부 전역키(예 `K` 트리 kill, `e` 확장, `n` Node-only,
   `r` reverse)가 상시 노출되지 않음** → 발견성은 `H`(Help)에 의존. CLI 도구 관행상 허용 범위이나,
   `21-screen-spec`에서 "Help 외 노출 경로"를 명시할 가치 있음(개선 제안, 결함 아님).
2. **TUI 내 설정 편집 화면 부재** → 값 변경은 파일 편집 후 재기동/재스캔. 단일 바이너리·저마찰 철학상
   의도된 선택(웹 GUI 설정 패널을 만드는 것이 오히려 spec §7 비목표·과설계).
3. **DLG-FORCE의 2단계 확인** → 일견 중복처럼 보이나, 1단계(SIGTERM)와 2단계(SIGKILL, 못 잡힘)는
   **의미가 다른 결정**이라 중복 마찰이 아니다(§4 근거). 유지 권장.

**결론:** ntop의 UX는 4-D 기본기를 충족한다 — 자동 분류로 모드 선택 벽을 없애고, 맥락을 server-side로
해결하며, 위험 동작에만 정당한 확인을 둔다. **새로 도입할 순진한 결함 없음.** 후속 `21`이 §9-(1)
발견성만 보강하면 된다.

---

## 10. 후속 스프린트(S4)로의 인계 + 사실 정정 플래그

- **`12-ia`** ← §2 IA 트리(색·역할 범례 포함) 그대로 사용.
- **`13-user-flows`** ← §3 플로우 6종 + §4 시퀀스/상태 다이어그램. 각 단계의 화면 ID·PRB 링크 유지.
- **`20-wireframes`** ← §8 와이어프레임 12종(상/하 분할 반영). **HUMAN_CHECKPOINT_REQUIRED** 인계.
- **`21-screen-spec`** ← §5 화면 인벤토리 + §6 4상태 + §7 키 바인딩(정규 출처). §9-(1) 발견성 보강 반영.
- **`30-functional-spec`** ← §4 graceful→force 상태기계(killer.rs 의미)를 처리 플로우로 참조.

> 🚩 **메인테이너 회부(P-1 휴먼 체크포인트에서 처리):** 도시에 §6의 메인 레이아웃 표기
> "process_list(좌 55%) | detail_panel(우 45%)"(좌/우)는 **실제 코드(`src/tui/ui.rs` L37–44,
> `Layout::vertical([55%,45%])` = 상/하)와 불일치**. 코드를 지상 진실로 본 본 문서는 **상/하**를
> 채택했으므로, 도시에 §6을 "상(55%)/하(45%) 세로 분할"로 정정할 것을 권고한다(또는 코드를 좌/우로
> 바꾸는 것은 구현 변경이라 비목표 — spec §7).

---

## 자체 검증 (Self-verification)

Sprint Contract의 각 체크를 실제 산출물에 비추어 확인한다.

| # | 체크 | 결과 | 어디서 충족 |
|---|---|---|---|
| **E-1** | IA 트리(TUI+CLI, 역할/색) | **PASS** — §2 색·역할 범례(코드 Color 근거) + TUI/CLI 통합 ASCII 트리(화면 ID·진입키·PRB 부기) | §2 |
| **E-2** | 핵심 플로우 6종 플로우차트 + 화면/키/PRB 링크 | **PASS** — F1~F6 전부 ASCII, 단계마다 화면 ID·키·PRB(research-s2) 라벨, 분기 `◇` | §3 |
| **E-3** | kill graceful→force 시퀀스 + 상태 다이어그램 | **PASS** — §4-1 상태기계(Terminated/TimedOut/EPERM/AlreadyDead), §4-2 시퀀스(User↔App↔Killer↔OS↔Process), TUI/CLI 에스컬레이션 차이 명시 | §4 |
| **E-4** | 화면 인벤토리(전 뷰/패널/탭/다이얼로그 + 빈/에러) | **PASS** — 21행 표(TUI 16 + CLI 5), ID·역할·위젯파일·진입·PRB | §5 |
| **E-5** | 핵심 화면 4상태(empty/loading/normal/error) | **PASS** — §6-1 매트릭스(V-LIST·NET·LOG·ENV·KILL), ★EPERM·★lsof/netstat 부재 트리거, empty=loading 합침 사실 명시, §6-2 전이도 | §6 |
| **E-6** | 키 바인딩 5그룹(도시에 §6+코드) | **PASS** — 전역/리스트/상세/필터/다이얼로그 표 + 컨텍스트 의존 하단바 실측(status_bar.rs) | §7 |
| **E-7** | 와이어프레임 화면당 1개 → HUMAN_CHECKPOINT | **PASS** — 12 시안(메인·Info/Log/Net/Env·KILL/KILLTREE/SIGNAL/FORCE/HELP·필터·빈상태), 상단 `HUMAN_CHECKPOINT_REQUIRED` 표기 | §8 |
| **E-8** | 디자인 품질 노트(4-D, 결함 플래그) | **PASS** — 4-D 번역 점검표(전부 ✅) + 순진한 결함 0 확인 + 경계 사례 3건 정직 플래그 | §9 |

**추가 게이트/원칙 자체 점검:**
- **G-g(인포그래픽 우선, 외부 렌더러 0):** IA 트리·플로우차트 6종·상태기계·시퀀스·4상태 매트릭스/전이도·
  와이어프레임 12종 모두 **순수 ASCII**. Mermaid 등 외부 렌더러 미사용, 빈 다이어그램 없음. 모든 화면에
  시각 동반(산문-only 화면 0).
- **P-1(와이어프레임 휴먼 체크포인트):** §8 상단에 `HUMAN_CHECKPOINT_REQUIRED` 명시, 검수 포인트 5개 제시.
- **C1 일부(플로우↔문제 일관):** §3 모든 플로우 단계에 PRB-ID(research-s2 §3) 링크 → 문제-플로우 일관.
- **사실 역추적성:** 모든 화면/키/플로우에 도시에 §번호 **또는 파일:라인**(ui.rs L19/L37/L239/L245/L275/
  L281/L286/L290, status_bar.rs, empty_state.rs, process_list.rs, detail_panel.rs) 부기. research-s2
  PRB-1~8 교차 인용.
- **창작 금지 준수:** 결제/정산/DB/REST/회원 화면 0. 없는 화면(설정 마법사·로그인·대시보드 등) 지어내지
  않음. CLI/TUI는 실제 구현된 것만 열거.
- **단일 출처 일관:** 키 바인딩은 §7을 정규 출처로 단일화(spec §4-3 준수, `21`이 이어받음).

**알려진 한계(generator_report에 상세):**
- **레이아웃 좌/우 vs 상/하 불일치:** 도시에 §6(좌/우) ↔ 코드(상/하). 코드를 채택하고 §0·§10에서
  정정 플래그 → P-1 휴먼 체크포인트로 회부. Evaluator가 도시에 표기를 강제하면 재조정 가능하나, spec
  line 5·playbook line 8이 "도시에 + 실제 소스코드"를 공동 사실 원천으로 규정하므로 코드 채택이 정합적.
- **와이어프레임 폭/정렬:** 저충실도·예시 폭(≈78칸)이며 실제는 터미널 폭에 신축. 시각 품질은 P-1 검수
  대상(채점 제외).
- 본 문서는 S3 범위(IA·플로우·화면·키·와이어프레임)에 한정. 정규 기능 ID 부여·기능↔문제 추적표는 S4
  `10-prd`, CLI 출력 스키마 상세는 S4 `32-api-spec`.
