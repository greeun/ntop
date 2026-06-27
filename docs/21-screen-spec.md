# 21 · 화면 명세 / 스토리보드 — 키 바인딩 정규 출처 (ntop)

> **이 문서는 두 가지의 단일 출처(single source)다.**
> ① **키 바인딩(전역/리스트/상세/필터/다이얼로그)의 정규 출처** — `12-ia`·`13-user-flows`·
> `30-functional-spec`·`41-qa-testcases`는 §2 키 카탈로그를 **참조**만 하고 재정의하지 않는다.
> ② **화면별 명세(목적/구성요소/상호작용/상태/예외/연결) + 핵심 화면 4상태(empty/loading/normal/error)**.
>
> **참조(재정의 0):** 화면 ID·노출 경로·IA는 **`12-ia`**, 기능 ID(F1~F21)·우선순위는 **`10-prd`**, 유저
> 플로우(UF-1~6)·kill 상태기계/시퀀스는 **`13-user-flows`**, 문제(PRB-1~8)는 **`research-s2`(→03 §2)**,
> 저충실도 시안은 **`20-wireframes`**, 구조체/enum/Config 기본값은 **`31-erd`**, 분류·스캔·graceful 처리
> 알고리즘은 **`30-functional-spec`**, CLI/lib/시스템명령 계약·출력 스키마는 **`32-api-spec`**, 안전·
> 프라이버시 정책(마스킹·권한 매트릭스)은 **`33-policy`** 가 단일 출처다(spec §4-3).
>
> ════════════════════════════════════════════════════════════════════════════
> **🟡 HUMAN_CHECKPOINT (P-1):** 본 문서의 화면 시안/컴포넌트 배치는 `20-wireframes`와 함께 **STEP 7
> 사람 검수 대상**이다(시각 품질은 채점 제외, **사실 정확성은 채점**).
> ════════════════════════════════════════════════════════════════════════════
>
> **전제:** ntop은 **이미 v0.2.0까지 구현된 MIT 오픈소스 도구**다(codebase-facts §0). 본 명세는 신규
> 설계가 아니라 **이미 구현된 화면을 역설계**한 것이다(창작 0). 모든 화면/키/상태는 실제 소스코드
> (`src/tui/ui.rs`·`event.rs`·`widgets/*.rs`)와 도시에 §6에 역추적된다.
>
> **용어 풀이(첫 등장):** **TUI** = 터미널 안에서 키보드로 조작하는 전체화면 텍스트 UI / **패널** =
> 화면을 나눈 영역 / **탭** = 한 패널 안에서 전환되는 하위 화면 / **모달 다이얼로그** = 떠서 다른 입력을
> 막고 결정을 받는 작은 창 / **포커스** = 지금 키 입력을 받는 패널 / **4상태** = empty(빈)/loading(불러오는
> 중)/normal(정상)/error(오류)로 화면이 가질 수 있는 네 가지 상태 / **graceful 종료** = "정리하고 꺼져라"
> 요청(SIGTERM) / **에스컬레이션** = 안 통하면 강제로 단계 올림 / **EPERM** = 권한 부족 오류(Error:
> Operation not PERMitted).

---

## Sprint Contract (self-proposed checks)

이 문서(g3의 `21-screen-spec`)가 만족시킬 **관찰 가능한 체크**(playbook §S4-g3 관찰 바 "21=화면당 시안+
컴포넌트 배치 + 핵심 화면 4상태 + 화면 명세 + 키 바인딩 정규 출처" + spec §5(21행)·§8에서 도출).

| # | 체크 | 어떻게 충족하나 | 본문 위치 |
|---|---|---|---|
| **P-1** | **키 바인딩 카탈로그 = 정규 출처** — 전역/리스트/상세/필터/다이얼로그 5그룹 + 컨텍스트 하단바 | §2 키 카탈로그(코드 근거) | §2 |
| **P-2** | **화면당 컴포넌트 배치 시안** — 16 TUI 화면(산문-only 0, G-g) | §3 각 화면 컴포넌트 다이어그램 | §3 |
| **P-3** | **화면별 명세 9요소** — 목적/구성요소/상호작용(키)/상태/예외/연결 기능(10)/화면(12)/데이터(31)/인터페이스(→32) | §3 각 화면 명세 블록 | §3 |
| **P-4** | **핵심 화면 4상태 세트(개별 미니 시안)** — V-LIST/TAB-LOG/TAB-NET 각각 empty/loading/normal/error | §3 해당 화면 4상태 블록 + §4 매트릭스 | §3·§4 |
| **P-5** | **에러 트리거 명시** — EPERM(권한거부)·lsof/netstat 부재·로그 접근 실패 | §3 예외 + §4 에러 트리거 표 | §3·§4 |
| **P-6** | **사실 정정 반영** — 세로 레이아웃·목록 FRAMEWORK 컬럼 없음(NAME 인라인)·체크박스 `[ ]`/`[x]` | §0 정정 박스 | §0 |
| **P-7** | **시각화(G-g, 외부 렌더러 0)** — 전 화면 ASCII 컴포넌트/4상태 다이어그램 | §2~§4 전부 ASCII | 전체 |
| **P-8** | **단일 출처 준수** — 기능=10·화면=12·플로우=13·데이터=31·알고리즘=30·계약=32·정책=33 참조, 키만 소유 | 본문 참조 표기 | 전체 |

> 게이트 공급: **G-g**(화면당 시각 동반), **P-1**(휴먼 체크포인트).

---

## 0. 사실 정정 (코드-진실) — 반드시 준수

```
 ① 레이아웃 = 세로(상/하)             ② 목록 FRAMEWORK 컬럼 없음        ③ 체크박스·헬스 표기
 ┌──────────────────────────┐         목록 컬럼(process_list.rs):       첫 칸 = "[ ]"/"[x]" + " " + "●"
 │ BAR-TOP  (1줄)            │ [크롬]   [ ]● PID·NAME·PORT·THR·CPU·       · [ ]=미선택 / [x]=Space 다중선택(Cyan)
 ├──────────────────────────┤            MEM·USER·STS·UPTIME            · ●=헬스(Green/Yellow/Red)
 │ V-LIST   목록 (위 55%)    │ [패널1]   └ FRAMEWORK 별도 컬럼 없음;        (연구 시안의 "□"는 부정확 → 정정)
 ├──────────────────────────┤             탐지 시 NAME에 인라인:
 │ V-DETAIL 상세 (아래 45%)  │ [패널2]     "next-server (Next.js)" /
 │   탭 Info·Log·Net·Env     │ [탭]        "uvicorn [Python]" /
 ├──────────────────────────┤             Node맨몸·트리부모=배지 없음
 │ BAR-BOTTOM (1줄)          │ [크롬]    전체 정체는 → TAB-INFO·CLI list  (ui.rs L37–44 vertical 55/45)
 └──────────────────────────┘
```

---

## 1. 화면 인벤토리 (참조 — 정규 출처 `12-ia §6`)

> 화면 ID 배정·노출 경로의 정규 출처는 `12-ia §6`. 본 문서는 그 화면들의 **명세·4상태·키**를 채운다.
> (TUI 고유 화면 16종 + CLI 5종. CLI 화면 명세는 `32-api-spec` 소유 — 본 문서는 TUI 16종만 명세.)

```
 크롬 2    : BAR-TOP · BAR-BOTTOM
 합성/패널 : V-MAIN · V-LIST(포커스1) · V-DETAIL(포커스2)
 탭 4      : TAB-INFO · TAB-LOG · TAB-NET · TAB-ENV
 모드 1    : V-FILTER
 빈/로딩 1 : V-EMPTY  (= empty 상태와 loading 상태를 합친 단일 위젯)
 모달 5    : DLG-KILL · DLG-KILLTREE · DLG-SIGNAL · DLG-FORCE · DLG-HELP
 ─ 핵심 화면(4상태 세트 제공): V-LIST · TAB-LOG · TAB-NET  (§3에서 개별 미니 시안)
```

---

## 2. ★ 키 바인딩 카탈로그 (정규 출처 / single source)

> **이 §2가 ntop 키 바인딩의 단일 출처다.** 값은 `src/tui/ui.rs`(키 핸들링)·`status_bar.rs`(하단바)·
> `help_dialog.rs`(인앱 도움말)에서 그대로 추출했다. `12`/`13`/`30`/`41`은 여기를 **참조**한다(spec §4-3).
> **Ctrl+C는 어디서든 종료**(ui.rs 최우선 처리). 키는 **컨텍스트 의존** — 같은 키도 포커스/모드/모달에
> 따라 의미가 갈린다.

### 2-1. 전역 (normal 모드 · 포커스 무관 공통)

| 키 | 동작 | 코드 근거 |
|---|---|---|
| `q` · `Ctrl+C` | 종료(`should_quit`) | ui.rs handle_normal_key · Ctrl+C 최우선 |
| `/` | 필터 모드 진입(`filter_text` 초기화) | ui.rs |
| `s` | 정렬 컬럼 순환(9종, `toggle_sort`) | ui.rs |
| `r` | 정렬 방향 반전(`sort_ascending`, 헤더 ` ^`/` v`) | ui.rs |
| `+` | refresh 간격 +1s **(최대 60s)** | 도시에 §6 |
| `-` | refresh 간격 −1s **(최소 1s)** | 도시에 §6 |
| `x` | DLG-KILL(KillConfirm) 열기 | ui.rs |
| `K` | DLG-KILLTREE(KillTreeConfirm) 열기 | ui.rs |
| `S` | DLG-SIGNAL(SignalPicker) 열기 | ui.rs |
| `H` | DLG-HELP(Help) 열기 | ui.rs |
| `e` | 전체 확장/축소 토글(`toggle_expand_all`) | ui.rs |
| `n` | Node-only 뷰 토글(`toggle_node_only`) | ui.rs |

### 2-2. 리스트 포커스 (V-LIST · `FocusPanel::ProcessList`)

| 키 | 동작 |
|---|---|
| `Up`/`k` · `Down`/`j` | 커서 위/아래 이동 |
| `PageUp` · `PageDown` | 10칸 위/아래 |
| `Home` · `End` | 처음 / 끝 |
| `Enter` | 트리 노드 확장/축소 토글 |
| `Right`/`l` | 노드 확장(`expand_selected`) |
| `Left`/`h` | 노드 축소(`collapse_selected`) |
| `Space` | 다중 선택 토글(`selected_pids`, 체크박스 `[x]`) |
| `Tab` | V-DETAIL로 포커스 이동 |
| `Esc` | 종료 |

### 2-3. 상세 포커스 (V-DETAIL · `FocusPanel::DetailPanel`)

| 키 | 동작 |
|---|---|
| `Tab` / `Right` / `l` | 다음 탭(Info→Log→Net→Env 순환) |
| `BackTab(Shift+Tab)` / `Left` / `h` | 이전 탭 |
| `Up`/`k` · `Down`/`j` | 콘텐츠 스크롤 ↑/↓(detail+log) |
| `PageUp` · `PageDown` | −10 / +10 스크롤 |
| `Home` | 맨 위 |
| `Esc` | V-LIST로 복귀 |
| `x` · `S` | (상세에서도) 종료 / 시그널 — 하단바 노출 |

### 2-4. 필터 모드 (V-FILTER · `filter_active`)

| 키 | 동작 |
|---|---|
| `Char(c)` | 입력(매칭: name·command·pid·framework·runtime·ports, 대소문자 무시 부분일치) |
| `Backspace` | 한 글자 삭제 |
| `Enter` | 필터 **유지**하고 모드 종료(Apply) |
| `Esc` | 필터 **비우고** 모드 종료(Cancel) |
| (입력마다) | `rebuild_view()` 실시간 갱신(다음 tick 대기 0) |

### 2-5. 다이얼로그 (모달별)

| 다이얼로그 | 키 → 동작 |
|---|---|
| **DLG-KILL** (KillConfirm) | `Enter`=SIGTERM 전송 · `Esc`=취소 |
| **DLG-KILLTREE** (KillTreeConfirm) | `Enter`=`kill_tree`(SIGTERM, 자식부터 역순) · `Esc`=취소 |
| **DLG-SIGNAL** (SignalPicker) | `Up`/`k`·`Down`/`j`=이동 · `Enter`=선택 시그널 전송 · `Esc`=취소 |
| **DLG-FORCE** (ForceKillPrompt) | `Enter`=`force_kill`(SIGKILL) · `Esc`=취소 |
| **DLG-HELP** (Help) | `Esc`/`H`/`q`=닫기 · `Up`/`k`·`Down`/`j`=한 줄 스크롤 · `PgUp`/`PgDn`=한 페이지 · `Home`/`End`=맨 위/아래 |

### 2-6. 컨텍스트 의존 하단바 (BAR-BOTTOM · `status_bar.rs::render_bottom_bar` 실측)

> 같은 화면이라도 **상황별로 다른 키힌트**가 뜬다(발견성). 그 순간 유효한 키만 보여준다.

```
 리스트 포커스 : [q] Quit | [Up/Down] Navigate | [PgUp/Dn] Page | [Enter] Expand | [Tab] Details
                 | [Space] Select | [/] Filter | [s] Sort | [x] Kill | [S] Signal
 상세 포커스   : [Esc] List | [Tab] Next Tab | [S-Tab] Prev Tab | [Up/Down] Scroll | [PgUp/Dn] Page
                 | [x] Kill | [S] Signal
 필터 활성     :  /{입력텍스트}█  | [Enter] Apply | [Esc] Cancel
 Kill 계열 모달:  [Enter] Confirm | [Esc] Cancel        (KillConfirm·KillTreeConfirm·ForceKillPrompt 공통)
 SignalPicker  :  [Up/Down] Select | [Enter] Send | [Esc] Cancel
 Help 모달     :  [Esc] Close
```

> **발견성 보강 메모(research-s3 §9-(1)):** 하단바가 컨텍스트별이라 일부 전역키(`K` 트리 kill, `e` 전체
> 확장, `n` Node-only, `r` 반전, `+/-` refresh)는 상시 노출되지 않는다 → **`H`(Help) 모달이 전체 키의
> 정식 노출 경로**다(§3 DLG-HELP가 그 인앱 표현). CLI 도구 관행상 허용 범위(결함 아님).

---

## 3. 화면별 명세 + 컴포넌트 배치 (+ 핵심 화면 4상태)

> 각 화면: **컴포넌트 배치 다이어그램** + **명세 9요소**. 저충실도 전체 프레임은 `20-wireframes` 참조.
> 핵심 화면(V-LIST·TAB-LOG·TAB-NET)은 **empty/loading/normal/error 개별 미니 시안**을 추가한다.

### 3-1. BAR-TOP · 상단 상태바
`[BAR-TOP · status_bar.rs::render_top_bar]`

```
┌ BAR-TOP (1줄·DarkGray 배경) ─────────────────────────────────────────────────────────────┐
│ [ntop v{ver}] · [CPU: {x}%] · [MEM: {used}/{total}MB] · [{Servers|Nodes}: {n}][Node-only] · [Refresh: {N}s] · [H]elp │
│   Cyan볼드      CPU색가변        White                 White·Yellow표식           White       Yellow │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```
- **목적:** 시스템 부하 맥락 + 분류 서버 수 + refresh 주기를 상시 노출(F18, PRB-6).
- **구성요소:** 버전 · 시스템 CPU%(>80 Red/>50 Yellow/else Green) · 시스템 MEM(used/total MB) · 서버 수
  (Node-only면 라벨 `Servers:`→`Nodes:` + ` [Node-only]` 표식) · refresh 초 · `[H]elp` 힌트.
- **상호작용(키):** 없음(표시 전용). 값은 `n`(Node-only)·`+/-`(refresh)·스캔 tick으로 갱신.
- **상태:** 상시(normal만). 서버 0이면 `Servers: 0`.
- **예외:** 없음.
- **연결 기능(10):** F18(정확 메모리·시스템 맥락) · F13(Node-only 라벨) · F1(서버 수).
- **연결 화면(12):** V-MAIN의 고정 크롬. · **연결 데이터(31):** `App.system_cpu/memory_*`·`node_only`·
  `refresh_secs`·`flat_list`(서버 수 카운트). · **연결 인터페이스(→32):** 없음(내부 스캔값).

### 3-2. BAR-BOTTOM · 하단 키힌트 바
`[BAR-BOTTOM · status_bar.rs::render_bottom_bar]`

```
┌ BAR-BOTTOM (1줄·DarkGray 배경) ─ 컨텍스트 의존(§2-6) ─────────────────────────────────────┐
│ [key] desc | [key] desc | …   (포커스/모드/모달에 따라 키힌트 집합이 통째로 교체됨)        │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```
- **목적:** 그 순간 쓸 수 있는 키만 실시간 노출(발견성).
- **구성요소:** `[key] desc`(Yellow 키) + `|` 구분자. 5가지 컨텍스트(§2-6).
- **상호작용(키):** 없음(표시 전용). 분기는 `App.dialog`·`filter_active`·`focus`로 결정.
- **상태:** 상시. 컨텍스트 5종으로 내용 교체.
- **예외:** 없음. · **연결 기능(10):** 전반(발견성). · **연결 화면(12):** V-MAIN 고정 크롬.
- **연결 데이터(31):** `App.{dialog, filter_active, focus, filter_text}`. · **연결 인터페이스(→32):** 없음.

### 3-3. V-MAIN · 메인 합성 뷰
`[V-MAIN · ui.rs::render · 진입: `ntop`]`

```
┌──────────────────────────── V-MAIN (세로 3단) ───────────────────────────────┐
│ BAR-TOP  (1줄)                                                  [크롬·상시]    │
├──────────────────────────────────────────────────────────────────────────────┤
│ main_content (가변 ≥5) ── 둘 중 하나로 렌더 ──                                 │
│   (A) flat_list 비었음  → V-EMPTY (전체 대체)                                  │
│   (B) flat_list 있음    → Layout::vertical([55%,45%])                          │
│        ├ V-LIST   (위 55%·포커스1)                                             │
│        └ V-DETAIL (아래 45%·포커스2·탭 Info/Log/Net/Env)                       │
├──────────────────────────────────────────────────────────────────────────────┤
│ BAR-BOTTOM (1줄)                                               [크롬·상시]     │
└──────────────────────────────────────────────────────────────────────────────┘
   오버레이(한 번에 하나): V-FILTER(/) · DLG-*(x·K·S·H·timeout)
```
- **목적:** 식별→점검→안전 종료를 한 화면에 집약 → GUI 왕복 제거(F20, PRB-8).
- **구성요소:** BAR-TOP + main_content(V-LIST/V-DETAIL 또는 V-EMPTY) + BAR-BOTTOM + 오버레이.
- **상호작용(키):** 전역 키(§2-1) + 포커스별(§2-2/2-3). `Tab`으로 V-LIST↔V-DETAIL.
- **상태:** loading/empty(→V-EMPTY) · normal(분할). 전체 에러 화면은 두지 않음(국소 수렴).
- **예외:** 터미널 폭/높이 부족 시 ratatui가 신축·절단(상세 height<3이면 탭 콘텐츠 생략).
- **연결 기능(10):** F20·F1·F5·F10·F15. · **연결 화면(12):** V-LIST·V-DETAIL·V-EMPTY·오버레이 부모.
- **연결 데이터(31):** `App`(애그리거트 루트). · **연결 인터페이스(→32):** `run_tui`(라이브러리 진입).

---

### 3-4. ★ V-LIST · 프로세스 목록 (핵심 화면 — 4상태 제공)
`[V-LIST · process_list.rs · 진입: TUI 기본 포커스]`

**컴포넌트 배치:**
```
┌ Processes ──────────────────────────────────────────────── (filter: …) ─┐  ← 블록 제목(필터 시 표기)
│ [헤더행] (빈) PID NAME PORT THR CPU MEM USER STS UPTIME  + 정렬자 ^/v     │  ← Yellow 볼드
│ [데이터행] [ ]/[x] ● {트리프리픽스}{NAME(+인라인배지)} … 컬럼들           │  ← 서버=LightCyan / 트리부모=DarkGray
│ …                                                                        │
│ (선택행: DarkGray 배경 + 볼드 강조 / row_highlight)                       │
└──────────────────────────────────────────────────────────────────────────┘
```
- **목적:** 분류된 서버를 트리로 식별·선택(F2·F10·F12, PRB-1·4).
- **구성요소:** 헤더(컬럼+정렬자) · 행(체크박스 `[ ]`/`[x]` + 헬스 `●` + 트리 `▾`/`▸`/`├`/`└` + NAME
  인라인 배지 + 9컬럼) · 블록 제목(필터 시 `(filter: 텍스트)`) · 활성 테두리 Cyan.
- **상호작용(키):** §2-2 전체(이동·트리 토글·확장/축소·`Space` 다중선택·`Tab`→상세). 전역 §2-1.
- **상태:** **empty/loading → V-EMPTY로 대체** · normal(트리) · error(스캔 실패는 빈 결과로 수렴→empty).
- **예외:** 분류 0건 → V-EMPTY. 좁은 폭(≤100)이면 컬럼만 축소(컬럼 수 유지).
- **연결 기능(10):** F2·F4·F10·F11·F12·F13·F14. · **연결 화면(12):** V-MAIN 자식, `Tab`→V-DETAIL.
- **연결 데이터(31):** `App.flat_list:Vec<(ProcessInfo,depth)>`·`selected_index`·`selected_pids`·
  `expanded_pids`·`sort_column`·`ProcessInfo`(전 컬럼 필드). · **연결 인터페이스(→32):** 내부 스캔
  (`ProcessScanner`); 포트는 `NetworkInspector::connections_by_pid`(32 계약).

**4상태 미니 시안:**
```
 [empty]  프로세스 0건 → V-EMPTY로 대체:        [loading]  첫 스캔 중 → V-EMPTY(같은 위젯)+스피너:
 ┌───────────── ntop ─────────────┐             ┌───────────── ntop ─────────────┐
 │  No server processes found.     │             │  ⠹ Scanning for server          │  ← tick_count 회전
 │  Waiting...                     │             │    processes...                 │
 └─────────────────────────────────┘             └─────────────────────────────────┘

 [normal]  분류 서버 트리:                       [error]  스캔 실패/0건 → 빈 결과로 수렴(=empty):
 ┌ Processes ─────────────────────┐             ┌───────────── ntop ─────────────┐
 │ [ ]● ▾ 932 claude  …            │             │  No server processes found.     │  ← 전용 에러 화면 없음.
 │ [ ]●  ├▾ 41822 next-server(Next)│             │  Waiting...                     │     권한거부(EPERM)는 목록이 아니라
 │ [x]●  └─ 41835 vite …           │             └─────────────────────────────────┘     kill 다이얼로그 결과로 노출(§3-12)
 └─────────────────────────────────┘
```

---

### 3-5. V-DETAIL · 상세 패널(4탭 컨테이너)
`[V-DETAIL · detail_panel.rs · 진입: `Tab`(V-LIST→상세)]`

```
┌ Details ─────────────────────────────────────────────────────────────────┐
│  Info │ Log │ Net │ Env        ← 탭바(divider "|", 활성 Cyan 볼드+밑줄)     │
│  {활성 탭 콘텐츠: TAB-INFO/LOG/NET/ENV 중 하나}                            │
└────────────────────────────────────────────────────────────────────────────┘
   (선택 프로세스 없음 → 중앙 "Select a process to view details")
```
- **목적:** 선택 프로세스의 4탭 상세 점검(F15, PRB-1·6).
- **구성요소:** 탭바(Info/Log/Net/Env) + 활성 탭 콘텐츠. 활성 테두리 Cyan.
- **상호작용(키):** §2-3(탭 순환·스크롤·`Esc`→리스트). 상세에서도 `x`·`S` 가능.
- **상태:** normal(콘텐츠) · 프로세스 미선택 → "Select a process to view details" · 탭별 상태는 §3-6~3-9.
- **예외:** inner height<3 또는 width<4 → 콘텐츠 렌더 생략(공간 부족).
- **연결 기능(10):** F15. · **연결 화면(12):** TAB-INFO/LOG/NET/ENV 부모, V-MAIN 자식.
- **연결 데이터(31):** `App.active_tab:DetailTab`·`detail_scroll`·`selected_process()`. · **연결 인터페이스
  (→32):** 탭별(아래).

### 3-6. TAB-INFO · 정보 탭
`[TAB-INFO · info_tab.rs · 진입: 상세 기본 탭]`

```
 {key 우정렬 12칸}: {value}      ← 18줄 세로 key:value (Cyan 볼드 key / White value)
 CWD · Command · PID · PPID · Name · Runtime · Framework · Version · Ports ·
 CPU · Memory · Memory(VMS) · Threads · Uptime · User · Status · Health · Open FDs
```
- **목적:** 프레임워크/런타임 정체 + 메트릭 확인(F1·F18, PRB-1·6). **목록에 없는 FRAMEWORK 정체가 여기.**
- **구성요소:** 18개 key:value(세로). `Version`은 framework_version 미채움 → 항상 `-`.
- **상호작용(키):** §2-3 스크롤(`Up/Down`·`PgUp/Dn`·`Home`). 탭 순환.
- **상태:** normal만(선택 프로세스의 필드). empty/loading 없음(이미 선택된 행의 보유 데이터).
- **예외:** 일부 필드 OS 미지원(Windows open_fds=0). · **연결 기능(10):** F1·F18·F15.
- **연결 화면(12):** V-DETAIL 자식. · **연결 데이터(31):** `ProcessInfo` 18필드·`runtime`·`framework`·
  `health()`·`memory_display()`. · **연결 인터페이스(→32):** macOS `phys_footprint`(platform FFI, 30).

### 3-7. ★ TAB-LOG · 로그 탭 (핵심 화면 — 4상태 제공)
`[TAB-LOG · log_tab.rs · 진입: 탭 순환]`

```
  Source: {가장 최근 수정 로그파일 경로}        ← Cyan
  ─ ─ ─ … (구분선)
  {tail 라인들}  (ERROR/error=Red · WARN/warn=Yellow · DEBUG/debug=DarkGray · else White)
```
- **목적:** 선택 프로세스 cwd 로그 자동 감지·실시간 tail(F16, PRB-1).
- **구성요소:** Source 헤더 + 구분선 + tail 버퍼(최대 1000줄, 31). 색상화(레벨 키워드).
- **상호작용(키):** §2-3 스크롤. · **연결 기능(10):** F16. · **연결 화면(12):** V-DETAIL 자식.
- **연결 데이터(31):** `App.log_streamer:Option<LogStreamer>`(file/path/buffer)·`log_scroll`·
  `MAX_BUFFER_LINES=1000`. · **연결 인터페이스(→32):** 파일시스템 glob·tail(`LogStreamer`, 30-9).

**4상태 미니 시안:**
```
 [empty] cwd glob 0건:                           [loading] 소스 있음·출력 전:
 │  No log source detected                  │     │  Source: …/page.log              │
 │  ntop looks for log files in the process │     │                                  │
 │  working directory:                      │     │  Waiting for log output...       │
 │    *.log, logs/*.log, .next/trace,       │     └──────────────────────────────────┘
 │    npm-debug.log, etc.                   │
 └──────────────────────────────────────────┘

 [normal] 소스+버퍼:                              [error] 파일 접근/권한 실패:
 │  Source: …/page.log                      │     · Linux: cwd 실패 → /proc/<pid>/fd/1(stdout) fallback
 │  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ │     · fallback도 실패 → "No log source detected"(=empty로 수렴)
 │  ✓ Compiled /app in 412ms               │     · 전용 에러 패널 없음
 │  ERROR ECONNREFUSED …  (Red)            │
 └──────────────────────────────────────────┘
```

---

### 3-8. ★ TAB-NET · 네트워크 탭 (핵심 화면 — 4상태 제공)
`[TAB-NET · net_tab.rs · 진입: 탭 순환]`

```
  LOCAL (40%)           REMOTE (40%)          STATE (20%)     ← 표 헤더 Yellow 볼드
  0.0.0.0:3000          -                     LISTEN          ← LISTEN=Green
  127.0.0.1:3000        127.0.0.1:51544       ESTABLISHED     ← ESTABLISHED=Cyan
```
- **목적:** 선택 프로세스의 LISTEN 포트 + 활성 TCP 연결(F6, PRB-2).
- **구성요소:** 3컬럼 표(LOCAL/REMOTE/STATE). 상태색(LISTEN/ESTABLISHED/CLOSE_WAIT/TIME_WAIT).
- **상호작용(키):** §2-3 스크롤(skip 기반). · **연결 기능(10):** F6·F5. · **연결 화면(12):** V-DETAIL 자식.
- **연결 데이터(31):** `NetworkConnection`(local_addr/remote_addr/state/pid). · **연결 인터페이스(→32):**
  `NetworkInspector::connections_for_pid` → `lsof`(Unix)/`netstat`(Win) 파싱(32 시스템명령 계약).

**4상태 미니 시안:**
```
 [empty] 서버지만 연결 0:                        [loading] 직전 스캔값 유지:
 │  No active network connections found.    │     │ (포트는 분류 후 별도로 채워짐 — 스캔 중엔   │
 └──────────────────────────────────────────┘     │  직전 값 표시) → 시각상 normal과 동일       │

 [normal] 연결 표:                                [error] ★ lsof/netstat 부재 또는 실행 실패:
 │  LOCAL          REMOTE        STATE      │     │  No active network connections found.    │
 │  0.0.0.0:3000   -             LISTEN     │     └──────────────────────────────────────────┘
 │  127.0.0.1:3000 …:51544       ESTABLISHED│       ★ 명령 부재 → 빈 결과로 수렴(크래시 0, 도시에 §12)
 └──────────────────────────────────────────┘         empty와 동일 메시지 — 구분 불가가 의도된 단순화
```

---

### 3-9. TAB-ENV · 환경변수 탭
`[TAB-ENV · env_tab.rs · 진입: 탭 순환]`

```
  {KEY}={value}        ← KEY=Cyan 볼드 · "="=DarkGray · value=White
  민감 KEY → value="********"(8자)   (mask_env_values 기본 true · 31)
  민감 패턴: PASSWORD·SECRET·TOKEN·KEY·API_KEY·PRIVATE·CREDENTIALS·AUTH (대문자 부분일치)
```
- **목적:** 환경변수 점검 + 민감값 자동 마스킹(F17, PRB-6).
- **구성요소:** `KEY=값` 목록. 민감 패턴 포함 키는 `********`. · **상호작용(키):** §2-3 스크롤.
- **상태:** normal · empty("No environment variables available.") · loading 없음 · error(권한 부족 시 일부 미수집).
- **예외:** 권한 부족 시 env_vars 일부 누락(부분 결과). · **연결 기능(10):** F17.
- **연결 화면(12):** V-DETAIL 자식. · **연결 데이터(31):** `ProcessInfo.env_vars:Vec<(String,String)>`·
  `Config.display.mask_env_values`. · **연결 인터페이스(→32):** 없음(스캔 시 수집). 마스킹 규칙 정규 출처 = `33`.

---

### 3-10. V-FILTER · 필터 모드
`[V-FILTER · ui.rs::handle_filter_key + status_bar.rs · 진입: `/`]`

```
┌ Processes (filter: {텍스트}) ───────────────────────────┐   ← 좁혀진 목록(실시간)
│ (매칭 행만 표시)                                          │
└──────────────────────────────────────────────────────────┘
 /{텍스트}█  | [Enter] Apply | [Esc] Cancel                    ← 하단바(커서 █ Yellow)
```
- **목적:** 부분일치로 대상 좁히기(F11, PRB-1·4). · **구성요소:** 하단바 입력 표시 + 좁혀진 목록 + 블록
  제목 `(filter: …)`.
- **상호작용(키):** §2-4(Char/Backspace/Enter 유지/Esc 비움, 입력마다 `rebuild_view()`).
- **상태:** 모드 활성(normal 위 오버레이). empty(매칭 0)면 목록 빔. loading/error 없음.
- **예외:** 매칭 0건 → 빈 목록(V-EMPTY 아님 — 필터 결과 0). · **연결 기능(10):** F11·F12.
- **연결 화면(12):** V-LIST 위 모드. · **연결 데이터(31):** `App.filter_text`·`filter_active`·
  `matches_filter()`. · **연결 인터페이스(→32):** 없음(내부 필터). 매칭 알고리즘 정규 출처 = `30`.

### 3-11. V-EMPTY · 빈/로딩 상태 (단일 위젯)
`[V-EMPTY · empty_state.rs · 진입: `flat_list` 비었을 때 자동]`

```
┌───────────────── ntop (제목 중앙) ─────────────────┐
│            ⠹  Scanning for server processes...      │  ← 스피너 Cyan(tick_count) + Gray 안내
│            No server processes found. Waiting...     │  ← DarkGray
│      Start a server and it will appear here ...      │  ← DarkGray
└──────────────────────────────────────────────────────┘
```
- **목적:** 프로세스 0건 안내 + "살아서 스캔 중" 표현(F1, PRB-1). **empty와 loading을 한 위젯으로 합침.**
- **구성요소:** 스피너(`⠋⠙⠹…` 회전) + 3줄 안내. · **상호작용(키):** 전역 키만(목록 비어 포커스 동작 무의미).
- **상태:** empty=loading(동일 위젯) · normal/error 없음(스캔 실패도 여기로 수렴).
- **예외:** 없음(항상 안전 표시). · **연결 기능(10):** F1. · **연결 화면(12):** V-MAIN 전체 대체.
- **연결 데이터(31):** `App.flat_list`(비었음)·`tick_count`(스피너). · **연결 인터페이스(→32):** 없음.

---

### 3-12. DLG-KILL · 단건 종료 확인
`[DLG-KILL(KillConfirm) · kill_dialog.rs · 진입: `x` · 테두리 Red · 50%×40% 중앙]`

```
┌──────── Kill Process ────────┐
│ Are you sure you want to kill │  ← Yellow
│ this process?                 │
│   PID:  {pid}   Name: {name}  │  ← key=Cyan/값=White
│   Port: {ports}   (있을 때만) │
│ Signal: SIGTERM (graceful)    │  ← DarkGray
│ [Enter] Confirm  [Esc] Cancel │  ← Enter=Green/Esc=Red
└───────────────────────────────┘
```
- **목적:** 파괴적 종료 전 확인 게이트로 오발사 차단(F7, PRB-3·5).
- **구성요소:** 안내문 + 대상(PID/Name/Port) + 시그널 표기 + 확인/취소. 포트는 있을 때만.
- **상호작용(키):** §2-5(`Enter`=`send_signal(pid, SIGTERM)` **1회 직접 전송** → `needs_rescan=true` /
  `Esc`=취소). **TUI는 폴링·자동 force 승격을 하지 않는다**(코드-진실 `ui.rs` L95-108).
- **상태:** normal(확인) → `Enter` 시 모달 닫힘·다음 스캔에서 종료 반영(별도 진행상태 화면 없음) ·
  **error: EPERM → PermissionDenied 메시지**(권한거부).
- **예외:** ★EPERM(권한거부) → `KillResult::PermissionDenied` 노출 · 이미 죽음(ESRCH) → AlreadyDead.
- **연결 기능(10):** F7. · **연결 화면(12):** V-LIST/V-DETAIL→여기(`Enter` 후 닫힘). **자동 graceful→force
  승격은 CLI 단건 전용**이라 TUI는 여기서 DLG-FORCE로 이어지지 않는다(DLG-FORCE는 현재 미도달 — §3-15·`40` M1).
- **연결 데이터(31):** `KillSignal::Term`·`KillResult`·`Config.confirm_before_kill`.
  (`App.kill_in_progress:Option<(u32,Instant)>`는 31에 선언돼 있으나 **현재 어디서도 대입되지 않는 미사용
  필드** — `40` M1.) · **연결 인터페이스(→32):** `ProcessKiller::send_signal`(Unix nix `signal::kill` /
  Windows `taskkill`; CLI 단건의 자동 에스컬레이션은 `graceful_kill`). 정책 정규 출처 = `33`.

### 3-13. DLG-KILLTREE · 트리 종료 확인
`[DLG-KILLTREE(KillTreeConfirm) · kill_dialog.rs · 진입: `K` · 테두리 Red]`

```
┌────── Kill Process Tree ──────┐
│ Kill process tree? This will  │  ← Yellow
│ terminate:                    │
│   Root: PID {pid} ({name})    │  ← Root=Cyan/값=White
│   Children: {N} process(es)   │  ← N=count_descendants(Red)
│     - PID {c} ({cname})       │  ← 직계 자식만 나열(DarkGray)
│ [Enter] Confirm  [Esc] Cancel │
└───────────────────────────────┘
```
- **목적:** 부모+자식 트리 일괄 종료(자식부터 역순)로 고아 방지(F8, PRB-5).
- **구성요소:** Root + 자손 총수(`count_descendants`) + 직계 자식 목록 + 확인/취소.
- **상호작용(키):** §2-5(`Enter`=`kill_tree(SIGTERM, 역순)` / `Esc`=취소).
- **상태:** normal · 각 PID 결과 `KillResult`(Success/AlreadyDead/PermissionDenied/Error).
- **예외:** 일부 PID EPERM → 해당 PID만 PermissionDenied(나머지는 진행). · **연결 기능(10):** F8.
- **연결 화면(12):** V-LIST→여기. · **연결 데이터(31):** `ProcessInfo.children`(자기참조 1:N)·`KillResult`.
- **연결 인터페이스(→32):** `ProcessKiller::kill_tree`→`Vec<(pid,KillResult)>`. 역순 규칙 정규 출처 = `33`.

### 3-14. DLG-SIGNAL · 시그널 선택
`[DLG-SIGNAL(SignalPicker) · signal_picker.rs · 진입: `S` · 테두리 Cyan · 45%×50%]`

```
┌──────── Signal Picker ────────┐
│ Select a signal to send:      │  ← Yellow
│ ▸ SIGTERM  - Graceful …       │  ← ▸=선택(Cyan 볼드), 이름 좌정렬 10칸
│   SIGKILL  - Force kill …      │
│   SIGHUP   - Hangup / reload … │  ← Unix만
│   SIGINT   - Interrupt …       │
│   SIGUSR1  - Node.js debugger  │  ← Unix만
│   SIGUSR2  - User-defined …    │  ← Unix만
│ [Enter] Send [Up/Down] [Esc]  │
└───────────────────────────────┘
```
- **목적:** 임의 시그널 정밀 전송(종료 외 reload/디버거 등)(F9, PRB-5).
- **구성요소:** 안내 + 시그널 목록(`KillSignal::all` — Unix 6/Win 3) + 설명 + 전송/이동/취소.
- **상호작용(키):** §2-5(`Up/Down` 이동·`Enter` 선택 전송·`Esc` 취소).
- **상태:** normal · 전송 결과 `KillResult`. · **예외:** EPERM → PermissionDenied. Windows는 3종만 노출.
- **연결 기능(10):** F9. · **연결 화면(12):** V-LIST/V-DETAIL→여기.
- **연결 데이터(31):** `KillSignal`(name/description/all)·`App.signal_picker_index`. · **연결 인터페이스
  (→32):** `ProcessKiller::send_signal`. OS×시그널 가용성·EPERM 정책 정규 출처 = `33`.

### 3-15. DLG-FORCE · 강제 종료 확인 (현재 미도달 죽은 코드)
`[DLG-FORCE(ForceKillPrompt) · kill_dialog.rs · 진입: (현재 코드에서 미도달 — 트리거 경로 없음) · 테두리 Red]`

```
┌──────── Force Kill ────────────┐
│ Graceful kill timed out.       │  ← Red
│ Send SIGKILL (force kill)?     │  ← Yellow
│   PID: {pid}                   │
│ WARNING: SIGKILL cannot be     │  ← Red 볼드
│ caught or ignored.             │
│ [Enter] Force Kill  [Esc] Cancel│
└────────────────────────────────┘
```
- **목적(설계 의도):** graceful 실패 후 "되돌릴 수 없는 SIGKILL"을 의식적 결정으로 만드는 TUI 강제 모달(F7,
  PRB-5). **단, 아래 상태/예외 참조 — 현재 이 화면을 띄우는 코드 경로가 없다.**
- **구성요소:** timeout 안내 + 대상 PID + SIGKILL 경고 + 강제/취소.
- **상호작용(키):** §2-5(`Enter`=`force_kill(SIGKILL)` / `Esc`=취소→여전히 실행). 핸들러는 `ui.rs` L183에
  실재한다(§2-5 키 카탈로그 유지).
- **상태(★코드-진실):** **트리거 경로가 현재 없음 — 미도달 죽은 코드.** `app.dialog=Some(...)`는
  KillConfirm/KillTreeConfirm/Help/SignalPicker 4종만 설정하고 `ForceKillPrompt`를 설정하는 지점이 없다.
  TUI `KillConfirm`은 SIGTERM 1회 직접 전송으로 끝나며(폴링·자동 force 없음, §3-12) 이 모달로 승격하지
  않는다. **자동 graceful→force는 CLI(`ntop kill`) 단건 전용**(13 §8 경로 ②). → 연결 또는 제거는 `40` M1.
- **예외:** SIGKILL은 catch 불가(즉사). · **연결 기능(10):** F7. · **연결 화면(12):** (현재 진입 경로 없음).
- **연결 데이터(31):** `GracefulResult::TimedOut`(CLI 단건 반환)·`KillSignal::Kill`. · **연결 인터페이스(→32):**
  `ProcessKiller::force_kill`. 에스컬레이션 정책 정규 출처 = `33` §2-1, 처리 알고리즘 = `30` §2-6·§8-3.

### 3-16. DLG-HELP · 도움말 모달
`[DLG-HELP(Help) · help_dialog.rs · 진입: `H` · 테두리 Cyan · 60%×70% · 스크롤바 ↑/↓]`

```
┌────────────── Help  [현재/총] ──────────────┐   ← 스크롤 시 페이지 표기
│ ntop v{ver}                              ↑   │
│ -- Process List (default focus) --           │   ← 섹션 헤더 Yellow 볼드
│   {key 좌정렬 14칸}  {desc}                   │   ← key=Green/desc=White
│ -- Detail Panel / Filtering & Sort /         │
│    Process Control / General / This Dialog -- │
│                                          ↓   │
└──────────────────────────────────────────────┘
```
- **목적:** 전체 키 바인딩의 인앱 노출(발견성 보강 §2-6 메모). **§2 키 카탈로그의 인앱 표현.**
- **구성요소:** 6섹션(Process List·Detail Panel·Filtering & Sort·Process Control·General·This Dialog) +
  스크롤바. · **상호작용(키):** §2-5 Help 행(Esc/H/q 닫기·스크롤).
- **상태:** normal(스크롤). · **예외:** 없음. · **연결 기능(10):** 발견성(전반).
- **연결 화면(12):** V-MAIN 위 모달. · **연결 데이터(31):** `App.help_scroll`/`help_max_scroll`.
- **연결 인터페이스(→32):** 없음.

---

## 4. 핵심 화면 4상태 매트릭스 + 에러 트리거 (P-4·P-5)

> §3의 개별 미니 시안을 한눈 매트릭스로 종합. ★ = playbook이 명시한 핵심 에러 트리거.

| 화면 | empty | loading | normal | error(트리거) |
|---|---|---|---|---|
| **V-LIST** | V-EMPTY "No server… Waiting" | V-EMPTY 스피너 `⠹ Scanning…` | 트리 목록(컬럼·정렬·확장) | 스캔 실패 → 빈 결과로 수렴(=empty). 전용 에러 화면 없음 |
| **TAB-LOG** | "No log source detected" + 패턴 안내 | "Source: …" + "Waiting for log output…" | Source 헤더 + tail(레벨 색상) | ★파일 접근/권한 실패 → Linux `/proc/<pid>/fd/1` fallback → 실패 시 empty 수렴 |
| **TAB-NET** | "No active network connections found." | 직전 스캔값 유지(시각상 normal) | LOCAL/REMOTE/STATE 표 | ★`lsof`/`netstat` 부재·실패 → 빈 결과(empty와 동일 메시지, 크래시 0) |
| **TAB-ENV** | "No environment variables available." | — | `KEY=값`(민감값 `********`) | 권한 부족 → env_vars 부분 누락(부분 결과) |
| **DLG-KILL** | — | (없음 — TUI는 `Enter` 시 SIGTERM 1회·즉시 닫힘, 폴링 없음; CLI 단건만 200ms 폴링) | 확인 프롬프트(대상·포트) | ★EPERM → `KillResult::PermissionDenied` 메시지 |

**에러 트리거 정리(코드-진실):**
```
 ★ EPERM(권한거부)      : send_signal/graceful_kill → KillResult::PermissionDenied → 다이얼로그 결과 메시지
                          (예: 다른 사용자/루트 소유 프로세스 종료 시도)
 ★ lsof/netstat 부재    : NetworkInspector가 빈 HashMap 반환 → 포트 빈칸·"No active network connections found."
                          (도시에 §12: 외부 명령 의존, 명령 없으면 빈 결과 — 크래시 아님)
 · 로그 접근/권한 실패  : Linux는 /proc/<pid>/fd/1 fallback, 그 외/실패 시 "No log source detected"
 · 스캔 실패/0건        : 전용 에러 화면 없이 V-EMPTY로 수렴 (사용자는 항상 메인 뷰에 머묾, PRB-8)
```

**상태 설계 원칙(research-s3 §6 승계):** ntop은 **전체 에러 화면을 거의 두지 않는다.** 에러는 **빈 결과로
수렴**(lsof 없으면 포트 빈칸)하거나 **국소 메시지**(EPERM→PermissionDenied)로 노출돼, 사용자는 항상 메인
뷰에 머문다 → 상태 차원에서도 PRB-8(문맥 전환)을 줄인다.

---

## 자체 검증 (Self-verification)

| # | 체크 | 결과 | 어디서 충족 |
|---|---|---|---|
| **P-1** | 키 바인딩 카탈로그 = 정규 출처(5그룹+하단바) | **PASS** — §2-1~§2-6(전역/리스트/상세/필터/다이얼로그/컨텍스트 하단바), 코드 근거(ui.rs·status_bar.rs·help_dialog.rs) | §2 |
| **P-2** | 화면당 컴포넌트 배치 시안(16 TUI 화면) | **PASS** — §3-1~§3-16 각 컴포넌트 다이어그램(산문-only 0) | §3 |
| **P-3** | 화면별 명세 9요소 | **PASS** — 각 화면 목적/구성요소/상호작용/상태/예외/연결 기능(10)/화면(12)/데이터(31)/인터페이스(→32) | §3 |
| **P-4** | 핵심 화면 4상태 개별 미니 시안 | **PASS** — V-LIST(§3-4)·TAB-LOG(§3-7)·TAB-NET(§3-8) 각 4상태 + §4 매트릭스 | §3·§4 |
| **P-5** | 에러 트리거 명시 | **PASS** — §3 예외 + §4(★EPERM·★lsof/netstat 부재·로그 접근 실패·스캔 실패) | §3·§4 |
| **P-6** | 사실 정정 반영 | **PASS** — §0(세로 레이아웃·FRAMEWORK 컬럼 없음·체크박스 `[ ]`/`[x]`) + 본문 일관 | §0 |
| **P-7** | 시각화(G-g, 외부 렌더러 0) | **PASS** — §2~§4 전부 순수 ASCII(컴포넌트·4상태·매트릭스), Mermaid 미사용 | 전체 |
| **P-8** | 단일 출처 준수(키만 소유, 나머지 참조) | **PASS** — 기능=10·화면=12·플로우=13·데이터=31·알고리즘=30·계약=32·정책=33 참조 표기 | 전체 |

**추가 원칙 점검:**
- **창작 금지(spec §7):** 없는 화면(설정 마법사·로그인·대시보드) 명세 0. 결제/DB/REST 0. 실제 위젯
  렌더 화면 16종만(`status_bar`·`process_list`·`detail_panel`·`info/log/net/env_tab`·`kill_dialog`·
  `signal_picker`·`help_dialog`·`empty_state`).
- **키 카탈로그 코드-검증:** §2가 `ui.rs`(키 핸들링)·`status_bar.rs`(하단바 실측)·`help_dialog.rs`(인앱
  도움말 6섹션)와 일치. `+`/`-` 범위(1~60s)·`BackTab`(Shift+Tab)·Help 스크롤키까지 반영.
- **사실 역추적성:** 모든 화면에 위젯 파일 + 4상태는 실제 분기(empty/buffer/EPERM/명령부재) 근거.
  레이아웃·컬럼·다이얼로그 문자열은 §3에서 소스 확인된 코드-진실.
- **쉬운 설명(probe):** 첫 등장 용어(TUI·패널·탭·모달·포커스·4상태·graceful·에스컬레이션·EPERM) 풀이.

**알려진 한계(P-1 휴먼 체크포인트):**
- **컴포넌트 배치 시안의 폭/정렬** 은 가독성 예시 — 실제는 터미널 폭 신축(시각 품질은 채점 제외).
- **저충실도 전체 프레임**은 `20-wireframes`가, **CLI 화면 명세/출력 스키마**는 `32-api-spec`이 소유 —
  본 문서는 TUI 16종 명세 + 키 카탈로그 + 4상태만 담당.
- **키/Config 값이 코드에서 바뀌면** 본 §2(키)·`31`(값)이 1순위 갱신 대상.
