# 32 · API 명세 (API Spec) — 외부 인터페이스 계약 (ntop)

> **이 문서는 단일 출처(single source)다.** ntop의 **외부 인터페이스 계약** — ① CLI 명령 계약(서브커맨드·
> 플래그·종료코드·출력 스키마) ② 공개 라이브러리 API(`ntop::` 크레이트가 외부에 노출하는 타입·함수) ③ 의존
> 시스템 명령(`lsof`/`netstat`/`taskkill`)과의 계약 — 은 **여기서만 계약으로 확정**한다. `21-screen-spec`·
> `30-functional-spec`·`41-qa-testcases`는 이 계약을 **참조**할 뿐 재정의하지 않는다(spec §4-3).
>
> **★ HTTP API가 아니다.** ntop은 **백엔드 서버·DB·REST 엔드포인트·회원 계정·결제가 전혀 없는 로컬
> 개발자 도구**다(codebase-facts §0-1). 따라서 "API"는 웹 엔드포인트가 아니라 **명령줄(CLI)·러스트 함수
> (lib)·외부 시스템 명령**이라는 세 종류의 계약을 뜻한다. URL·메서드(GET/POST)·상태코드(200/404)는 없다.
>
> **값은 여기서 정의하지 않는다.** 구조체/enum 필드와 Config 기본값(`refresh_interval=3`·
> `graceful_timeout=10`·`MAX_BUFFER_LINES=1000` 등)의 정규 출처는 **`31-erd`**, 동작·알고리즘(스캔 2-pass·
> 분류·파싱·graceful 폴링)은 **`30-functional-spec`**, 안전·프라이버시 정책(확인 게이트·시그널 권한·마스킹·
> fs 미독)은 **`33-policy`**다. 본 문서는 그 값·동작·정책을 **인용만** 한다(중복정의 0).
>
> **전제:** ntop은 **이미 v0.2.0까지 구현된 MIT 오픈소스 도구**다. 본 명세는 "앞으로 만들 API"가 아니라
> **이미 구현된 인터페이스를 역설계한 계약**이며, 모든 시그니처·플래그·스키마는 실제 소스코드(`src/cli.rs`·
> `src/main.rs`·`src/lib.rs`·`src/process/*.rs`·`src/config.rs`·`src/log/streamer.rs`)에서 그대로
> 추출했다(창작 0).
>
> **용어 풀이(첫 등장):** **CLI**(Command-Line Interface) = 명령 한 줄로 실행해 결과를 출력하는 방식 /
> **서브커맨드(subcommand)** = `ntop list`처럼 메인 명령 뒤에 붙는 하위 명령 / **플래그(flag)** = `--json`
> 처럼 동작을 바꾸는 옵션 / **종료코드(exit code)** = 프로그램이 끝날 때 셸에 돌려주는 숫자(0=성공 관례) /
> **스키마(schema)** = 출력 데이터의 컬럼·필드 구조 / **크레이트(crate)** = 러스트의 라이브러리/패키지 단위 /
> **시그니처(signature)** = 함수의 이름·인자·반환 타입 / **stdout** = 표준 출력(화면) / **stderr** = 표준
> 오류 출력 / **field-mode** = lsof의 "한 줄에 한 필드" 출력 형식.

---

## Sprint Contract (self-proposed checks)

이 문서(g4의 `32-api-spec`)가 만족시킬 **관찰 가능한 체크**(playbook §S4-g4 관찰 바 + spec §5(32행)·§8).

| # | 체크 | 어떻게 충족하나 | 본문 위치 |
|---|---|---|---|
| **A-1** | **CLI 계약 — 서브커맨드/플래그/필수성** — `list`·`kill`·`info`·`log`·`config`(+무서브=TUI) 각 인자·플래그·필수 규칙 | §2 명령/플래그 표 + §2-1~2-6 | §2 |
| **A-2** | **종료코드 계약** — 성공 0 / clap 파싱오류 2 / 런타임 오류 1, "not found"·"cancelled"·"permission denied"는 0(코드-진실) | §2-7 종료코드 표 | §2-7 |
| **A-3** | **출력 스키마** — table(7컬럼)·JSON(17키+depth)·CSV(9컬럼) 정확 명세 | §3 출력 스키마 표 3종 | §3 |
| **A-4** | **공개 라이브러리 API** — `ntop::` 노출 타입/함수 시그니처(ProcessScanner·NetworkInspector·ProcessKiller·FrameworkDetector·Config·TreeBuilder·LogStreamer·KillSignal) | §4 lib API 표(타입 정의는 31 참조) | §4 |
| **A-5** | **의존 시스템 명령 계약** — `lsof`(unix)/`netstat`(win)/`taskkill`(win) 정확 인자 + 부재→빈 결과 | §5 시스템명령 표 + 부재 처리 | §5 |
| **A-6** | **시각화(G-g)** — CLI 호출 시퀀스 다이어그램 + 명령/플래그/종료코드 표 + 출력 스키마 표(외부 렌더러 0) | §6 시퀀스 + §2·§3 표 | §6, §2, §3 |
| **A-7** | **단일 출처 준수** — 값=31·동작=30·정책=33 참조, 계약만 소유. HTTP/REST/DB 창작 0 | 본문 곳곳 참조 + §0 HTTP 부재 콜아웃 | 전체 |

---

## 0. ★ HTTP API가 아니다 — 세 종류의 계약

> ntop에는 **네트워크 서버도, REST 엔드포인트도, 데이터베이스도 없다.** "외부에서 ntop을 어떻게 호출하는가"
> 는 아래 세 경로로만 일어난다. 이 문서는 그 세 계약을 정의한다.

```
 ┌──────────────────────────────────────────────────────────────────────────────┐
 │  ntop의 "외부 인터페이스" = 3종 계약 (HTTP 엔드포인트 0개)                       │
 │                                                                                │
 │  ① CLI 명령 계약          사람/스크립트가 셸에서 `ntop <cmd>` 실행 → stdout      │
 │     ntop / list / kill / info / log / config         (§2·§3)                    │
 │                                                                                │
 │  ② 공개 라이브러리 API     다른 러스트 프로그램이 `use ntop::...`로 재사용         │
 │     ProcessScanner · NetworkInspector · ProcessKiller …   (§4)                  │
 │                                                                                │
 │  ③ 의존 시스템 명령 계약   ntop이 OS의 명령을 호출해 정보 수집 (ntop이 "호출자")  │
 │     lsof(unix) · netstat(win) · taskkill(win)            (§5)                   │
 │                                                                                │
 │   ✗ 없음:  REST/GraphQL 엔드포인트 · URL 경로 · GET/POST · 200/404 · 인증 토큰   │
 │            · DB 쿼리 · 웹훅 · 회원 API                                          │
 └──────────────────────────────────────────────────────────────────────────────┘
```

**이 문서가 소유(정규 정의)하는 것:** CLI 서브커맨드·플래그·필수성·종료코드·출력 스키마(table/JSON/CSV)·
공개 lib 함수 시그니처·의존 시스템 명령의 정확 인자와 부재 처리.

**참조(여기서 정의 안 함):** 구조체/enum/Config 값 → **31** · 처리 알고리즘(스캔·분류·파싱·graceful) →
**30** · 안전/프라이버시 정책 → **33** · 키 바인딩/화면 → **21** · 기능 ID(F#) → **10-prd**.

---

## 1. 진입점과 디스패치 (Entry point)

`ntop` 바이너리는 clap(러스트 명령줄 파서)으로 인자를 읽고, **서브커맨드가 없으면 TUI를 띄우고, 있으면 해당
CLI 핸들러로 분기**한다(`src/main.rs` `main()`). 모든 핸들러는 `do_scan`과 동일하게 `ProcessScanner` +
`NetworkInspector`를 재사용한다(처리 골격은 30-16).

```
 $ ntop [<SUBCOMMAND> [ARGS/FLAGS]]
        │
        ▼  Cli::parse()  +  Config::load()   (~/.config/ntop/config.toml, 부재→기본값 31)
   ┌────┴───────────────────────────────────────────────────────────┐
   │ 서브커맨드 분기 (main.rs match cli.command)                       │
   ├──────────────────────────────────────────────────────────────────┤
   │ None              → run_tui(config)         전체화면 TUI 이벤트 루프  │
   │ List{json,format} → cmd_list                서버 목록(table/json/csv) │
   │ Kill{...}         → cmd_kill                 PID/트리/전체 종료        │
   │ Info{pid}         → cmd_info                 단일 프로세스 상세        │
   │ Log{pid}          → cmd_log                  로그 tail 스트림          │
   │ Config            → cmd_config               설정 경로 + 현재 값       │
   └──────────────────────────────────────────────────────────────────┘
```

**전역 플래그(clap 자동 제공):** `--help`/`-h`(도움말), `--version`/`-V`(버전 = `Cargo.toml`의 `0.2.0`).
clap `about` 문구: `"Node Top - Monitor and manage server processes (Node, Python, Java, and more)"`.

---

## 2. CLI 명령 계약 (Contract ①)

### 2-0. 명령/인자/플래그 표 (정규)

> `required`=필수, `opt`=선택. 코드 근거: `src/cli.rs`(인자 정의) + `src/main.rs`(핸들러).

| 명령 | 인자 | 플래그 | 필수성 | 동작(요지) |
|---|---|---|---|---|
| `ntop` | — | (전역 `-h`/`-V`) | — | TUI 기동(서브커맨드 없음) |
| `ntop list` | — | `--json` (bool) · `--format <table\|csv\|json>` | 둘 다 opt | 분류된 서버 목록 출력 |
| `ntop kill` | `<PID>` (u32) | `--tree` · `--signal <SIG>` · `--all` · `--no-confirm` (모두 bool/opt) | **`PID`는 `--all` 없을 때 필수**(`required_unless_present="all"`) | PID/트리/전체 종료 |
| `ntop info` | `<PID>` (u32, required) | — | `PID` 필수 | 단일 프로세스 상세 출력 |
| `ntop log` | `<PID>` (u32, required) | — | `PID` 필수 | 로그 자동 감지 → 실시간 tail |
| `ntop config` | — | — | — | 설정 파일 경로 + 현재 3섹션 값 출력 |

> **플래그 상호작용(코드-진실):**
> - `list`: `--json`이 켜지면 `--format`보다 **우선**(`effective_format = Json`). `--format` 미지정 시
>   기본 `table`.
> - `kill`: `--signal` 문자열은 `KillSignal::from_str`로 해석(`"SIGTERM"`/`"TERM"` 등 허용, 31 §3-4),
>   **인식 실패 시 조용히 기본 `SIGTERM`으로 폴백**(`unwrap_or(KillSignal::Term)`). `--all`이 켜지면 `PID`
>   인자는 무시(전체 서버 대상). `--tree`는 `PID` 필수.
> - `--no-confirm` 또는 `config.general.confirm_before_kill=false`(31값)면 확인 프롬프트 생략(정책은 33).

### 2-1. `ntop list` — 서버 목록

- **입력:** 없음(스캔). `scan_blocking()`(1회성 측정, 30-3) → 포트 채움 → 트리 빌드 → 평탄화 → 포맷 출력.
- **출력 포맷 3종:** `table`(기본) / `json`(`--json` 또는 `--format json`) / `csv`(`--format csv`). 스키마는 §3.
- **빈 결과:** table 경로는 `"No server processes found."` 한 줄 출력. (JSON/CSV는 빈 배열/헤더만.)
- **종료코드:** 항상 0(스캔 결과가 비어도 정상).

### 2-2. `ntop kill` — 종료 (3경로)

- **단건(`ntop kill <PID>`):** 확인 게이트(33) 통과 → `graceful_kill(pid, graceful_duration())`(30-6).
  결과 분기 출력 — `Terminated`/`AlreadyDead`/`PermissionDenied`/`Error`는 메시지만, **`TimedOut`이면
  `"Graceful kill timed out. Force killing..."` 후 `force_kill`(SIGKILL) 자동 에스컬레이션**(정책 33).
- **트리(`--tree`):** 스캔→트리 빌드→`find_in_trees(PID)`→`collect_pids`→확인→`kill_tree(pids, signal)`
  (자식부터 역순, 30-7). PID 미발견 시 `"Process <PID> not found."`.
- **전체(`--all`):** 스캔→서버 목록→확인(목록 미리보기)→각 PID에 `send_signal(signal)` 1회씩. 서버가 없으면
  `"No server processes found."`.
- **출력:** 각 PID별 `  PID <n>: <KillResult/GracefulResult>` 형태. 취소 시 `"Cancelled."`.
- **종료코드:** 모든 분기 0(권한 거부·미발견·취소 포함 — 메시지만 출력하고 `Ok` 반환, §2-7).

### 2-3. `ntop info <PID>` — 단일 상세

- **출력(사람이 읽는 블록, 16행 + 부록):** `PID · PPID · Name · Runtime(없으면 "—") · Framework · Version
  (없으면 "-") · Ports(LISTEN만, 콤마구분) · CPU(`{:.1}%`) · Memory(`memory_display`) · Threads · Uptime ·
  User · Status · Health(30-12) · CWD · Command · Open FDs` → 이어서 **Environment Variables**(있으면, 개수
  헤더 + `KEY=VALUE` 줄들) → **Network Connections**(있으면, `local -> remote [state]`).
- **★ 코드-진실(프라이버시 주의):** **`ntop info`의 환경변수 출력은 마스킹하지 않는다**(`KEY=VALUE` 원문).
  민감값 마스킹은 **TUI Env 탭에서만** 적용된다(33-policy §4·40-backlog 후보). 창작이 아니라 실제 동작이다.
- **미발견:** `"Process <PID> not found."` 출력 후 종료코드 0.

### 2-4. `ntop log <PID>` — 로그 스트림

- **동작:** 스캔→PID 찾기→`cwd` 비면 `"Cannot determine working directory..."` / `LogStreamer::detect_and_open`
  (30-9)→소스 없으면 `"No log source found..."` → 소스 있으면 `"Streaming logs from: <path>"` + `"Press
  Ctrl+C to stop."` 후 **200ms 폴링 tail 루프**. **프로세스가 죽으면 `"Process <PID> has exited."` 출력 후
  종료.** (Ctrl+C로 수동 중단.)
- **미발견:** `"Process <PID> not found."` → 0.

### 2-5. `ntop config` — 설정 출력

- **출력:** `"Config file: <경로>"`(= `~/.config/ntop/config.toml`, 31) + `[general]`/`[display]`/`[filter]`
  3섹션의 **현재 값**(부재 시 기본값, 31 §4). `include_bun`은 `"include_bun (deprecated):"`로 표기.
- **종료코드:** 0.

### 2-6. `ntop`(서브커맨드 없음) — TUI

- 전체화면 TUI 이벤트 루프 진입(`run_tui`, 30-16·codebase-facts §6). 화면/키 계약은 **21-screen-spec** 소유.
  정상 종료(`q`/`Ctrl+C`/Esc) 시 터미널 복원 후 종료코드 0.

### 2-7. ★ 종료코드 계약 (코드-진실)

> **핵심:** `main()`은 `anyhow::Result<()>`를 반환한다. 거의 모든 핸들러는 실패 상황("프로세스 없음",
> "권한 거부", "취소", "서버 없음")에서도 **메시지만 출력하고 `Ok(())`를 반환**한다 → 종료코드 **0**.
> 비정상 종료코드는 (a) clap 인자 파싱 실패, (b) `?`로 전파되는 런타임 오류뿐이다. **"kill 실패 시 1을
> 반환한다" 같은 동작은 코드에 없다**(창작 금지).

| 코드 | 언제 | 근거 |
|---|---|---|
| **0** | 정상 종료 + **"not found"/"cancelled"/"permission denied"/"no server processes"** 모두 포함 | 각 핸들러가 메시지 출력 후 `Ok(())`(main.rs cmd_*) |
| **2** | **clap 인자 파싱 오류** — 예: `kill`을 `--all` 없이 `PID`도 없이 호출, 미지의 플래그, 잘못된 `--format` 값 | clap 기본 동작(usage 에러는 코드 2, stderr) |
| **1** | **런타임 오류 `?` 전파** — 터미널 셋업 실패, stdin/stdout I/O 오류, JSON/CSV 직렬화 오류 등 anyhow 에러 | `main() -> Result` 가 `Err` 반환 → Rust 런타임이 코드 1로 종료 |

> **자동화 함의(스크립트 작성자에게):** ntop CLI는 "대상 프로세스가 없었다"와 "성공했다"를 **종료코드로
> 구분하지 않는다.** 결과 판정이 필요하면 **stdout 텍스트(`list --json`의 빈 배열, `info`의 "not found"
> 문자열)를 파싱**해야 한다. 이는 현재 동작의 한계이며 개선 여지는 40-backlog 소유.

---

## 3. 출력 스키마 (Output schema) — `ntop list` 3포맷

> **이 절이 `list` 출력 스키마의 정규 출처다.** 코드 근거: `print_table`/`print_json`/`print_csv`
> (`src/main.rs`). 필드 값의 의미·타입은 `ProcessInfo`(31 §2-1)를 참조.

### 3-1. `table`(기본) — 7컬럼, 사람용

```
 컬럼(폭):  PID(8)  NAME(20)  FRAMEWORK(12)  PORT(10)  CPU(8)  MEM(10)  UPTIME(12)
 ────────────────────────────────────────────────────────────────────────────────
 PID      NAME                 FRAMEWORK    PORT       CPU      MEM        UPTIME
 --------------------------------------------------------------------------------   (80자 구분선)
 51234    next-server          Next.js      3000       2.3%     180.4 MB   1h 2m 5s
 51240      worker             Generic      -          0.0%     45.0 MB    1h 1m 0s   (트리 깊이=들여쓰기)
```

- **★ table에는 FRAMEWORK 컬럼이 있다**(TUI 목록에는 없음 — 정체는 TUI Info 탭에서만, 30 §0·21).
- `NAME`은 `display_name()`(30-2), 트리 깊이만큼 `"  "` 들여쓰기(컬럼 폭은 `20 - 들여쓰기` 보정).
- `PORT`는 LISTEN 포트 콤마(`,`) 결합, 없으면 `-`. `CPU`=`{:.1}%`. `MEM`=`memory_display()`(예: `180.4 MB`).
- 빈 목록 → `"No server processes found."`.

### 3-2. `--json` / `--format json` — 17키 배열, 기계용

> `serde_json` pretty 배열. 각 원소 17키(아래). `--json`은 `--format`보다 우선(§2-0).

| 키 | 타입 | 비고 |
|---|---|---|
| `pid` | number | |
| `ppid` | number | |
| `name` | string | OS 보고 이름(원문, `display_name` 아님) |
| `runtime` | string \| null | `Some`이면 Display(예 `"Node"`), 트리 부모면 `null`(31 R4) |
| `framework` | string | 예 `"Next.js"`·`"Generic"` |
| `framework_version` | string \| null | 현재 항상 `null`(미채움, 31 §2-1) |
| `ports` | number[] | LISTEN 포트 배열 |
| `cpu_percent` | number | 백분율(raw f32) |
| `memory_rss` | number | byte 단위 원시값(macOS는 phys_footprint, 30-10) |
| `memory_display` | string | 사람용(예 `"180.4 MB"`) |
| `uptime_seconds` | number | 초 |
| `uptime_display` | string | 예 `"1h 2m 5s"` |
| `user` | string | |
| `status` | string | 예 `"Running"` |
| `depth` | number | **트리 깊이**(평탄화 시 부여 — 32 고유, 0=루트) |
| `cwd` | string | |
| `command` | string | 전체 커맨드라인 |

> **계약 메모:** `--json`은 `memory_rss`(원시 byte)와 `memory_display`(사람용)를 **둘 다** 준다. `depth`로
> 트리 구조를 복원할 수 있다(들여쓰기 깊이).

### 3-3. `--format csv` — 9컬럼, 표계산용

> `csv` 크레이트 writer. **헤더 1행 + 데이터 행**. 코드 근거: `print_csv`.

```
 헤더: PID, PPID, NAME, FRAMEWORK, PORTS, CPU, MEMORY, UPTIME, STATUS
 ─────────────────────────────────────────────────────────────────────────────
 컬럼      값                                          table/json과 다른 점
 ─────────────────────────────────────────────────────────────────────────────
 PID       proc.pid
 PPID      proc.ppid                                   (table엔 없음)
 NAME      proc.name (원문 — display_name 아님!)        ★ table NAME과 다름
 FRAMEWORK proc.framework (Display)
 PORTS     포트들을 ';'(세미콜론)으로 결합              ★ table는 ',' / CSV는 ';'
 CPU       {:.1}  (★ '%' 기호 없음 — 숫자만)            ★ table는 '%' 포함
 MEMORY    proc.memory_rss (원시 byte 정수)            ★ table는 사람용 "180.4 MB"
 UPTIME    uptime_display ("1h 2m 5s")
 STATUS    proc.status                                 (table엔 없음)
```

- **table/JSON/CSV 차이 요약:** CSV의 `NAME`은 **원문 이름**(table은 `display_name`), `PORTS`는 **세미콜론**
  결합(table은 콤마), `CPU`는 **숫자만**(`%` 없음), `MEMORY`는 **원시 byte**(table은 사람용). 깊이(트리)는
  CSV에 없다(평탄 행만). 이 차이는 실제 코드 동작이므로 자동화 시 주의.

---

## 4. 공개 라이브러리 API (Contract ②) — `ntop::` 크레이트

> `src/lib.rs`가 노출하는 모듈: **`process`·`config`·`log`·`cli`·`tui`**(전부 `pub`). 다른 러스트 프로그램은
> `use ntop::process::scanner::ProcessScanner;`처럼 재사용한다. **타입(구조체/enum)의 필드·정의는
> `31-erd`가 단일 출처**이고, 본 절은 **공개 함수의 시그니처와 계약(반환/부작용)** 만 정의한다.

### 4-1. 핵심 공개 함수 표 (시그니처 = 코드 그대로)

| 타입/모듈 | 공개 함수 시그니처 | 반환/계약 | 처리 알고리즘 |
|---|---|---|---|
| **`ProcessScanner<'a>`** | `new(config: &'a Config) -> Self` | persistent `System` prime(2 refresh + sleep) | 30-3 |
| (scanner.rs) | `scan(&mut self) -> Vec<ProcessInfo>` | 2-pass 분류(runtime=Some/None), 포트는 **미포함** | 30-3 |
| | `scan_blocking(&mut self) -> Vec<ProcessInfo>` | sleep 후 3번째 측정 refresh + `scan`(1회성 CLI용) | 30-3 |
| **`NetworkInspector`** (단위 struct, 상태없음) | `connections_by_pid() -> HashMap<u32, Vec<NetworkConnection>>` | PID→연결들. 명령부재→**빈 HashMap** | 30-5, §5 |
| (network.rs) | `connections_for_pid(pid: u32) -> Vec<NetworkConnection>` | 해당 PID 연결만 | 30-5 |
| | `listening_ports_for_pid(pid: u32) -> Vec<u16>` | LISTEN 포트만 | 30-5 |
| | `parse_addr(s: &str) -> Option<SocketAddr>` | `*:port`/`[::1]:port`/`host:port` 파싱(실패→None) | 30-5 |
| **`ProcessKiller`** (단위 struct) | `send_signal(pid: u32, signal: KillSignal) -> KillResult` | 1회 시그널. `KillResult{Success/AlreadyDead/PermissionDenied/Error}` | 30-8, 33 |
| (killer.rs) | `graceful_kill(pid: u32, timeout: Duration) -> GracefulResult` | SIGTERM→200ms 폴링→timeout. `GracefulResult`(5종) | 30-6, 33 |
| | `force_kill(pid: u32) -> KillResult` | SIGKILL 즉시 | 30-6 |
| | `kill_tree(pids: &[u32], signal: KillSignal) -> Vec<(u32, KillResult)>` | **역순(자식부터)** 시그널 | 30-7, 33 |
| | `is_alive(pid: u32) -> bool` | 생존 확인 | 30-8 |
| **`KillSignal`** (enum) | `all() -> &'static [KillSignal]` | **Unix 6종 / Windows 3종**(31 §3-4) | 33 |
| (killer.rs) | `from_str(s: &str) -> Option<Self>` | `"SIGTERM"`/`"TERM"` 허용(대소문자·`SIG` 무시) | 31 §3-4 |
| | `name(&self) -> &'static str` · `description(&self) -> &'static str` | 예 `"SIGTERM"` / `"Graceful termination request"` | 31 §3-4 |
| **`FrameworkDetector`** | `classify(name: &str, command: &str, config: &Config) -> Option<(Runtime, FrameworkKind)>` | 서버면 `Some`, 아니면 `None`(미표시). **fs 미독** | 30-1, 33 |
| (framework.rs) | | (보조: `detect`/`detect_by_name`/`detect_by_command` 존재) | 30-1 |
| **`Config`** (config.rs) | `load() -> Self` | TOML 로드, 실패→`default()`(부분/부재 안전) | 31 §2-2 |
| | `config_path() -> Option<PathBuf>` | `~/.config/ntop/config.toml` | 31 §4 |
| | `refresh_duration() -> Duration` · `graceful_duration() -> Duration` | 초→Duration 변환 | 31 §4 |
| **`TreeBuilder`** (단위 struct) | `build(flat: Vec<ProcessInfo>) -> Vec<ProcessInfo>` | flat→roots(자식 중첩) | 30-4 |
| (tree.rs) | `flatten_with_depth(trees: &[ProcessInfo]) -> Vec<(&ProcessInfo, usize)>` | DFS (참조, 깊이) | 30-4 |
| | `collect_pids(node: &ProcessInfo) -> Vec<u32>` | 노드+모든 자손 PID(kill_tree 입력) | 30-4·30-7 |
| | `sort_recursive<F>(trees: &mut [ProcessInfo], cmp: &F)` | 레벨별 정렬 | 30-14 |
| **`LogStreamer`** | `detect_and_open(cwd: &str) -> Self` | cwd 글롭→최신 로그 tail(seek end) | 30-9 |
| (streamer.rs) | `new() -> Self` · `Default` | 빈 스트리머 | 30-9 |
| | `poll_new_lines(&mut self) -> Vec<String>` | EOF까지 새 줄, 버퍼≤`MAX_BUFFER_LINES`(31) | 30-9 |
| | `has_source(&self) -> bool` · `source_path(&self) -> Option<&Path>` · `buffer(&self) -> &VecDeque<String>` | 상태 조회 | 30-9 |
| | `detect_and_open_with_proc(cwd: &str, pid: u32) -> Self` | Linux `/proc/<pid>/fd/1` fallback 포함 | 30-9 |

> **`ProcessInfo` 공개 메서드(31 §2-1 소유, 동작은 30):** `new` · `is_server` · `is_node` · `display_name`
> · `uptime_display` · `memory_display` · `health` — 외부 코드도 호출 가능. **`HealthStatus::from_cpu_mem`
> /`from_process_status`** 도 공개(임계 알고리즘 30-12).

### 4-2. 라이브러리 사용 계약(요지)

- **상태성:** `ProcessScanner`는 `&Config` 수명에 묶인 **가변·persistent** 객체(CPU delta 위해 재생성 금지,
  30-3). `NetworkInspector`·`ProcessKiller`·`TreeBuilder`·`FrameworkDetector`는 **상태 없는 단위 struct**로
  연관함수(`::`)만 호출한다.
- **에러 모델:** 시스템 명령 의존부(network)는 **실패해도 패닉하지 않고 빈 컬렉션을 반환**(graceful
  degradation, §5). kill 계열은 결과를 enum(`KillResult`/`GracefulResult`)으로 돌려준다(예외 전파 아님).
- **타입 안정성:** 위 시그니처는 v0.2.0 기준. 타입 정의 변경 시 31-erd가 1순위 갱신 대상(본 문서는 시그니처만).

---

## 5. 의존 시스템 명령 계약 (Contract ③) — ntop이 "호출자"

> ntop은 포트/연결 정보와 일부 종료를 **OS 외부 명령**에 위임한다. 여기서 ntop은 서버가 아니라 **호출자**다.
> 코드 근거: `src/process/network.rs`·`src/process/killer.rs`. 파싱 알고리즘은 30-5 소유.

| 플랫폼 | 호출 명령(정확) | 입력/목적 | 파싱 | 부재·실패 시 |
|---|---|---|---|---|
| **Unix(mac·Linux)** | `lsof -iTCP -sTCP:LISTEN,ESTABLISHED -nP -F pcnT` | TCP LISTEN/ESTABLISHED 연결 | **field-mode**(`-F pcnT`: p=PID, c=cmd, n=주소, T=상태) → `parse_connection`(30-5) | **빈 `HashMap`**(크래시 0) |
| **Windows** | `netstat -ano -p TCP` | TCP 연결 | `TCP ` 줄 공백분해(local·remote·state·pid), `"LISTENING"→"LISTEN"` 정규화 | **빈 `HashMap`** |
| **Windows(종료)** | `taskkill /PID <pid>` | `Term`/`Int` 시그널 종료(`Kill`은 `OpenProcess`+`TerminateProcess` FFI) | stderr 검사: `"not found"`→AlreadyDead, `"Access"`→PermissionDenied | `KillResult::Error` |

### 5-1. ★ 부재·실패 처리 계약 (graceful degradation)

> **명령이 없거나(미설치) 비정상 종료해도 ntop은 크래시하지 않는다.** 코드 그대로:
> - 명령 spawn 실패: `Err(_) => return HashMap::new()` (network.rs L104·L200)
> - 종료 상태 비성공: `if !output.status.success() { return HashMap::new() }` (L107-108·L203-204)

```
 NetworkInspector::connections_by_pid()
        │
        ▼  Command::new("lsof"/"netstat") 실행
   ┌────┴──────────────────────┐
   │ spawn 실패? ─── 예 ───────▶ HashMap::new()  (빈 결과)
   │      │아니오               │
   │ status 비성공? ─ 예 ──────▶ HashMap::new()  (빈 결과)
   │      │성공                                   ↓
   │ stdout 파싱(30-5)                    영향: 포트 컬럼·Net 탭만 빔
   └──────┴──────────────────────────────────────  (그 외 기능 정상)
```

→ 결과: `lsof`/`netstat`가 없으면 **포트(`ProcessInfo.ports`)와 Net 탭이 비는 정도**로만 저하되고, 식별·
종료·로그 등 나머지 기능은 정상 동작한다. (안정성 NFR은 10-prd §5, 외부 명령 의존 한계는 40-backlog.)

---

## 6. 시각화 (G-g) — CLI 호출 시퀀스 다이어그램 (외부 렌더러 0)

### 6-1. `ntop list` 호출 시퀀스 (CLI → Scanner → NetworkInspector → 시스템명령 → 출력)

```
 사용자/스크립트     main.rs        ProcessScanner    NetworkInspector    OS 명령(lsof/netstat)   stdout
   │                  │                 │                  │                    │                  │
   │ $ ntop list --json                 │                  │                    │                  │
   ├─────────────────▶│                 │                  │                    │                  │
   │            Config::load()          │                  │                    │                  │
   │            (~/.config/ntop, 부재→기본 31)              │                    │                  │
   │                  │ new(&config)    │                  │                    │                  │
   │                  ├────────────────▶│ prime: refresh×2 + sleep             │                  │
   │                  │ scan_blocking() │                  │                    │                  │
   │                  ├────────────────▶│ 측정 refresh + 2-pass classify(30)   │                  │
   │                  │                 │ (fs 미독 — 이름+커맨드만, 33)        │                  │
   │                  │◀────Vec<ProcessInfo> (runtime Some/None, 포트 없음)    │                  │
   │                  │ connections_by_pid()               │                    │                  │
   │                  ├───────────────────────────────────▶│ Command::new("lsof"…)               │
   │                  │                 │                  ├───────────────────▶│ -iTCP -F pcnT    │
   │                  │                 │                  │◀── stdout / (부재→빈)│                  │
   │                  │◀── HashMap<pid, Vec<NetworkConnection>> (LISTEN만 추려 ports 채움, 30)     │
   │                  │ TreeBuilder::build + flatten_with_depth (30-4)          │                  │
   │                  │ print_json(flat)  17키 + depth (§3-2)                   │                  │
   │                  ├───────────────────────────────────────────────────────────────────────────▶│
   │◀─────────────────┴──────────────── exit 0 ────────────────────────────────────────────────────┤
```

### 6-2. `ntop kill <PID>` 단건 시퀀스 (graceful→force 자동 에스컬레이션, CLI 전용)

```
 사용자        main.rs(cmd_kill)      ProcessKiller            대상 프로세스
   │              │                       │                        │
   │ $ ntop kill 51234                    │                        │
   ├─────────────▶│ confirm_before_kill?(31) & !--no-confirm       │
   │              │  → "Kill 51234? [y/N]" (정책 33)               │
   │◀── y ────────┤                       │                        │
   │              │ graceful_kill(pid, graceful_duration()=10s 31)  │
   │              ├──────────────────────▶│ SIGTERM ──────────────▶│ (정리 시도)
   │              │                       │ 200ms 폴링 is_alive…    │
   │              │                       │   ├ 죽음 → Terminated   │
   │              │                       │   └ 10s 초과 → TimedOut │
   │              │◀── GracefulResult ────┤                        │
   │              │ TimedOut? → "Force killing..."                  │
   │              │ force_kill(pid) ─────▶│ SIGKILL ──────────────▶│ (강제 종료)
   │◀── 결과 메시지 + exit 0 ─────────────┴────────────────────────┤
   (※ TUI KillConfirm 경로는 이 폴링/자동 force를 쓰지 않음 — send_signal(SIGTERM) 직접 1회. 정책·경로 구분 = 33)
```

### 6-3. 인터페이스 계약 한눈 요약

```
 ┌── Contract ① CLI ────────────────┬── Contract ② lib(ntop::) ──────┬── Contract ③ syscmd ──────┐
 │ ntop / list / kill / info / log / │ ProcessScanner.scan(_blocking)  │ lsof -iTCP …-F pcnT (unix) │
 │ config                            │ NetworkInspector.connections_*  │ netstat -ano -p TCP (win)  │
 │ 플래그: --json --format --tree     │ ProcessKiller.send/graceful/    │ taskkill /PID <pid> (win)  │
 │        --signal --all --no-confirm │   force/kill_tree/is_alive      │                            │
 │ 출력: table7 / JSON17 / CSV9       │ FrameworkDetector.classify      │ 부재/실패 → 빈 결과         │
 │ 종료코드: 0(대부분)/2(파싱)/1(런타임)│ Config.load / TreeBuilder.build │   (크래시 0, 포트만 빔)    │
 │                                   │ LogStreamer.detect_and_open     │                            │
 └───────────────────────────────────┴─────────────────────────────────┴────────────────────────────┘
   값 정의 → 31-erd · 동작 → 30-functional-spec · 안전/프라이버시 → 33-policy · 키/화면 → 21-screen-spec
```

---

## 자체 검증 (Self-verification)

| # | 체크 | 결과 | 어디서 충족 |
|---|---|---|---|
| **A-1** | CLI 서브커맨드/플래그/필수성 | **PASS** — §2-0 표(5서브+TUI, `PID` required_unless_present="all", 플래그 상호작용) + §2-1~2-6 | §2 |
| **A-2** | 종료코드 계약(0/2/1, not-found·cancelled=0) | **PASS** — §2-7 표 + 자동화 함의(stdout 파싱 필요) | §2-7 |
| **A-3** | 출력 스키마 table7/JSON17+depth/CSV9 | **PASS** — §3-1·§3-2·§3-3, table/JSON/CSV 차이(NAME·구분자·CPU·MEMORY) 명시 | §3 |
| **A-4** | 공개 lib API 시그니처 | **PASS** — §4-1 표(Scanner·Network·Killer·KillSignal·FrameworkDetector·Config·TreeBuilder·LogStreamer 코드 그대로), 타입은 31 회부 | §4 |
| **A-5** | 시스템명령 계약 + 부재→빈 결과 | **PASS** — §5 표(lsof/netstat/taskkill 정확 인자) + §5-1 graceful degradation 플로우(코드 L104/107) | §5 |
| **A-6** | 시각화(시퀀스 + 표) | **PASS** — §6-1 list 시퀀스·§6-2 kill 에스컬레이션 시퀀스·§6-3 요약 + §2/§3 표(mermaid 0) | §6, §2, §3 |
| **A-7** | 단일 출처 + 창작 0 | **PASS** — 값=31·동작=30·정책=33 참조, HTTP/REST/DB/결제 0(§0 콜아웃), 시그니처/플래그/스키마는 소스 추출 | 전체 |

**추가 원칙 점검:**
- **창작 금지(spec §7):** REST/GraphQL/URL/메서드/상태코드/DB/회원/결제 **0**(§0 명시). 모든 명령·플래그·
  스키마·시그니처는 실제 소스(`cli.rs`·`main.rs`·`lib.rs`·`network.rs`·`killer.rs`·`framework.rs`·
  `config.rs`·`tree.rs`·`streamer.rs`)에서 추출.
- **코드-진실 정직 표기:** ① 종료코드는 거의 항상 0(실패도 메시지+`Ok`) — "kill 실패 시 1" 같은 동작
  없음(§2-7); ② `ntop info`의 env는 **마스킹 안 됨**(TUI만 마스킹, §2-3·33); ③ `--signal` 인식 실패 시 조용히
  SIGTERM 폴백; ④ CSV는 `display_name`이 아니라 원문 `name`·세미콜론 포트·`%` 없는 CPU·원시 byte MEMORY(§3-3).
- **중복정의 0:** `graceful_timeout=10`·`MAX_BUFFER_LINES=1000`·Config 기본값은 값으로 적지 않고 "31값" 인용,
  알고리즘은 "30 참조", 정책은 "33 참조".
- **쉬운 설명:** CLI·서브커맨드·플래그·종료코드·스키마·크레이트·시그니처·stdout·field-mode 첫 등장 풀이.

**알려진 한계(후속 보완):**
- 종료코드가 결과(성공/미발견)를 구분하지 않음 → 자동화는 stdout 파싱 필요(개선 = 40-backlog).
- `ntop info`의 env 미마스킹은 프라이버시 갭(개선 후보 = 40-backlog, 정책 단일 출처 33).
- 시그니처/스키마는 v0.2.0 기준 — 코드 변경 시 본 문서가 따라가야 하며 타입 정의는 31-erd가 1순위.
