# 31 · ERD / 데이터 모델 — 인메모리 도메인 모델 + Config 기본값 (ntop)

> **이 문서는 단일 출처(single source)다.** ntop의 **구조체·열거형(enum) 정의**와 **Config 기본값**
> (`refresh_interval`·`graceful_timeout`·`MAX_BUFFER_LINES` 등)은 **여기서만 값으로 확정**한다.
> 동작·알고리즘을 다루는 `30-functional-spec`, 외부 계약을 다루는 `32-api-spec`, 안전·프라이버시 정책을
> 다루는 `33-policy`, 화면/키를 다루는 `21-screen-spec`은 이 문서의 타입·값을 **참조**할 뿐 재정의하지
> 않는다(spec §4-3).
>
> **전제:** ntop은 **이미 v0.2.0까지 구현된 MIT 오픈소스 도구**다(codebase-facts §0). 따라서 이 ERD는
> "앞으로 설계할 데이터베이스"가 아니라 **이미 구현된 소스코드의 인메모리 타입을 역설계해 그린 그림**이다.
> 모든 필드명·타입·기본값은 실제 코드(`src/process/mod.rs`·`src/config.rs`·`src/process/network.rs`·
> `src/log/streamer.rs`·`src/tui/app.rs`·`src/process/killer.rs`)에서 그대로 추출했다(창작 0).
>
> **용어 풀이(첫 등장):** **ERD**(Entity-Relationship Diagram) = 데이터가 어떤 덩어리(엔티티)로 나뉘고
> 서로 어떻게 연결되는지 그린 그림 / **인메모리(in-memory)** = 디스크/DB에 저장하지 않고 프로그램이
> 도는 동안 메모리에만 들고 있다가 종료하면 사라지는 / **구조체(struct)** = 여러 필드를 한 덩어리로 묶은
> 데이터 타입 / **열거형(enum)** = 정해진 몇 가지 값 중 하나만 갖는 타입(예: 신호등 = 빨강·노랑·초록) /
> **1:N 관계** = 하나가 여러 개를 거느리는 관계(부모 1 ↔ 자식 여러) / **자기참조(self-reference)** = 같은
> 타입을 자기 안에 다시 품는 것(프로세스가 자식 프로세스를 품음) / **Option<T>** = 값이 있을 수도(`Some`)
> 없을 수도(`None`) 있음을 타입으로 표현 / **PID** = 프로세스 고유 번호.

---

## Sprint Contract (self-proposed checks)

이 문서(g4의 `31-erd`)가 만족시킬 **관찰 가능한 체크**(playbook §S4-g4 관찰 바 + spec §5(31행)·§8에서 도출).

| # | 체크 | 어떻게 충족하나 | 본문 위치 |
|---|---|---|---|
| **E-1** | **DB 없음 명시** — 데이터베이스가 아니라 인메모리 도메인 모델임을 분명히 | §0 "DB 없음" 콜아웃 + 전체 프레이밍 | §0 |
| **E-2** | **핵심 구조체 전 필드(타입 포함)** — `ProcessInfo`·`Config`(+3 하위)·`NetworkConnection`·`LogStreamer`·`App`·`Rule` | §2 엔티티 명세(필드명·타입·코드 근거) | §2 |
| **E-3** | **enum 값 전부** — `Runtime`(8)·`FrameworkKind`(15)·`HealthStatus`(3)·`KillSignal`·`KillResult`(4)·`GracefulResult`(5)·`DetailTab`(4)·`DialogKind`(5)·`FocusPanel`(2)·`SortColumn`(9) | §3 enum 값 테이블(+Display/라벨) | §3 |
| **E-4** | **Config 기본값 단일 출처** — refresh=3·signal=SIGTERM·graceful=10·confirm=true·show_tree=true·theme=auto·mask=true·include_*=false + MAX_BUFFER_LINES=1000 | §4 Config 기본값 정규표 | §4 |
| **E-5** | **관계 명세** — App↔ProcessInfo(1:N flat+tree)·children 자기참조 1:N·ProcessInfo↔Runtime/FrameworkKind·App↔Config·NetworkConnection↔pid·App↔LogStreamer | §5 관계 표 + §1 ERD 다이어그램 | §1, §5 |
| **E-6** | **시각화(G-g)** — ERD 다이어그램(엔티티 박스 + 1:N 관계선, crow's-foot ASCII) + enum 값 테이블 | §1 ERD + §3 표(외부 렌더러 0) | §1, §3 |
| **E-7** | **단일 출처 준수** — 동작/알고리즘 임계값은 30, 정책 규칙은 33, 키 바인딩은 21로 회부(값만 여기서 정의) | 본문 곳곳 "→ 30/33/21" 참조 | 전체 |

---

## 0. ★ 데이터베이스 없음 — 인메모리 도메인 모델

> **ntop에는 데이터베이스가 없다.** 회원 테이블도, SQL도, ORM도, 영속 저장소도 없다(codebase-facts §0-1).
> ntop은 **로컬 개발자 도구(CLI/TUI)**로, 매 스캔 tick마다 OS에서 프로세스 상태를 읽어 **메모리 위 구조체**로
> 들고 있다가 종료하면 전부 사라진다. **유일하게 디스크에 남는 것은 사용자 설정 파일 1개**
> (`~/.config/ntop/config.toml`)뿐이며, 이것도 "데이터"가 아니라 "설정"이다.

```
 ┌──────────────────────────────────────────────────────────────────────────┐
 │  ntop 데이터의 생애 (DB 아님)                                              │
 │                                                                            │
 │   OS 커널 ──refresh()──▶  [메모리 위 구조체]  ──종료 시 소멸──▶ (없음)     │
 │  (sysinfo,lsof…)          ProcessInfo, App…                                │
 │                                                                            │
 │   디스크에 남는 유일한 파일:  ~/.config/ntop/config.toml  (설정만, 선택적) │
 │      └ 없거나 깨져도 기본값으로 동작(serde default) → §4                    │
 └──────────────────────────────────────────────────────────────────────────┘
```

따라서 본 ERD는 "테이블·컬럼·인덱스·외래키"가 아니라 **Rust 구조체·열거형과 그들 사이의 소유/참조
관계**를 ERD 문법(엔티티 박스 + 1:N 관계선)으로 옮겨 그린 것이다. "관계"는 DB의 외래키가 아니라 **코드상의
소유(struct가 다른 struct를 필드로 가짐)·자기참조·논리적 조인(같은 `pid`로 연결)**을 뜻한다.

---

## 1. ERD 다이어그램 (G-g) — 엔티티 박스 + 1:N 관계선

> crow's-foot(까마귀발) 표기를 ASCII로 옮겼다. **`||` = "정확히 1"**, **`o<` / `}<` = "여러(N), 0개 이상"**,
> **`──` = 소유(struct가 필드로 가짐)**, **`- - -` = 논리적 조인(같은 pid로 연결, 소유 아님)**, **`▷` =
> enum 값을 가리킴(참조)**. 화살표 방향은 "1 → N".

```
                       ┌────────────────────────────────────────────────┐
                       │                     App                         │   src/tui/app.rs
                       │  TUI 단일 상태 구조체 (프로그램당 1개, 인메모리) │   (= 애그리거트 루트)
                       │  config · raw_processes · process_trees ·        │
                       │  flat_list · selected_index · selected_pids ·    │
                       │  expanded_pids · active_tab · dialog · filter_* · │
                       │  sort_* · node_only · log_streamer · system_* …  │
                       └──┬──────────────┬───────────────┬───────────┬────┘
                  1 ||    │          1 ||│          1 ||  │       1 ||│ 0..1
                          │ 소유          │ 소유 1:N        │ 소유       │ 소유(Option)
                          ▼              ▼ }<              ▼ }<         ▼ o<
                   ┌────────────┐  ┌───────────────────────────┐  ┌──────────────┐
                   │   Config   │  │        ProcessInfo        │  │  LogStreamer │  src/log/
                   │ src/config │  │     src/process/mod.rs    │  │  streamer.rs │
                   └─────┬──────┘  │  pid·ppid·name·command·   │  │ file·path·   │
            1 || 소유 1:3│         │  cwd·framework·ports·     │  │ buffer       │
        ┌────────┬───────┴───┐     │  cpu_percent·memory_*·    │  └──────────────┘
        ▼ ||     ▼ ||        ▼ ||  │  threads·uptime·user·     │   buffer 상한:
 ┌────────────┐┌──────────┐┌──────┐│  status·open_fds·         │   MAX_BUFFER_LINES
 │GeneralConfig││Display   ││Filter││  env_vars·runtime·        │   = 1000 (§4)
 │ refresh=3   ││Config    ││Config││  children ◄──┐            │
 │ signal=TERM ││tree=true ││bun=F ││  ────────────┘ 자기참조   │
 │ graceful=10 ││theme=auto││tsx=F ││  1:N (자식도 ProcessInfo) │
 │ confirm=true││mask=true ││tsn=F ││  }< self                  │
 └────────────┘└──────────┘└──────┘└───┬───────────┬───────────┘
                                        │ ▷ 참조     │ ▷ 참조      ┌───────────────────┐
                                        ▼ N:1        ▼ N:1         │ NetworkConnection │ network.rs
                                  ┌──────────┐  ┌──────────────┐   │ local_addr·       │
                                  │ Runtime  │  │ FrameworkKind│   │ remote_addr·      │
                                  │ (enum 8) │  │  (enum 15)   │   │ state·pid         │
                                  │ Option<> │  └──────────────┘   └─────────┬─────────┘
                                  └──────────┘                               │ pid (u32)
                                                                             ┊ 논리적 조인
                          ProcessInfo.pid  - - - - - - - - - - - - - - - - - ┘ (do_scan이
                                                                 같은 pid로 ports 채움 → 30)
```

> **읽는 법:** `App`이 모든 것을 소유하는 **애그리거트 루트**(전체 상태의 꼭대기)다. `App` 1개가
> `ProcessInfo` 여러 개를 두 형태로 들고 있다 — 평탄 목록(`raw_processes`/`flat_list`)과 트리
> (`process_trees`). `ProcessInfo`는 `children` 필드로 **자기 자신을 다시 품어**(자기참조 1:N) 부모-자식
> 트리를 만든다. `ProcessInfo`는 `Runtime`·`FrameworkKind` enum을 가리킨다(`runtime`은 `Option` — 서버면
> `Some`, 트리 부모면 `None`). `NetworkConnection`은 `ProcessInfo`의 필드가 아니라 **같은 `pid`로 논리적
> 연결**되며, 스캔 후 `do_scan`이 LISTEN 포트만 골라 `ProcessInfo.ports`에 채운다(처리 절차는 30 소유).

---

## 2. 엔티티 명세 (구조체)

> 각 표는 **필드명 · 타입 · 의미 · 코드 근거**. 타입은 Rust 원본 그대로다. 메서드는 "동작"이므로 상세는
> `30-functional-spec`이 소유하고, 여기서는 존재만 나열한다.

### 2-1. `ProcessInfo` — 모니터링 대상 프로세스 (핵심 엔티티) · `src/process/mod.rs`

| 필드 | 타입 | 의미 |
|---|---|---|
| `pid` | `u32` | 프로세스 고유 번호 |
| `ppid` | `u32` | 부모 프로세스 번호(트리 구성 키). 0 = 부모 없음/루트 |
| `name` | `String` | OS가 보고한 프로세스 이름(macOS는 `comm` 16자 절단 가능 → 30 `normalize_name`) |
| `command` | `String` | 전체 커맨드라인(공백 join). 분류·`display_name`의 입력 |
| `cwd` | `String` | 작업 디렉터리. 로그 감지(`LogStreamer`)에만 사용, **분류엔 쓰지 않음**(30·33) |
| `framework` | `FrameworkKind` | 탐지된 프레임워크(미탐지=`Generic`) → §3 |
| `framework_version` | `Option<String>` | 프레임워크 버전(현재 항상 `None` — detect가 버전 미추출) |
| `ports` | `Vec<u16>` | LISTEN 포트 목록. 스캔 후 `NetworkInspector`가 별도로 채움(30) |
| `cpu_percent` | `f32` | CPU 사용률(%). persistent `System`의 2-refresh delta(30) |
| `memory_rss` | `u64` | 실제 사용 메모리(byte). macOS는 `phys_footprint` 우선(30) |
| `memory_vms` | `u64` | 가상 메모리(byte) |
| `threads` | `u32` | 스레드 수(platform FFI) |
| `uptime` | `Duration` | 구동 시간 |
| `user` | `String` | 소유 사용자(user id 문자열, 실패 시 `"unknown"`) |
| `status` | `String` | 프로세스 상태 문자열(예: `"Running"`,`"Sleeping"`,`"Zombie"`) → health 판정 입력(30) |
| `open_fds` | `u32` | 열린 파일 디스크립터 수(macOS/Linux, Windows=0) |
| `children` | `Vec<ProcessInfo>` | **자식 프로세스들 — 자기참조 1:N(트리)** |
| `env_vars` | `Vec<(String, String)>` | 환경변수 (키,값) 쌍. Env 탭에서 민감값 마스킹(33) |
| `runtime` | `Option<Runtime>` | **`Some`=분류된 서버 / `None`=트리 컨텍스트 부모**(launchd·claude·shell). `is_server()`/`is_node()`/Node-only 필터의 키 |

**메서드(동작은 30 소유):** `new(pid,name)` · `is_server()`(=`runtime.is_some()`) · `is_node()`
(=`runtime==Some(Node)`) · `uptime_display()`("1h 2m 5s") · `memory_display()`("128.0 MB"/"1.5 GB") ·
`display_name()`(커맨드에서 사람이 읽을 이름 추출 → 30-2) · `health()`(→ 30-12).

> **주의(코드-진실):** `framework_version`은 구조체에 존재하나 현재 탐지기가 채우지 않아 항상 `None`이다
> (40-backlog 로드맵 후보). 창작이 아니라 실제 코드 상태다.

### 2-2. `Config` 및 하위 3섹션 — 사용자 설정 · `src/config.rs`

`Config`는 3개 하위 구조체를 1:1로 소유한다. **모든 섹션·필드에 `#[serde(default)]`** 가 붙어, 설정 파일이
없거나 일부만 있거나 파싱 실패해도 **각 필드가 기본값으로 채워진다**(`load()`는 실패 시 `Config::default()`).

| 구조체 | 필드 | 타입 | 기본값(§4 정규) |
|---|---|---|---|
| **`Config`** | `general` / `display` / `filter` | 각 하위 구조체 | 각 `default()` |
| **`GeneralConfig`** | `refresh_interval` | `u64` | `3` (초) |
| | `default_signal` | `String` | `"SIGTERM"` |
| | `graceful_timeout` | `u64` | `10` (초) |
| | `confirm_before_kill` | `bool` | `true` |
| **`DisplayConfig`** | `show_tree` | `bool` | `true` |
| | `color_theme` | `String` | `"auto"` (auto·dark·light) |
| | `mask_env_values` | `bool` | `true` |
| **`FilterConfig`** | `include_bun` | `bool` | `false` (**DEPRECATED** — Bun은 1급 런타임, 항상 탐지) |
| | `include_tsx` | `bool` | `false` (tsx dev runner 포함 토글) |
| | `include_ts_node` | `bool` | `false` (ts-node dev runner 포함 토글) |

**경로/메서드:** `config_path()` → `dirs::config_dir()/ntop/config.toml`(= `~/.config/ntop/config.toml`) ·
`load()` · `refresh_duration()`(=`refresh_interval`초 → `Duration`) · `graceful_duration()`
(=`graceful_timeout`초 → `Duration`).

### 2-3. `NetworkConnection` — TCP 연결 1건 · `src/process/network.rs`

| 필드 | 타입 | 의미 |
|---|---|---|
| `local_addr` | `SocketAddr` | 로컬 주소+포트(예: `0.0.0.0:3000`) |
| `remote_addr` | `Option<SocketAddr>` | 원격 주소(LISTEN은 보통 `None`, ESTABLISHED는 `Some`) |
| `state` | `String` | `"LISTEN"`·`"ESTABLISHED"` 등(Windows `"LISTENING"`→`"LISTEN"` 정규화는 30) |
| `pid` | `u32` | 이 연결을 소유한 프로세스 — `ProcessInfo.pid`와 **논리적 조인 키** |

`NetworkInspector`(구조체, 상태 없음)가 시스템 명령(lsof/netstat) 출력을 파싱해 `HashMap<u32, Vec<NetworkConnection>>`
(PID→연결들)을 만든다. 파싱 알고리즘은 30, 외부 명령 계약은 32 소유.

### 2-4. `LogStreamer` — 로그 tail 상태 · `src/log/streamer.rs`

| 필드 | 타입 | 의미 |
|---|---|---|
| `file` | `Option<BufReader<File>>` | 열린 로그 파일 리더(없으면 `None`) |
| `path` | `Option<PathBuf>` | 열린 로그 파일 경로 |
| `buffer` | `VecDeque<String>` | 최근 줄 버퍼(앞에서 폐기, 뒤에 추가) |

**상수(이 엔티티의 정규값):** `MAX_BUFFER_LINES = 1000` — 버퍼가 1000줄을 넘으면 가장 오래된 줄부터 폐기
(메모리 상한). 글로벌 글롭 패턴 목록 `LOG_PATTERNS`와 tail 알고리즘은 30 소유.

### 2-5. `App` — TUI 단일 상태(애그리거트 루트) · `src/tui/app.rs`

> 모든 UI/런타임 상태를 담는 **단 하나의 구조체**(프로그램당 1개). 필드를 성격별로 묶었다.

| 분류 | 필드(타입) | 의미 |
|---|---|---|
| 설정 | `config: Config` | 로드된 설정(소유, 1:1) |
| 데이터(원본) | `raw_processes: Vec<ProcessInfo>` | 마지막 스캔 결과(필터 전) |
| 데이터(표시) | `process_trees: Vec<ProcessInfo>` · `flat_list: Vec<(ProcessInfo, usize)>` | 필터·정렬·평탄화된 트리/목록(`usize`=깊이) |
| 선택 | `selected_index: usize` · `selected_pids: HashSet<u32>` · `expanded_pids: HashSet<u32>` | 커서 행 · 다중선택(Space) · 확장된 노드 |
| 상세 탭 | `active_tab: DetailTab` | 현재 상세 탭 → §3 |
| 다이얼로그 | `dialog: Option<DialogKind>` · `signal_picker_index: usize` | 열린 모달 · 시그널 선택 인덱스 → §3 |
| 필터 | `filter_text: String` · `filter_active: bool` | 필터 문자열 · 입력 모드 여부 |
| 정렬 | `sort_column: SortColumn` · `sort_ascending: bool` | 정렬 컬럼 · 방향 → §3 |
| 로그 | `log_streamer: Option<LogStreamer>` · `log_scroll: u16` | 로그 스트리머(소유 0..1) · 스크롤 |
| 상세 스크롤 | `detail_scroll`/`detail_content_lines`/`detail_view_height: u16` | 렌더 중 계산되는 스크롤 경계 |
| 시스템 | `system_cpu: f32` · `system_memory_used/total: u64` | 전체 시스템 지표(상단바) |
| kill 진행 | `kill_in_progress: Option<(u32, Instant)>` | 진행 중 kill(대상 pid, 시작 시각) |
| 포커스/뷰 | `focus: FocusPanel` · `node_only: bool` | 포커스 패널 · Node-only 토글 → §3 |
| 루프 제어 | `should_quit`·`needs_rescan`·`refresh_changed: bool` · `refresh_secs: u64` · `tick_count: u64` · `first_load: bool` | 종료/재스캔/주기변경 플래그 · 현재 주기 · 틱(스피너) · 첫 로드 |
| 도움말 | `help_scroll`/`help_max_scroll: u16` | 도움말 스크롤 |
| 테이블 | `table_state: TableState` | ratatui 테이블 스크롤 상태 |

**대표 메서드(동작은 30):** `matches_filter()`(30-13) · `update_processes()`/`rebuild_view()`(30-3·30-15) ·
`toggle_expand`/`expand_selected`/`collapse_selected`/`toggle_expand_all`(30-4) · `toggle_select` ·
`next_tab`/`prev_tab` · `toggle_sort`(30-14) · `toggle_node_only`(30-15) · `selected_kill_signal`.

### 2-6. `Rule` — 탐지 규칙 1건 (확장 포인트) · `src/process/framework_rules.rs`

| 필드 | 타입 | 의미 |
|---|---|---|
| `runtime` | `Runtime` | 이 규칙이 부여할 런타임 |
| `framework` | `FrameworkKind` | 이 규칙이 부여할 프레임워크 |
| `name_exact` | `&'static [&'static str]` | 정규화 이름 정확 일치 후보(매처 1순위) |
| `command_binary` | `&'static [&'static str]` | 커맨드 첫 토큰의 basename 정확 일치(매처 2순위) |
| `command_contains` | `&'static [&'static str]` | 전체 커맨드라인 부분 문자열(매처 3순위) |

규칙 테이블 2개(`FRAMEWORK_RULES`·`RUNTIME_RULES`)와 매칭 순서는 30-1이 소유(값 표 포함). 새 런타임/FW
추가 = `Rule` 1개 + enum variant 1개(그 외 코드 변경 0).

---

## 3. enum 값 테이블 (G-g)

> 각 enum의 **variant 전체 + Display/라벨/설명**. 가용성 매트릭스(OS별 시그널)·임계값(health)·키 매핑 등
> "규칙/동작"은 33(정책)·30(알고리즘)·21(키)이 소유하고, 여기서는 **값만** 확정한다.

### 3-1. `Runtime` (8종) · `src/process/mod.rs`

| variant | Display | variant | Display |
|---|---|---|---|
| `Node` | `Node` | `Bun` | `Bun` |
| `Python` | `Python` | `Ruby` | `Ruby` |
| `Java` | `Java` | `Php` | `PHP` |
| `Deno` | `Deno` | `DotNet` | `.NET` |

> `ProcessInfo.runtime`은 `Option<Runtime>` — `None`이면 "분류 안 된 트리 부모"(서버 아님). 등록 규칙은 30-1.

### 3-2. `FrameworkKind` (15종) · `src/process/mod.rs`

| variant | Display | 규칙 등록? | variant | Display | 규칙 등록? |
|---|---|---|---|---|---|
| `NextJs` | `Next.js` | ✅ | `FastApi` | `FastAPI` | ✅ |
| `Express` | `Express` | ❌ enum만 | `Flask` | `Flask` | ✅ |
| `Fastify` | `Fastify` | ❌ enum만 | `Django` | `Django` | ✅ |
| `NestJs` | `NestJs` | ✅ | `SpringBoot` | `Spring Boot` | ✅ |
| `Nuxt` | `Nuxt.js` | ✅ | `Rails` | `Rails` | ✅ |
| `Koa` | `Koa` | ❌ enum만 | `Laravel` | `Laravel` | ✅ |
| `Hapi` | `Hapi` | ❌ enum만 | `AspNet` | `ASP.NET` | ✅ |
| | | | `Generic` | `Generic` | (런타임 규칙 기본값) |

> **코드-진실:** `Express`·`Fastify`·`Koa`·`Hapi`는 enum variant만 있고 `FRAMEWORK_RULES`에 탐지 규칙이
> **미등록**이다 → 현재 탐지 불가, 향후 규칙 추가 여지(40-backlog 로드맵, codebase-facts §12). `Generic`은
> 런타임 규칙이 부여하는 기본 프레임워크 값이다.

### 3-3. `HealthStatus` (3종) · `src/process/mod.rs`

| variant | Display | 의미 |
|---|---|---|
| `Healthy` | `Healthy` | 정상 |
| `Warning` | `Warning` | 주의 |
| `Critical` | `Critical` | 위험 |

> 판정 임계값(CPU/MEM ≥90→Critical, ≥80→Warning / Zombie·Dead→Critical)과 `from_cpu_mem`·
> `from_process_status`·`health()` 알고리즘은 **30-12**가 소유.

### 3-4. `KillSignal` + 결과 enum · `src/process/killer.rs`

**`KillSignal`** — 보낼 수 있는 시그널. **크로스플랫폼 3종 + Unix 전용 3종**:

| variant | name() | description() | 가용 OS |
|---|---|---|---|
| `Term` | `SIGTERM` | Graceful termination request | 전체 |
| `Kill` | `SIGKILL` | Force kill (cannot be caught) | 전체 |
| `Int` | `SIGINT` | Interrupt (like Ctrl+C) | 전체 |
| `Hup` | `SIGHUP` | Hangup / reload configuration | Unix만 |
| `Usr1` | `SIGUSR1` | User-defined (Node.js: activate debugger) | Unix만 |
| `Usr2` | `SIGUSR2` | User-defined signal | Unix만 |

`all()` 반환 개수·순서: **Unix = 6종**[Term,Kill,Hup,Int,Usr1,Usr2] / **Windows = 3종**[Term,Kill,Int].
`from_str`은 `"SIGTERM"`·`"TERM"` 둘 다 허용(대소문자·`SIG` 접두 무시). **OS×시그널 가용성 매트릭스와
권한(EPERM) 정책은 33-policy 소유.**

**`KillResult`** (단건 시그널 결과): `Success` · `AlreadyDead` · `PermissionDenied` · `Error(String)`.

**`GracefulResult`** (graceful_kill 결과): `Terminated` · `TimedOut` · `AlreadyDead` · `PermissionDenied` ·
`Error(String)`. 분기 처리 알고리즘은 **30-6**.

### 3-5. TUI enum · `src/tui/app.rs`

| enum | variant | 비고 |
|---|---|---|
| **`DetailTab`** (4) | `Info` · `Log` · `Net` · `Env` | 라벨 동일. `next()`/`prev()` 순환 |
| **`DialogKind`** (5) | `KillConfirm` · `KillTreeConfirm` · `SignalPicker` · `ForceKillPrompt` · `Help` | 열린 모달 종류 |
| **`FocusPanel`** (2) | `ProcessList` · `DetailPanel` | 키보드 포커스 위치 |
| **`SortColumn`** (9) | `Pid` · `Name` · `Port` · `Threads` · `Cpu` · `Memory` · `User` · `Status` · `Uptime` | `next()` 순환 |

**`SortColumn` 라벨**(목록 헤더): `PID` · `NAME` · `PORT` · `THR` · `CPU` · `MEM` · `USER` · `STATUS` ·
`UPTIME`. (목록 화면에서 Status 컬럼은 `STS`로 약칭되나 `SortColumn::Status.label()`은 `"STATUS"` — 21
참조.) 탭 전환/정렬 순환 동작은 30-14, 키 매핑은 21 소유.

---

## 4. ★ Config 기본값 — 단일 출처 정규표

> **이 표가 ntop의 모든 기본값·임계 상수의 정규 출처다.** `10-prd`(NFR)·`30`(알고리즘 임계)·`33`(정책)·
> `21`(화면)은 같은 수치를 **이 표에서 인용만** 하고 재정의하지 않는다. 출처 코드는 `GeneralConfig/
> DisplayConfig/FilterConfig::default()`(`src/config.rs`)와 `LogStreamer`(`src/log/streamer.rs`).

| 키 | 정규 기본값 | 의미 | 주 참조 |
|---|---|---|---|
| `general.refresh_interval` | **`3`** 초 | 스캔 tick 주기(런타임 `+`/`-`로 1~60s 조정 가능) | 30-3, 10-prd NFR(성능) |
| `general.default_signal` | **`"SIGTERM"`** | 기본 종료 시그널 | 30-6, 33 |
| `general.graceful_timeout` | **`10`** 초 | graceful 종료 대기 한계(초과 시 force) | 30-6, 33 |
| `general.confirm_before_kill` | **`true`** | 파괴적 동작 전 확인 게이트 | 33, 10-prd NFR(안전) |
| `display.show_tree` | **`true`** | 트리 뷰 표시 | 21 |
| `display.color_theme` | **`"auto"`** | 테마(auto·dark·light) | 21 |
| `display.mask_env_values` | **`true`** | 환경변수 민감값 마스킹 | 33, 30-11 |
| `filter.include_bun` | **`false`** | **DEPRECATED**(Bun 항상 탐지) | 30-1 |
| `filter.include_tsx` | **`false`** | tsx dev runner 포함 토글 | 30-1 |
| `filter.include_ts_node` | **`false`** | ts-node dev runner 포함 토글 | 30-1 |
| `LogStreamer::MAX_BUFFER_LINES` | **`1000`** 줄 | 로그 버퍼 상한(상수, Config 아님) | 30-9, 10-prd NFR(성능) |

> **알고리즘 전용 상수(Config 아님, 참고):** graceful 폴링 간격 `200ms`, sysinfo `MINIMUM_CPU_UPDATE_INTERVAL`
> (prime sleep)은 **동작 파라미터**이므로 값·의미를 **30**이 소유한다(여기서는 위치만 안내).

---

## 5. 관계 명세 (Relationships)

> DB의 외래키가 아니라 **코드상의 소유/자기참조/논리적 조인**이다. 카디널리티는 crow's-foot 표기.

| # | 관계 | 카디널리티 | 종류 | 코드 근거 |
|---|---|---|---|---|
| R1 | `App` → `ProcessInfo` (평탄) | 1 → N | 소유 (`raw_processes`/`flat_list`) | `App.raw_processes: Vec<ProcessInfo>` |
| R2 | `App` → `ProcessInfo` (트리) | 1 → N | 소유 (`process_trees` 루트들) | `App.process_trees: Vec<ProcessInfo>` |
| R3 | `ProcessInfo` → `ProcessInfo` | 1 → N | **자기참조**(부모→자식) | `ProcessInfo.children: Vec<ProcessInfo>` |
| R4 | `ProcessInfo` → `Runtime` | N → 1 (0..1) | 참조(`Option`) | `ProcessInfo.runtime: Option<Runtime>` |
| R5 | `ProcessInfo` → `FrameworkKind` | N → 1 | 참조 | `ProcessInfo.framework: FrameworkKind` |
| R6 | `App` → `Config` | 1 → 1 | 소유 | `App.config: Config` |
| R7 | `Config` → `General/Display/FilterConfig` | 1 → 1 (각) | 소유(3 섹션) | `Config.{general,display,filter}` |
| R8 | `App` → `LogStreamer` | 1 → 0..1 | 소유(`Option`) | `App.log_streamer: Option<LogStreamer>` |
| R9 | `NetworkConnection` ↔ `ProcessInfo` | N ↔ 1 | **논리적 조인**(같은 `pid`) | `NetworkConnection.pid` = `ProcessInfo.pid` (do_scan, 30) |
| R10 | `Rule` → `Runtime`/`FrameworkKind` | N → 1 (각) | 참조 | `Rule.{runtime,framework}` |
| R11 | `App` → `DialogKind`/`DetailTab`/`SortColumn`/`FocusPanel` | 1 → 1 (각, Dialog는 0..1) | 참조(상태값) | `App.{dialog,active_tab,sort_column,focus}` |

**핵심 관계 3가지 요약:**
- **R3 자기참조 트리**가 ntop 도메인의 중심 — 한 프로세스가 자식을 품어 부모-자식 트리를 이룬다(다중선택·
  트리 종료·확장/축소가 모두 이 관계에 기댐).
- **R4 `Option<Runtime>`** 가 "서버냐 트리 부모냐"를 가른다(`is_server()`/`is_node()`/Node-only 필터).
- **R9 논리적 조인**으로 네트워크 포트가 프로세스에 결합된다(소유 아님 — 별도 스캔 후 `pid`로 매칭).

---

## 자체 검증 (Self-verification)

| # | 체크 | 결과 | 어디서 충족 |
|---|---|---|---|
| **E-1** | DB 없음 명시 | **PASS** — §0 콜아웃 + 생애 도식("종료 시 소멸, 설정 파일만 디스크") | §0 |
| **E-2** | 핵심 구조체 전 필드(타입) | **PASS** — `ProcessInfo`(18필드)·`Config`+3하위·`NetworkConnection`·`LogStreamer`·`App`·`Rule` 모두 필드·타입·코드 근거 | §2 |
| **E-3** | enum 값 전부 | **PASS** — Runtime8·FrameworkKind15·HealthStatus3·KillSignal(3+3)·KillResult4·GracefulResult5·DetailTab4·DialogKind5·FocusPanel2·SortColumn9 | §3 |
| **E-4** | Config 기본값 단일 출처 | **PASS** — §4 정규표(refresh=3·SIGTERM·graceful=10·confirm=true·tree=true·auto·mask=true·include_*=false·MAX_BUFFER_LINES=1000) | §4 |
| **E-5** | 관계 명세 | **PASS** — §5 R1~R11 표(카디널리티·종류·코드 근거) + §1 ERD | §1, §5 |
| **E-6** | 시각화(ERD + enum 표, 외부 렌더러 0) | **PASS** — §1 crow's-foot ASCII ERD, §3 순수 표 | §1, §3 |
| **E-7** | 단일 출처 준수(동작=30·정책=33·키=21) | **PASS** — health 임계·분류 규칙·파싱·시그널 가용성·키 매핑을 30/33/21로 회부, 값만 정의 | 전체 |

**추가 원칙 점검:**
- **창작 금지(spec §7):** DB 테이블·SQL·REST·결제·회원 0. 모든 필드·타입·기본값은 실제 소스(`mod.rs`·
  `config.rs`·`network.rs`·`streamer.rs`·`app.rs`·`killer.rs`·`framework_rules.rs`)에서 추출.
- **코드-진실 정직 표기:** `framework_version`은 항상 `None`(미채움), `Express`/`Fastify`/`Koa`/`Hapi`는
  enum만 있고 규칙 미등록 — 둘 다 실제 코드 상태로 명시(40-backlog 로드맵 연결).
- **중복정의 0:** 같은 수치를 두 곳에 값으로 적지 않음 — Config/상수 값은 §4에만, 알고리즘 임계(200ms·
  health 90/80)는 30, 시그널 가용성/마스킹 규칙은 33으로 회부.
- **쉬운 설명:** ERD·인메모리·구조체·enum·1:N·자기참조·Option·PID 첫 등장 풀이.

**알려진 한계(후속 보완):**
- enum 값/Config 기본값이 코드에서 바뀌면 본 문서가 따라가야 한다(정규 출처이므로 코드 변경 시 1순위 갱신
  대상). 동작/임계값은 본 문서가 아니라 30/33을 봐야 한다.
