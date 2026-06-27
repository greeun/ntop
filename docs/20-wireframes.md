# 20 · 와이어프레임 — 저충실도 TUI 화면 시안 (ntop)

> ════════════════════════════════════════════════════════════════════════════
> # 🟡 HUMAN_CHECKPOINT_REQUIRED  (STEP 7 / 게이트 P-1)
> ════════════════════════════════════════════════════════════════════════════
>
> **이 문서는 사람 검수 대상이다.** 와이어프레임의 시각 품질(폭·정렬·문자열 충실도)은 LLM이
> 자체 채점할 수 없는 영역이므로, **메인테이너/디자이너가 아래 「검수 포인트」를 직접 확인**한 뒤
> 다음 단계(g4~)로 진행한다. 채점에서는 제외되지만 **사실 정확성(코드-진실)은 채점된다.**

> **ntop은 터미널 도구다.** 여기서 "화면(screen)"은 웹/GUI 페이지가 아니라 **TUI 뷰·패널·탭·모달
> 다이얼로그**다. 따라서 **ASCII 터미널 프레임이 곧 실제 화면 시안**이다(spec §4-0). 아래 모든 시안은
> 실제 소스코드(`src/tui/widgets/*.rs`, `src/tui/ui.rs`)에서 렌더되는 **실문자열**을 그대로 옮겼고,
> 한국어는 `←` 주석으로만 덧붙였다(창작 0).
>
> **참조(재정의 0):** 화면 ID·노출 경로는 **`12-ia`**, 기능 ID(F1~F21)는 **`10-prd`**, 유저 플로우는
> **`13-user-flows`**, **키 바인딩의 정규 출처는 `21-screen-spec`**, 화면 명세·4상태는 **`21-screen-spec`**,
> 구조체/필드/Config 기본값은 **`31-erd`**, CLI 출력 스키마는 **`32-api-spec`**가 단일 출처다. 본
> 문서는 **저충실도 시안(레이아웃 그림)** 만 소유한다(spec §4-3).

---

## Sprint Contract (self-proposed checks)

이 문서(g3의 `20-wireframes`)가 만족시킬 **관찰 가능한 체크**(playbook §S4-g3 관찰 바 "20=화면당 1
저충실도 시안(ASCII/인라인-SVG 터미널 프레임)" + spec §5(20행)·§8에서 도출).

| # | 체크 | 어떻게 충족하나 | 본문 위치 |
|---|---|---|---|
| **W-1** | **화면당 1 ASCII 터미널 프레임** — 12 TUI 화면 전부(메인·상세 4탭·다이얼로그 5·필터·빈상태) | §3 W-1~W-12 시안(+CLI 참고) | §3 |
| **W-2** | **세로(상/하) 레이아웃** — 목록 위 55% · 상세 아래 45% (코드-진실, ui.rs L37–44) | §0 정정 박스 + W-1 V-MAIN | §0·§3 |
| **W-3** | **목록 컬럼 코드-진실** — `[✓/●] PID·NAME·PORT·THR·CPU·MEM·USER·STS·UPTIME` (별도 FRAMEWORK 컬럼 없음) | §0 + W-1 컬럼 헤더 | §0·§3 |
| **W-4** | **실문자열 충실도** — 상단/하단바·다이얼로그·탭 콘텐츠를 코드 렌더 문자열 그대로 | §3 전 시안(영문 실문자열 + `←` 한글 주석) | §3 |
| **W-5** | **HUMAN_CHECKPOINT 배너 + 검수 포인트** | 문서 상단 배너 + §2 검수 포인트 | 상단·§2 |
| **W-6** | **시각화(G-g)** — 전 화면 ASCII 프레임(산문-only 화면 0, 외부 렌더러 0) | §3 전부 ASCII | 전체 |

> 게이트 공급: **G-g**(화면당 시각 동반), **P-1**(와이어프레임 휴먼 체크포인트).

---

## 0. 사실 정정 (코드-진실) — 시안에 반드시 반영됨

> 일부 선행 표현(도시에 §6 일부·README 예시)이 "좌/우 분할" 또는 "목록 FRAMEWORK 컬럼"으로 적었으나
> **실제 코드와 불일치**다. 본 시안은 아래 **코드-진실**을 정규로 쓴다(`src/tui/ui.rs` L37–44,
> `src/tui/widgets/process_list.rs`).

```
 ① 레이아웃 = 세로(상/하) 분할                ② 목록 NAME 컬럼의 프레임워크 표기 방식
 ┌──────────────────────────────────┐         목록 컬럼(process_list.rs L23–34 헤더):
 │ BAR-TOP   상단 상태바 (1줄)        │ [크롬]    [ ]● · PID · NAME · PORT · THR · CPU · MEM · USER · STS · UPTIME
 ├──────────────────────────────────┤          └─ ★ 별도 "FRAMEWORK" 컬럼은 없다.
 │ V-LIST    프로세스 목록 (위 55%)   │ [패널1]      대신 프레임워크가 탐지되면 NAME 안에 인라인으로 붙는다:
 ├──────────────────────────────────┤              · framework≠Generic → "next-server (Next.js)"  (L121)
 │ V-DETAIL  상세 패널 (아래 45%)     │ [패널2]      · Node 맨몸          → 배지 없음 "vite"
 │   └ 탭: Info · Log · Net · Env     │ [탭]         · 그 외 런타임 맨몸  → "uvicorn [Python]"  (L127)
 ├──────────────────────────────────┤              · runtime=None(트리부모)→ 배지 없음 "claude"(흐림)
 │ BAR-BOTTOM 하단 키힌트 (1줄)       │ [크롬]    전체 런타임/프레임워크/버전 상세는 → TAB-INFO·CLI `list` 테이블
 └──────────────────────────────────┘          (ui.rs L37–44 Layout::vertical([55%,45%]) — 좌/우 아님)
```

> **체크박스·헬스 표기(코드-진실):** 첫 칸은 `[ ]`(미선택)/`[x]`(Space 다중선택) + 헬스 점 `●`
> (Green=정상·Yellow=주의·Red=위험). 연구 시안의 `□`는 부정확 → 본 시안은 `[ ]`/`[x]`로 정정.
> **트리 프리펫스(코드-진실):** depth0 `▾ `(펼침)/`▸ `(접힘)/`  `(자식없음), depth N
> `{들여쓰기}├▾ `/`├▸ `/`└─ ` (세로 연결선 `│`은 코드가 그리지 않음).

---

## 1. 색·역할 범례 (코드 근거 · md는 역할 태그로 표기)

> 터미널이라 "색"이 영역 구분·위험 신호의 핵심 언어다. 색은 `status_bar.rs`·`empty_state.rs`·
> `process_list.rs` 등의 실제 `Color::*`에서 추출(12-ia §2 승계). md는 역할 태그, HTML(`20-…html`)에서
> 인라인 SVG로 실색 렌더.

```
 역할 태그   영역                                   │ 색(코드 근거)
 ───────────────────────────────────────────────────┼───────────────────────────────────
 [크롬]      상/하 바(BAR-TOP/BOTTOM)                │ 배경 DarkGray · 글자 White
 [패널]      포커스 영역(V-LIST/V-DETAIL)            │ 활성 테두리 Cyan · 비활성 Gray
 [탭]        상세 하위(Info/Log/Net/Env)             │ 활성 탭 Cyan 볼드+밑줄, 비활성 Gray
 [모드]      필터 입력                               │ Yellow(`/`·커서 █)
 [모달·위험] kill 계열 다이얼로그                    │ 테두리 Red
 [모달]      SignalPicker·Help                       │ 테두리 Cyan
 [빈/로딩]   프로세스 0건 + 스피너                   │ 스피너 Cyan · 안내문 DarkGray
 ───────────────────────────────────────────────────┴───────────────────────────────────
 색 의미: Cyan=브랜드/포커스/스피너 · Green=정상(CPU<50·헬스 Healthy·LISTEN) ·
          Yellow=주의(CPU 50~80·키힌트·헬스 Warning) · Red=위험(CPU>80·헬스 Critical·파괴적 kill) ·
          LightCyan=분류된 서버 행 · DarkGray=트리부모 행(흐림)/크롬 배경/비활성
```

---

## 2. ✅ 검수 포인트 (STEP 7에서 사람이 확인)

> 아래 6가지가 **실제 ntop 화면과 일치하는지** 메인테이너/디자이너가 직접 확인한다.

1. **레이아웃 방향** — 목록이 **위(55%)**, 상세가 **아래(45%)** 인 **세로 분할**이 맞는가?
   (좌/우 분할이 아님 — ui.rs L37–44)
2. **목록 컬럼 순서/폭** — `[ ]● · PID · NAME · PORT · THR · CPU · MEM · USER · STS · UPTIME` 순서와
   대략적 폭(넓은 터미널 기준 PID 7 / NAME 가변 / PORT 7 / THR 4 / CPU 7 / MEM 9 / USER 8 / STS 7 /
   UPTIME 10)이 맞는가? **별도 FRAMEWORK 컬럼이 없고**, 프레임워크는 NAME에 `(Next.js)`로 인라인 표기됨.
3. **상단/하단바 문구** — `ntop v0.2.0 | CPU: x% MEM: u/tMB | Servers: n | Refresh: Ns | [H]elp`,
   하단바가 **포커스/모드별로 다른 키힌트**(리스트/상세/필터/모달)를 보여주는가?
4. **다이얼로그 실문자열** — kill/signal/force/help 모달의 **영문 문구**(`Are you sure you want to kill
   this process?`, `Send SIGKILL (force kill)?`, `Select a signal to send:` 등)가 코드와 일치하는가?
5. **탭 콘텐츠 표현** — Info(세로 key:value 18줄)·Log(`Source:` 헤더+tail)·Net(LOCAL/REMOTE/STATE
   표)·Env(`KEY=값`, 민감값 `********` 마스킹)의 구성이 맞는가?
6. **스피너·체크박스·트리·정렬자 기호** — 스피너(`⠋⠙⠹…` Cyan), 체크박스 `[ ]`/`[x]`, 트리 `▾`/`▸`/`├`/`└`,
   정렬 인디케이터 ` ^`/` v` 표현이 맞는가?

> 폭은 가독성을 위한 **약 84칸 예시**이며, 실제는 터미널 폭에 맞춰 ratatui가 신축/절단한다(넓이>100이면
> 넓은 컬럼, ≤100이면 동일 컬럼 축소). 시각 품질(정렬 미세 어긋남)은 채점 제외, **문자열·구성·순서의
> 사실 정확성만** 본다.

---

## 3. 화면 시안 (화면당 1개 · 12 TUI 화면)

> 화면 ID는 `12-ia §6` 승계. 각 시안 머리에 `[화면 ID · 위젯 파일 · 진입]`을 부기한다. 명세(목적·키·
> 상태·예외·연결)는 `21-screen-spec`이 소유 — 여기서는 **레이아웃 그림만**.

### W-1 · V-MAIN (메인 뷰: 상단바 + 목록 위 + 상세 아래 + 하단바)
`[V-MAIN · ui.rs::render · 진입: `ntop`(서브커맨드 없음)]`

```
 ntop v0.2.0  |  CPU: 23.4%  MEM: 8124/16384MB  |  Servers: 5  |  Refresh: 3s  |  [H]elp      ← BAR-TOP [크롬]
┌ Processes ─────────────────────────────────────────────────────────────────────────────┐
│       PID    NAME                      PORT   THR  CPU↓    MEM       USER     STS   UPTIME │ ← 헤더(활성 정렬 CPU에 ↓)
│ [ ] ● ▾ 932  claude                    -      18   1.2%   210.0 MB  seoyeon  Run   3h 5m  │ ← 트리부모(runtime None·흐림)
│ [ ] ●   ├▾ 41822 next-server (Next.js) 3000   11   2.1%   128.0 MB  seoyeon  Run   1h 2m ◀│═ 선택행(DarkGray 강조·볼드)
│ [ ] ●   │  └─ 41850 worker             -       3   0.4%   64.0 MB   seoyeon  Run   1h 1m  │   (자식: 인라인 배지 없음=Node맨몸)
│ [x] ●   └─ 41835 vite                  -       7   0.4%   96.0 MB   seoyeon  Run   1h 2m  │ ← [x]=Space 다중선택(Cyan)
│ [ ] ● ▸ 42013 uvicorn [Python]         8000    5   5.0%   180.0 MB  seoyeon  Run   58m    │ ← 비-Node 맨몸 → [Python] 배지
│ [ ] ●   42090 celery [Python]          -       3   0.0%   140.0 MB  seoyeon  Run   58m    │
└──────────────────────────────────────────────────────────────────────────────────────────┘
┌ Details ───────────────────────────────────────────────────────────────────────────────┐
│  Info │ Log │ Net │ Env                                                                   │ ← 탭바(활성 Info=Cyan 볼드밑줄)
│  ‾‾‾‾                                                                                      │
│           PID: 41822                                                                       │ ← key 우정렬 12칸 (info_tab.rs)
│          Name: next-server                                                                 │
│       Runtime: Node          Framework: Next.js          Ports: 3000                       │ ← (실제는 한 줄당 1쌍, 여기선 압축 표기)
│           CPU: 2.1%             Memory: 128.0 MB         Uptime: 1h 2m 5s                   │
└──────────────────────────────────────────────────────────────────────────────────────────┘
 [q] Quit | [Up/Down] Navigate | [PgUp/Dn] Page | [Enter] Expand | [Tab] Details | [Space] Select | [/] Filter | [s] Sort | [x] Kill | [S] Signal   ← BAR-BOTTOM(리스트 포커스 힌트)
```

> 주: V-DETAIL의 Info 탭은 실제로 **한 줄당 `key: value` 한 쌍**의 세로 목록(18줄)이다(W-2). 위 메인
> 시안은 공간상 2~3쌍을 한 줄로 압축해 보였을 뿐 — 정확한 세로 배열은 W-2를 본다.

---

### W-2 · TAB-INFO (상세 Info 탭 — 프레임워크 정체 확인 자리)
`[TAB-INFO · info_tab.rs · 진입: V-DETAIL 기본 탭]`

```
┌ Details ───────────────────────────────────────────────────────────────────────────────┐
│  Info │ Log │ Net │ Env                                                                   │
│  ‾‾‾‾                                                                                      │
│           CWD: /Users/seoyeon/proj/web/apps/site                                           │ ← 18개 key:value 세로 목록
│       Command: node node_modules/.bin/next dev                                             │   (key 우정렬 12칸·Cyan 볼드)
│           PID: 41822                                                                       │
│          PPID: 932                                                                         │
│          Name: next-server                                                                 │
│       Runtime: Node                                                                        │ ← ★ 프레임워크 정체는 여기서 확인
│     Framework: Next.js                                                                     │
│       Version: -                                                                           │ ← framework_version 항상 None → "-"
│         Ports: 3000                                                                        │
│           CPU: 2.1%                                                                        │
│        Memory: 128.0 MB                                                                    │ ← macOS phys_footprint(Activity Monitor 일치)
│  Memory (VMS): 1.2 GB                                                                      │
│       Threads: 11                                                                          │
│        Uptime: 1h 2m 5s                                                                    │
│          User: seoyeon                                                                     │
│        Status: Running                                                                     │
│        Health: Healthy                                                                     │ ← health() = Healthy/Warning/Critical
│      Open FDs: 64                                                                          │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

---

### W-3 · TAB-LOG (상세 Log 탭 — 로그 자동 감지·tail)
`[TAB-LOG · log_tab.rs · 진입: 탭 순환(Tab/←→)]`

```
┌ Details ───────────────────────────────────────────────────────────────────────────────┐
│  Info │ Log │ Net │ Env                                                                   │ ← 활성 Log=Cyan 볼드밑줄
│        ‾‾‾                                                                                  │
│  Source: /Users/seoyeon/proj/web/.next/server/app/page.log                                │ ← 가장 최근 수정 로그파일 tail
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ │ ← "  ─" 반복 구분선
│  ✓ Compiled /app in 412ms                                                                 │ ← 일반 줄: White
│  GET / 200 in 34ms                                                                        │
│  WARN  slow query 812ms                                                                   │ ← "WARN"/"warn" 포함 → Yellow
│  ERROR  ECONNREFUSED 127.0.0.1:5432                                                        │ ← "ERROR"/"error" 포함 → Red
│  GET /api/health 200 in 3ms                                                               │   (DEBUG/debug → DarkGray)
│  ▌(새 줄 실시간 추가… 버퍼 최대 1000줄, Up/Down 스크롤)                                    │ ← MAX_BUFFER_LINES=1000(31)
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

---

### W-4 · TAB-NET (상세 Net 탭 — LISTEN/연결)
`[TAB-NET · net_tab.rs · 진입: 탭 순환]`

```
┌ Details ───────────────────────────────────────────────────────────────────────────────┐
│  Info │ Log │ Net │ Env                                                                   │ ← 활성 Net
│              ‾‾‾                                                                            │
│  LOCAL                          REMOTE                       STATE                         │ ← 표 헤더(40%/40%/20%·Yellow)
│  0.0.0.0:3000                   -                            LISTEN                        │ ← LISTEN=Green
│  127.0.0.1:3000                 127.0.0.1:51544              ESTABLISHED                   │ ← ESTABLISHED=Cyan
│  127.0.0.1:3000                 127.0.0.1:51547              ESTABLISHED                   │   (CLOSE_WAIT=Yellow·TIME_WAIT=DarkGray)
│                                                                                            │
│  ※ lsof(Unix)/netstat(Win) 부재 시 → "No active network connections found." (빈 결과)     │ ← 에러는 빈 결과로 수렴(§W 주)
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

---

### W-5 · TAB-ENV (상세 Env 탭 — 민감값 마스킹)
`[TAB-ENV · env_tab.rs · 진입: 탭 순환]`

```
┌ Details ───────────────────────────────────────────────────────────────────────────────┐
│  Info │ Log │ Net │ Env                                                                   │ ← 활성 Env
│                    ‾‾‾                                                                      │
│  NODE_ENV=development                                                                      │ ← "KEY=값" (KEY=Cyan 볼드·= DarkGray)
│  PORT=3000                                                                                 │
│  DATABASE_URL=********                                                                     │ ← 민감 패턴 → "********"(8자) 마스킹
│  API_SECRET=********                                                                       │   (SECRET 포함)
│  AWS_TOKEN=********                                                                        │   (TOKEN 포함)
│  PATH=/usr/local/bin:/usr/bin:/bin                                                         │ ← 비민감 → 원값
└──────────────────────────────────────────────────────────────────────────────────────────┘
   민감 패턴(env_tab.rs): PASSWORD · SECRET · TOKEN · KEY · API_KEY · PRIVATE · CREDENTIALS · AUTH
   (mask_env_values 기본 true · 31 정규값. false면 원값 노출)
```

---

### W-6 · DLG-KILL (단건 종료 확인 모달)
`[DLG-KILL(KillConfirm) · kill_dialog.rs · 진입: `x` · 테두리 Red · 화면 50%×40% 중앙]`

```
                ┌──────────────── Kill Process ─────────────────┐
                │                                                │
                │  Are you sure you want to kill this process?   │ ← Yellow
                │                                                │
                │    PID:  41822                                 │ ← key=Cyan / 값=White
                │    Name: next-server                           │
                │    Port: 3000                                  │ ← 포트 있을 때만 노출
                │                                                │
                │  Signal: SIGTERM (graceful)                    │ ← DarkGray
                │                                                │
                │  [Enter] Confirm  [Esc] Cancel                 │ ← Enter=Green / Esc=Red
                └────────────────────────────────────────────────┘
```

---

### W-7 · DLG-KILLTREE (트리 종료 확인 모달)
`[DLG-KILLTREE(KillTreeConfirm) · kill_dialog.rs · 진입: `K` · 테두리 Red]`

```
                ┌─────────────── Kill Process Tree ─────────────┐
                │                                                │
                │  Kill process tree? This will terminate:       │ ← Yellow
                │                                                │
                │    Root: PID 41822 (next-server)               │ ← key=Cyan / 값=White
                │    Children: 2 process(es)                     │ ← 자손 총수=Red (count_descendants)
                │      - PID 41850 (worker)                      │ ← 직계 자식만 나열(DarkGray)
                │      - PID 41835 (vite)                        │
                │                                                │
                │  [Enter] Confirm  [Esc] Cancel                 │
                └────────────────────────────────────────────────┘
   ※ kill_tree는 자식부터 역순(SIGTERM) → 고아 프로세스 0 (처리 정규 출처 30·33)
```

---

### W-8 · DLG-SIGNAL (임의 시그널 선택 모달)
`[DLG-SIGNAL(SignalPicker) · signal_picker.rs · 진입: `S` · 테두리 Cyan · 45%×50%]`

```
              ┌──────────────── Signal Picker ─────────────────┐
              │                                                 │
              │  Select a signal to send:                       │ ← Yellow
              │                                                 │
              │  ▸ SIGTERM    - Graceful termination request    │ ← ▸=현재 선택(Cyan 볼드)
              │    SIGKILL    - Force kill (cannot be caught)    │   (이름 좌정렬 10칸 {:<10})
              │    SIGHUP     - Hangup / reload configuration    │
              │    SIGINT     - Interrupt (like Ctrl+C)          │
              │    SIGUSR1    - User-defined (Node.js: activate debugger) │
              │    SIGUSR2    - User-defined signal              │
              │                                                 │
              │  [Enter] Send  [Up/Down] Select  [Esc] Cancel   │
              └─────────────────────────────────────────────────┘
   ※ Unix=6종(위 전부) · Windows=3종(SIGTERM/SIGKILL/SIGINT만 — KillSignal::all, 31/33)
```

---

### W-9 · DLG-FORCE (강제 종료 확인 모달 · 현재 미도달 죽은 코드)
`[DLG-FORCE(ForceKillPrompt) · kill_dialog.rs · 진입: (현재 코드에서 미도달 — 어떤 키/경로로도 트리거되지 않음) · 테두리 Red]`

```
                ┌──────────────── Force Kill ───────────────────┐
                │                                                │
                │  Graceful kill timed out.                      │ ← Red
                │                                                │
                │  Send SIGKILL (force kill)?                    │ ← Yellow
                │                                                │
                │    PID: 41822                                  │
                │                                                │
                │  WARNING: SIGKILL cannot be caught or ignored. │ ← Red 볼드
                │                                                │
                │  [Enter] Force Kill  [Esc] Cancel              │
                └────────────────────────────────────────────────┘
   ※ 위 시안은 핸들러(ui.rs L183)·위젯(kill_dialog.rs)이 그릴 수 있는 화면이나, **현재 이 모달을 띄우는
     코드 경로가 없다**(app.dialog=Some(...)는 KillConfirm/KillTreeConfirm/Help/SignalPicker 4종만 설정).
     즉 **자동 graceful→force 승격은 TUI가 아니라 CLI(`ntop kill`) 단건 전용**이며, TUI 강제 모달은 미도달
     죽은 코드다 — 연결 또는 제거는 로드맵 `40` M1. (정규 출처: 30 §2-6·§8-3 / 33 §2-1)
```

---

### W-10 · DLG-HELP (키 바인딩 도움말 모달 · 스크롤)
`[DLG-HELP(Help) · help_dialog.rs · 진입: `H` · 테두리 Cyan · 60%×70% · 우측 스크롤바 ↑/↓]`

```
       ┌──────────────────────────── Help  [1/2] ────────────────────────────┐ ← 스크롤 시 [현재/총] 표기
       │  ntop v0.2.0                                                       ↑ │
       │                                                                      │
       │  -- Process List (default focus) --                                  │ ← 섹션 헤더 Yellow 볼드
       │    Up/Down       Move cursor                                         │ ← key=Green(좌정렬 14칸) / desc=White
       │    PgUp/PgDn     Page up/down                                        │
       │    Home/End      Jump to first/last                                  │
       │    Enter         Expand / collapse tree                              │
       │    Left/Right    Collapse / expand node                              │
       │    Space         Toggle multi-select                                 │
       │    Tab           Move focus to detail panel                          │
       │                                                                      │
       │  -- Detail Panel (Tab to enter) --                                   │
       │    Tab           Next detail tab                                     │
       │    Shift+Tab     Previous detail tab                                 │
       │    Up/Down       Scroll content                                      │
       │    PgUp/PgDn     Page content                                        │
       │    Esc           Return to process list                             ↓ │
       │  …(아래로 스크롤: Filtering & Sort / Process Control / General / This Dialog)…  │
       └──────────────────────────────────────────────────────────────────────┘
   ※ 전체 키 카탈로그(정규 출처)는 21-screen-spec §2. 본 Help 모달이 그 카탈로그의 인앱 표현이다.
```

---

### W-11 · V-FILTER (필터 입력 모드)
`[V-FILTER · ui.rs::handle_filter_key + status_bar.rs · 진입: `/`]`

```
 ntop v0.2.0  |  CPU: 23.4%  MEM: 8124/16384MB  |  Servers: 2  |  Refresh: 3s  |  [H]elp
┌ Processes (filter: next) ──────────────────────────────────────────────────────────────┐ ← 블록 제목에 (filter: 텍스트)
│       PID    NAME                      PORT   THR  CPU     MEM       USER     STS   UPTIME │
│ [ ] ●   41822 next-server (Next.js)    3000   11   2.1%   128.0 MB  seoyeon  Run   1h 2m  │ ← "next" 부분일치만 실시간 좁힘
│ [ ] ●   41835 vite                     -       7   0.4%   96.0 MB   seoyeon  Run   1h 2m  │   (매칭: name·command·pid·framework·runtime·ports)
└──────────────────────────────────────────────────────────────────────────────────────────┘
┌ Details ───────────────────────────────────────────────────────────────────────────────┐
│  Info │ Log │ Net │ Env                                                                   │
│           PID: 41822    Name: next-server    Framework: Next.js    Ports: 3000             │
└──────────────────────────────────────────────────────────────────────────────────────────┘
 /next█  | [Enter] Apply | [Esc] Cancel                                                       ← BAR-BOTTOM(필터 모드 · 커서 █ Yellow)
```

> 입력 한 글자마다 `rebuild_view()`로 **실시간 갱신**(다음 스캔 tick 대기 0). `Enter`=필터 유지하고
> normal 복귀 · `Esc`=필터 비우고 복귀.

---

### W-12 · V-EMPTY (빈 상태 = 로딩 상태 · 스피너)
`[V-EMPTY · empty_state.rs · 진입: `flat_list`가 비었을 때 자동(메인 전체 대체)]`

```
 ntop v0.2.0  |  CPU: 5.1%  MEM: 6100/16384MB  |  Servers: 0  |  Refresh: 3s  |  [H]elp
┌───────────────────────────────────────── ntop ──────────────────────────────────────────┐ ← 제목 중앙 " ntop "
│                                                                                            │
│                                                                                            │
│                       ⠹  Scanning for server processes...                                  │ ← 스피너(Cyan 볼드·tick_count 회전)
│                                                                                            │   + 안내문(Gray)
│                       No server processes found. Waiting...                                 │ ← (DarkGray)
│                                                                                            │
│                Start a server and it will appear here automatically.                        │ ← (DarkGray)
│                                                                                            │
└──────────────────────────────────────────────────────────────────────────────────────────┘
 [q] Quit | [Up/Down] Navigate | [PgUp/Dn] Page | [Enter] Expand | [Tab] Details | [Space] Select | [/] Filter | [s] Sort | [x] Kill | [S] Signal
```

> **중요(코드-진실):** ntop은 **empty와 loading을 하나의 위젯(`empty_state.rs`)으로 합쳤다.** 스피너
> 프레임 `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`(Braille, tick_count로 회전)이 "살아서 스캔 중(loading)"을, "No server
> processes found. Waiting…"가 "결과 없음(empty)"을 **같은 자리에서** 함께 표현한다.

---

### (참고) CLI 화면 — 출력 스키마 정규 출처는 `32-api-spec`

> CLI 5종(`list`/`kill`/`info`/`log`/`config`)도 IA상 "화면"이지만, **출력 스키마(table 컬럼·JSON
> 필드·CSV 9컬럼)의 정규 출처는 `32-api-spec`**이다. 아래는 시각 참고용 1컷(레이아웃 감만).

```
$ ntop list                                            ← CLI-LIST · 기본 table (스키마 정규 출처=32)
  PID    NAME          FRAMEWORK   PORT    CPU     MEM        UPTIME
  41822  next-server   Next.js     3000    2.1%    128.0 MB   1h 2m 5s
  42013  uvicorn       Generic     8000    5.0%    180.0 MB   58m
  ↳ ★ CLI table에는 FRAMEWORK 컬럼이 있다(TUI 목록과 다른 점). --json/--format csv는 32 참조.
```

---

## 자체 검증 (Self-verification)

| # | 체크 | 결과 | 어디서 충족 |
|---|---|---|---|
| **W-1** | 화면당 1 ASCII 시안(12 TUI 화면) | **PASS** — W-1~W-12(메인·Info/Log/Net/Env·KILL/KILLTREE/SIGNAL/FORCE/HELP·필터·빈상태) 전부 | §3 |
| **W-2** | 세로(상/하) 레이아웃 | **PASS** — §0 정정 박스 + W-1 V-MAIN(목록 위 55%·상세 아래 45%) | §0·§3 |
| **W-3** | 목록 컬럼 코드-진실(FRAMEWORK 컬럼 없음·인라인 배지) | **PASS** — §0 + W-1/W-11 컬럼 헤더, NAME 인라인 `(Next.js)`/`[Python]` | §0·§3 |
| **W-4** | 실문자열 충실도(코드 렌더 그대로) | **PASS** — 상/하바·다이얼로그(영문 실문자열)·탭 콘텐츠를 `*.rs`에서 추출, 한글은 `←` 주석만 | §3 |
| **W-5** | HUMAN_CHECKPOINT 배너 + 검수 포인트 | **PASS** — 상단 배너 + §2 검수 포인트 6항 | 상단·§2 |
| **W-6** | 시각화(G-g, 산문-only 0, 외부 렌더러 0) | **PASS** — §1~§3 전부 순수 ASCII 프레임, Mermaid 미사용 | 전체 |

**추가 원칙 점검:**
- **창작 금지(spec §7):** 없는 화면(설정 마법사·로그인·대시보드) 시안 0. 결제/DB/REST 0. 실제 위젯
  렌더 화면만(`process_list.rs`·`detail_panel.rs`·`info/log/net/env_tab.rs`·`kill_dialog.rs`·
  `signal_picker.rs`·`help_dialog.rs`·`status_bar.rs`·`empty_state.rs`).
- **코드-진실 정정 반영:** 세로 레이아웃 / 별도 FRAMEWORK 컬럼 없음(NAME 인라인 배지) / 체크박스
  `[ ]`·`[x]`(연구 시안의 `□` 정정) / Info 탭 세로 18줄 key:value(연구 시안의 2컬럼 정정) / 다이얼로그
  영문 실문자열(연구 시안의 한글 의역 정정) / 스피너 프레임·empty=loading 합침 — 모두 소스 확인.
- **단일 출처 준수:** 키 바인딩·명세·4상태 = `21-screen-spec` / CLI 출력 스키마 = `32-api-spec` /
  Config 값(1000줄·10s·3s) = `31-erd` / 화면 ID = `12-ia` 로 회부. 본 문서는 레이아웃 그림만 소유.

**알려진 한계(P-1 검수 대상):**
- **폭/정렬은 약 84칸 예시** — 실제는 터미널 폭에 신축(넓이>100/≤100 분기). 미세 정렬 어긋남은 시각
  품질 영역으로 STEP 7 검수 대상(채점 제외).
- **메인 W-1의 Info 탭 압축 표기** — 실제는 한 줄당 `key: value` 한 쌍의 세로 배열(정확본 W-2). 공간상
  메인 시안에서만 2~3쌍을 한 줄로 압축해 표현했음을 명시.
