# ntop — Codebase Facts Dossier (Source of Truth)

> 이 문서는 ntop 실제 소스코드를 정밀 분석해 추출한 **사실 모음**이다. Planner/Generator/Evaluator 모든 에이전트는
> 기획서의 사실 근거를 **오직 이 dossier + 실제 코드**에서만 가져온다. 추측·창작 금지.
> ntop은 **이미 구현된 OSS 도구**다. 따라서 이 기획서는 "신규 아이디어"가 아니라 **기존 코드를 역설계(reverse-engineering)한 기획서**다.

---

## 0. 한 줄 정의

**ntop ("Node Top")** = 여러 런타임(Node·Python·Java·Deno·Bun·Ruby·PHP·.NET)의 **서버 프로세스를 모니터링·관리**하는 Rust 製 TUI + CLI 단일 바이너리. macOS·Linux·Windows.

- Cargo: `name = "ntop"`, `version = "0.2.0"`, `edition = "2021"`, `license = "MIT"`, repo `https://github.com/greeun/ntop`.
- 바이너리 1개(`src/main.rs`), 라이브러리 크레이트(`src/lib.rs`, `use ntop::...`).
- keywords: node, process, tui, monitor, nuxt. categories: command-line-utilities, development-tools.

## 0-1. 도메인 특수성 (Mode 5 16문서 적응 규칙 — 반드시 준수)

ntop은 **백엔드 서버·DB·결제·HTTP API·회원 계정이 없는 로컬 개발자 도구(OSS CLI/TUI)**다. Mode 5의 16종 문서를 기계적으로 적용하면 안 되고 **도메인에 맞게 재해석**한다:

| 표준 문서 | ntop에서의 재해석 |
|---|---|
| `00-business-model` | **OSS 채택·지속가능성 모델**: 라이선스(MIT), 배포 채널(cargo/crates.io/Homebrew/릴리스 바이너리), GitHub Stars·다운로드 기반 성장, **수익화 옵션(GitHub Sponsors·기업 후원·향후 Pro/Team 기능 가설)**. "정산/take-rate"는 없음 → "비용 구조 + 채택 퍼널 + 후원/수익 가설"로 치환. 가설은 반드시 "가설"로 명시. |
| `31-erd` | **인메모리 도메인 모델**: DB 없음. `ProcessInfo`·`Config`·`NetworkConnection`·`LogStreamer`·`App` 등 핵심 구조체의 필드·관계를 ERD 스타일로 시각화. |
| `32-api-spec` | **외부 인터페이스 계약**: ① CLI 명령 계약(서브커맨드/플래그/종료코드/`--json`·`--format csv` 출력 스키마) ② **공개 라이브러리 API**(`ntop::` 크레이트 공개 함수/타입) ③ 의존 시스템 명령(`lsof`/`netstat`/`taskkill`)과의 계약. HTTP 엔드포인트 아님. |
| `33-policy` | **안전·프라이버시 정책**: kill 확인 게이트, 시그널 권한(EPERM), 환경변수 민감값 마스킹, **"파일시스템 안 읽음" 프라이버시 원칙**, graceful→force 에스컬레이션 정책. |
| `03-personas` | 결제 고객 아님 → **개발자/운영자 사용자 페르소나**(로컬에서 다중 런타임 서버 띄우는 개발자 등). |
| `40-backlog` | ntop은 이미 v0.2.0까지 구현됨 → 백로그는 "현재까지 구현된 것 정리 + **향후 로드맵**(README/코드의 미구현 흔적 기반)" 관점. |

이 재해석을 어기고 "결제/정산/회원 DB/REST 엔드포인트"를 지어내면 **창작(fabrication) = 실패**다.

---

## 1. 아키텍처 / 모듈 맵

```
src/
  main.rs              바이너리 진입점: clap 파싱 → 서브커맨드 분기 or TUI 기동(run_tui)
  lib.rs               라이브러리 크레이트 노출 (use ntop::...)
  cli.rs               clap derive: Cli + Commands enum + ListFormat
  config.rs            TOML 설정 (~/.config/ntop/config.toml), serde(default)
  process/
    mod.rs             Runtime / FrameworkKind / HealthStatus / ProcessInfo (핵심 도메인 타입)
    framework_rules.rs FRAMEWORK_RULES + RUNTIME_RULES 규칙 테이블 (★확장 포인트)
    framework.rs       FrameworkDetector::classify — 2단계 규칙 적용
    scanner.rs         ProcessScanner — sysinfo 기반 스캔 (CPU delta 위해 long-lived)
    tree.rs            TreeBuilder — flat list → roots/children, flatten, sort
    killer.rs          KillSignal / ProcessKiller — 시그널·graceful·tree·force kill
    network.rs         NetworkInspector — lsof(unix)/netstat(win) 파싱 → 포트/연결
    platform.rs        OS별 FFI — macOS phys_footprint, thread/fd count
  log/
    mod.rs
    streamer.rs        LogStreamer — cwd 로그파일 glob·tail
  tui/
    mod.rs
    app.rs             App — 단일 상태 구조체 (raw_processes / flat_list / 모든 UI 상태)
    event.rs           EventHandler — async tick + key + resize
    ui.rs              렌더링 + 모든 키 핸들링 (handle_key 분기)
    widgets/
      process_list.rs  좌측 프로세스 목록(트리)
      detail_panel.rs  우측 상세 패널 (Info/Log/Net/Env 탭 컨테이너)
      info_tab.rs / log_tab.rs / net_tab.rs / env_tab.rs   각 탭
      kill_dialog.rs / signal_picker.rs / help_dialog.rs    다이얼로그
      status_bar.rs / empty_state.rs                        상단바 / 빈 상태
```

**의존성(Cargo.toml)**: clap4(derive), crossterm 0.28, ratatui 0.29, sysinfo 0.33, tokio1(full), serde/serde_json, toml 0.8, csv 1, chrono 0.4, dirs 6, notify 7, glob 0.3, unicode-width 0.2, anyhow 1. Unix: nix 0.29(signal,process). Windows: windows-sys 0.59. dev: tempfile, assert_cmd, predicates.

---

## 2. 런타임 / 프레임워크 탐지 (핵심 추상화)

**데이터 주도(data-driven)** 규칙 테이블이 전부. `src/process/framework_rules.rs`의 두 `&[Rule]`:

- `Rule` = `{ runtime, framework, name_exact, command_binary, command_contains }`. 3종 매처를 신뢰도 순으로 검사: **name_exact → command_binary → command_contains**.
- 2단계 분류(`FrameworkDetector::classify`): ① `FRAMEWORK_RULES`(구체 프레임워크) 먼저 → ② `RUNTIME_RULES`(맨몸 인터프리터) → ③ config로 게이트되는 dev runner(`tsx`/`ts-node`). 어느 규칙에도 안 맞으면 `None` → **표시 안 됨**.
- **탐지는 프로세스-로컬 전용**: 프로세스 이름 + 커맨드라인만. **파일시스템(`package.json`)을 절대 읽지 않음.** 이유: 전역 실행 프로세스(npx MCP 서버, CLI 도구)를 상속된 cwd로 오분류하는 것을 막기 위함. (= 차별점이자 설계 원칙)
- 확장: 테이블에 `Rule` 1개 추가 + `Runtime`/`FrameworkKind` enum에 variant 추가. **그 외 코드 변경 불필요.**
- `normalize_name`: macOS가 `comm` 필드를 16자로 자르기 때문에 첫 공백 토큰만 취함.

### Runtime enum (8종)
`Node, Python, Java, Deno, Bun, Ruby, Php, DotNet` — Display: Node/Python/Java/Deno/Bun/Ruby/PHP/.NET.

### FrameworkKind enum (15종)
`NextJs, Express, Fastify, NestJs, Nuxt, Koa, Hapi, FastApi, Flask, Django, SpringBoot, Rails, Laravel, AspNet, Generic`.
(주의: Express/Fastify/Koa/Hapi는 enum에 존재하나 framework_rules에는 현재 규칙 미등록 — 코드상 enum variant만 있음.)

### 현재 등록된 규칙 (framework_rules.rs 실제값)
**FRAMEWORK_RULES (구체 프레임워크, 먼저 검사):**
| Runtime | Framework | name_exact | command_binary | command_contains |
|---|---|---|---|---|
| Node | Next.js | next-server, next-router-worker, next-router-page-worker | (동일) | node_modules/.bin/next |
| Node | Nuxt.js | nuxt, nuxi | nuxt, nuxi | node_modules/.bin/nuxt |
| Node | NestJs | — | — | node_modules/.bin/nest |
| Python | Django | — | — | manage.py, django |
| Python | Flask | — | flask | -m flask |
| Python | FastAPI | — | — | fastapi |
| Java | Spring Boot | — | — | org.springframework, spring-boot |
| Ruby | Rails | rails | rails | bin/rails |
| PHP | Laravel | — | — | artisan |
| .NET | ASP.NET | — | — | Microsoft.AspNetCore, aspnet |

**RUNTIME_RULES (맨몸 런타임, 나중에 검사):**
| Runtime | name_exact / command_binary |
|---|---|
| Node | node |
| Python | python, python3, uvicorn, gunicorn, hypercorn, celery |
| Java | java |
| Ruby | ruby, puma, unicorn, rackup |
| PHP | php, php-fpm |
| .NET | dotnet |
| Deno | deno |
| Bun | bun |

---

## 3. 핵심 도메인 타입 (인메모리 데이터 모델 = ERD 근거)

### `ProcessInfo` (src/process/mod.rs)
```
pid: u32
ppid: u32
name: String
command: String
cwd: String
framework: FrameworkKind
framework_version: Option<String>
ports: Vec<u16>
cpu_percent: f32
memory_rss: u64
memory_vms: u64
threads: u32
uptime: Duration
user: String
status: String
open_fds: u32
children: Vec<ProcessInfo>     // 트리 — 자기참조 1:N
env_vars: Vec<(String,String)>
runtime: Option<Runtime>       // Some=분류된 서버 / None=트리 컨텍스트 부모(launchd,claude,shell)
```
메서드: `is_server()`(runtime.is_some), `is_node()`(runtime==Node), `uptime_display()`("1h 2m 5s"), `memory_display()`("128.0 MB"/"1.5 GB"), `display_name()`(커맨드에서 사람이 읽을 이름 추출 — node_modules 툴명/서브커맨드/스크립트 파싱), `health()`.

### `HealthStatus` { Healthy, Warning, Critical }
- `from_cpu_mem`: CPU≥90 or MEM≥90 → Critical; ≥80 → Warning; else Healthy.
- `from_process_status`: "Zombie"|"Dead" → Critical; else Healthy.

### `Config` (config.rs) — 3 섹션, 모두 `#[serde(default)]`
```
[general]
  refresh_interval: u64 = 3        # 초
  default_signal: String = "SIGTERM"
  graceful_timeout: u64 = 10       # 초
  confirm_before_kill: bool = true
[display]
  show_tree: bool = true
  color_theme: String = "auto"     # auto | dark | light
  mask_env_values: bool = true     # PASSWORD/SECRET/TOKEN 등 마스킹
[filter]
  include_bun: bool = false         # DEPRECATED (Bun은 1급 런타임, 항상 탐지)
  include_tsx: bool = false         # tsx dev runner 포함 토글
  include_ts_node: bool = false     # ts-node dev runner 포함 토글
```
경로: `dirs::config_dir()/ntop/config.toml` (= `~/.config/ntop/config.toml`). 파일 없거나 파싱 실패 → 기본값. `refresh_duration()`/`graceful_duration()` → Duration 변환.

### `NetworkConnection` (network.rs)
```
local_addr: SocketAddr
remote_addr: Option<SocketAddr>
state: String          # "LISTEN" / "ESTABLISHED" 등
pid: u32
```

---

## 4. 스캐닝 (scanner.rs + main.rs do_scan)

- **`System`(sysinfo)는 의도적으로 long-lived.** sysinfo는 두 refresh 간 delta로 per-process CPU를 계산 → 매 tick마다 재생성하면 모든 CPU가 영원히 0.0%. `new()`는 두 번 refresh + `MINIMUM_CPU_UPDATE_INTERVAL` sleep으로 prime. `scan_blocking()`은 1회성 CLI 호출용으로 3번째 측정 refresh 추가. TUI 루프 내내 persistent 유지.
- **2-pass 스캔**: Pass1 = 분류된 서버(`runtime=Some(_)`) 수집. Pass2 = 서버 아닌 부모(launchd/claude/shell)를 backfill해 트리 루트 확보 → 이들은 `runtime=None`.
- **포트는 분류 후 별도로** `NetworkInspector::connections_by_pid()`(LISTEN만)로 채움. 스캐너는 포트를 안 가져옴.

## 4-1. 네트워크 검사 (network.rs)
- Unix: `lsof -iTCP -sTCP:LISTEN,ESTABLISHED -nP -F pcnT` 의 field-mode 출력 파싱.
- Windows: `netstat -ano -p TCP` 파싱 ("LISTENING"→"LISTEN" 정규화).
- `parse_addr`: `*:port`→0.0.0.0, `[::1]:port` IPv6, `127.0.0.1:port` IPv4.
- `listening_ports_for_pid` / `connections_for_pid` / `connections_by_pid`.

## 4-2. 플랫폼 FFI (platform.rs)
- macOS: `phys_footprint`(via `proc_pid_rusage`)를 sysinfo RSS 대신 사용 — Activity Monitor와 일치(유휴 Node에서 sysinfo RSS가 최대 100배 과소보고). `thread_count`, `open_fd_count`.
- 非macOS/Windows: zero/None fallback.

## 4-3. 로그 스트리밍 (log/streamer.rs)
- `LOG_PATTERNS`: `*.log`, `.next/server/app/**/*.log`, `.next/trace`, `logs/*.log`, `log/*.log`, `npm-debug.log`, `yarn-error.log`.
- `detect_and_open(cwd)`: 패턴 glob → 가장 최근 수정 파일 tail, 끝(SeekFrom::End)으로 seek 후 새 줄만 스트림. 버퍼 `MAX_BUFFER_LINES = 1000`.
- Linux: `detect_and_open_with_proc` — cwd 실패 시 `/proc/<pid>/fd/1`(stdout) fallback.

---

## 5. 종료 / 시그널 (killer.rs)

### `KillSignal` enum
- 크로스플랫폼: `Term`(SIGTERM), `Kill`(SIGKILL), `Int`(SIGINT).
- Unix 전용(#[cfg(unix)]): `Hup`(SIGHUP), `Usr1`(SIGUSR1), `Usr2`(SIGUSR2).
- `all()`: Unix=6종[Term,Kill,Hup,Int,Usr1,Usr2], Windows=3종[Term,Kill,Int].
- 설명: Term="Graceful termination request", Kill="Force kill (cannot be caught)", Hup="Hangup / reload configuration", Int="Interrupt (like Ctrl+C)", Usr1="User-defined (Node.js: activate debugger)", Usr2="User-defined signal".
- `from_str`: "SIGTERM"/"TERM" 둘 다 허용(대소문자 무시).

### `ProcessKiller`
- `send_signal(pid, signal)` → `KillResult` { Success, AlreadyDead, PermissionDenied, Error(String) }. Unix=nix `signal::kill`(ESRCH→AlreadyDead, EPERM→PermissionDenied). Windows: Term/Int→`taskkill /PID`, Kill→`OpenProcess`+`TerminateProcess`.
- `is_alive(pid)`.
- `graceful_kill(pid, timeout)` → `GracefulResult` { Terminated, TimedOut, AlreadyDead, PermissionDenied, Error }. SIGTERM 후 200ms 간격 폴링, timeout까지 살아있으면 TimedOut.
- `force_kill(pid)` = SIGKILL.
- `kill_tree(pids, signal)` — pids를 **역순(자식부터)** 으로 시그널, `Vec<(pid, KillResult)>` 반환.

---

## 6. TUI (tui/app.rs + ui.rs + event.rs)

### App 상태 (단일 구조체, app.rs)
- `raw_processes`(스캔 원본, 필터 전) vs `process_trees`/`flat_list`(필터·정렬·평탄화된 표시용). `flat_list = Vec<(ProcessInfo, depth)>`.
- `rebuild_view()`: 필터 + node-only 토글 + 정렬 재적용 후 재평탄화. **필터 키 입력마다 호출**해 다음 스캔 tick 안 기다리고 실시간 갱신.
- 선택: `selected_index`, `selected_pids: HashSet<u32>`(Space 다중선택), `expanded_pids: HashSet<u32>`(트리 확장).
- 탭: `active_tab: DetailTab`.
- 다이얼로그: `dialog: Option<DialogKind>`.
- 필터: `filter_text`, `filter_active`.
- 정렬: `sort_column: SortColumn`, `sort_ascending: bool`.
- 로그: `log_streamer`, `log_scroll`.
- 시스템: `system_cpu`, `system_memory_used`, `system_memory_total`.
- kill 진행: `kill_in_progress: Option<(u32, Instant)>`.
- 기타: `tick_count`(스피너), `first_load`(첫 로드 시 전부 확장), `refresh_secs`/`refresh_changed`, `needs_rescan`, `focus: FocusPanel`, `node_only`.

### enum들
- `DetailTab` { Info, Log, Net, Env } — next/prev 순환.
- `DialogKind` { KillConfirm, KillTreeConfirm, SignalPicker, ForceKillPrompt, Help }.
- `FocusPanel` { ProcessList, DetailPanel }.
- `SortColumn` { Pid, Name, Port, Threads, Cpu, Memory, User, Status, Uptime } — next로 순환, 라벨 PID/NAME/PORT/THR/CPU/MEM/USER/STATUS/UPTIME.

### 필터 매칭 (`matches_filter`)
대소문자 무시 substring: name, command, pid(문자열), framework, runtime, ports 중 하나라도 매칭. 빈 필터 = 전부 매칭.

### Node-only 토글 (`rebuild_view` 필터)
`node_only`면 strictly Node 서버(`is_node()`)만 + 트리 컨텍스트 부모(runtime None) 유지. Deno/Bun은 별도 런타임이라 숨겨짐.

### 화면 레이아웃 (ui.rs — 코드 권위, 정정됨)
- 세로: `[top_bar(1줄) | main_content(≥5) | bottom_bar(1줄)]`.
- main_content는 **세로(상/하) 분할** — `Layout::vertical([55%, 45%])` (ui.rs L37–46): **위 55% = process_list, 아래 45% = detail_panel**. (좌/우 아님. README의 좌우 ASCII는 구식 예시 — 코드가 권위.)
- flat_list가 비면 main_content 전체에 empty_state 렌더(스피너 = tick_count).

### TUI 프로세스 목록 컬럼 (process_list.rs — 코드 권위, 정정됨)
실제 컬럼(넓이≥100 기준): `[ ✓/health dot ] · PID · NAME · PORT · THR · CPU · MEM · USER · STS · UPTIME` (정렬 인디케이터 `^`/`v`).
- **TUI 좌(상단) 목록에는 FRAMEWORK 컬럼이 없다.** framework/runtime은 **상세 Info 탭** + **CLI `list` 테이블(FRAMEWORK 컬럼)** 에서만 노출. (※ 일부 선행 문서가 "목록 NAME/FRAMEWORK 컬럼"으로 표현했으나 이는 코드와 불일치 — 21-screen-spec/32-api-spec은 이 정정값을 정규 출처로 사용.)
- 헤더 라벨은 STATUS 컬럼이 목록에선 `STS`로 약칭(SortColumn::Status 라벨은 "STATUS").
- 좁은 폭(≤100)에선 동일 컬럼에 넓이만 축소.

### 키 바인딩 (ui.rs 권위 — 정밀)
**전역(normal 모드):**
| Key | Action |
|---|---|
| q, Ctrl+C | 종료 |
| / | 필터 모드 진입 |
| s | 정렬 컬럼 순환 |
| r | 정렬 방향 반전 |
| + | refresh 간격 +1s (최대 60s) |
| - | refresh 간격 −1s (최소 1s) |
| x | KillConfirm 다이얼로그 |
| K | KillTreeConfirm 다이얼로그 |
| H | Help 다이얼로그 |
| S | SignalPicker 다이얼로그 |
| e | 전체 확장/축소 토글 |
| n | Node-only 뷰 토글 |

**리스트 포커스(ProcessList):** Up/k=위, Down/j=아래, PageUp=10↑, PageDown=10↓, Home=처음, End=끝, Enter=트리 토글, Right/l=확장(expand_selected), Left/h=축소(collapse_selected), Space=다중선택, Tab=DetailPanel로 포커스, Esc=종료.

**상세 포커스(DetailPanel):** Tab/Right/l=다음 탭, BackTab/Left/h=이전 탭, Up/k=스크롤↑(detail+log), Down/j=스크롤↓, PageUp=−10, PageDown=+10, Home=맨위, Esc=ProcessList로 복귀.

**필터 모드:** Esc=필터 비우고 종료, Enter=필터 유지하고 종료, Backspace=한 글자 삭제, Char(c)=입력(name/command/framework/pid/ports 대소문자 무시 매칭).

**다이얼로그:**
- KillConfirm: Enter=SIGTERM 전송, Esc=취소.
- KillTreeConfirm: Enter=kill_tree(SIGTERM), Esc=취소.
- SignalPicker: Up/k·Down/j=이동, Enter=선택 시그널 전송, Esc=취소.
- Help: Esc/H/q=닫기, Up/k·Down/j·PageUp/Down·Home·End=스크롤.
- ForceKillPrompt: Enter=force_kill, Esc=취소.

### 상세 패널 탭 내용
- Info: PID/Framework/Port/CPU/Memory/Uptime 등 메트릭.
- Log: 감지된 로그파일 실시간 tail.
- Net: LISTEN 포트 + 활성 TCP 연결.
- Env: 환경변수 (민감값 자동 마스킹, `mask_env_values`).

---

## 7. CLI (cli.rs + main.rs)

`ntop` (서브커맨드 없음) → TUI 기동. 서브커맨드:
- `list` `[--json] [--format <table|csv|json>]` — 분류된 서버 목록.
  - **table 컬럼**: `PID | NAME | FRAMEWORK | PORT | CPU | MEM | UPTIME`.
  - **JSON 출력**: 모든 필드 + depth.
  - **CSV 출력**: `PID | PPID | NAME | FRAMEWORK | PORTS | CPU | MEMORY | UPTIME | STATUS`.
- `kill <PID>` `[--tree] [--signal <SIG>] [--all] [--no-confirm]` — PID는 `--all` 없으면 필수.
  - **graceful flow**: Term(기본) → `config.graceful_timeout` 초과 시 → `force_kill`(SIGKILL) 에스컬레이션. 결과 enum `GracefulResult { Terminated, TimedOut, AlreadyDead, PermissionDenied, Error }`.
  - **tree flow**: 재귀로 모든 PID 수집 → 확인 프롬프트 → 각각 kill.
  - **all flow**: 확인 프롬프트 → 모든 서버 프로세스 열거 → 각각 kill.
  - `--no-confirm`이면 확인 생략.
- `info <PID>` — 상세 출력: PID, PPID, name, runtime, framework, version, ports(LISTEN만), CPU%, memory, threads, uptime, user, status, health(), cwd, command, open FDs, env vars, network connections.
- `log <PID>` — cwd에서 로그소스 자동 감지 → 200ms 폴링 스트림 → 프로세스 사망 시 종료.
- `config` — 설정 파일 경로 + `[general] [display] [filter]` 설정 출력.

clap `about`: "Node Top - Monitor and manage server processes (Node, Python, Java, and more)".

CLI 핸들러는 `do_scan`과 동일하게 `ProcessScanner` + `NetworkInspector` 재사용. `scan_blocking()`(sleep + scan)으로 1회성 측정.

### 이벤트 루프 (main.rs run_tui + event.rs)
- **터미널 셋업**: `enable_raw_mode()` → `EnterAlternateScreen` → `CrosstermBackend` → `Terminal::new()`. **해제**: `disable_raw_mode()` → `LeaveAlternateScreen` → `show_cursor()`.
- `EventHandler::new(tick_rate)`가 blocking std::thread를 띄워 crossterm 이벤트 폴링 → mpsc 채널로 `AppEvent::{Tick, Key(키, press만), Resize(w,h)}` 전송. `next()`는 async recv, 채널 닫히면 None.
- 루프: `terminal.draw(ui::render)` → `events.next().await` → Tick=`tick_count++`+`do_scan()`+(Log탭이면 로그 폴링) / Key=`ui::handle_key()` 후 `should_quit`·`refresh_changed`·`needs_rescan` 체크 / Resize=다음 프레임 재렌더.

### 스캐너 메트릭 (scanner.rs scan() 2-pass)
- Pass1: 전체 프로세스 열거 → `FrameworkDetector::classify(name, command, config)` → 서버 프로세스 pid별 수집.
- Pass2: 서버의 부모(ppid) 중 서버 아닌 것 backfill.
- 프로세스별 메트릭: cpu_percent, memory_rss(macOS는 phys_footprint 우선), memory_vms, threads, open_fds, status, uptime, env_vars, user, ppid, command, cwd.

---

## 8. 트리 빌드 (tree.rs)
- `TreeBuilder::build`: flat list → roots/children를 PID 집합으로 분할.
- `flatten_with_depth`, `collect_pids`, `sort_recursive`(레벨별 정렬).

## 9. 테스트 (tests/*.rs, 모듈당 1파일)
`framework_test, scanner_test, tree_test, killer_test, network_test, log_test, cli_test, config_test, filter_test, types_test`. `cli_test`는 `assert_cmd`로 빌드된 바이너리 구동. 커버: 프레임워크 탐지, 트리 빌드, 시그널 처리, 네트워크 주소 파싱, 로그 스트리밍, CLI 디스패치.

## 10. 빌드/품질 명령
`cargo build [--release]`, `cargo run [-- <subcommand>]`, `cargo test [--test <file>] [<name>]`, `cargo clippy`(CI/커밋은 clippy-clean 기대), `cargo fmt`. 브랜치: `develop`에서 작업 → `main`으로 릴리스. 버전은 Cargo.toml.

## 11. 요구사항
macOS / Linux / Windows. Rust 1.70+ (소스 빌드). 설치: `cargo install --git https://github.com/greeun/ntop` 또는 릴리스 바이너리.

---

## 12. 미구현/흔적 (로드맵 근거 — 코드에 존재하나 규칙 미등록 or 한계)
- FrameworkKind에 Express/Fastify/Koa/Hapi variant는 있으나 framework_rules에 탐지 규칙 없음 → 향후 규칙 추가 여지.
- `include_bun` config는 deprecated.
- Windows는 Unix 전용 시그널(HUP/USR1/USR2) 없음.
- 네트워크는 외부 명령(lsof/netstat) 의존 → 해당 명령 없으면 빈 결과.
- 로그는 cwd glob 기반 → 표준출력 직접 캡처는 Linux `/proc/<pid>/fd/1` fallback만.
