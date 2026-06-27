# 13 · 유저 플로우 / 유저 저니 (ntop)

> 이 문서는 ntop **유저 플로우의 단일 출처**다. 사용자가 ntop으로 **무엇을 어떤 순서로** 하는지(주요
> 경로 + 분기 + 상태 전이)를 플로우차트로 정리하고, 각 단계를 **화면 ID(`12-ia`) · 기능 ID(`10-prd`) ·
> 문제(PRB, `research-s2`)** 로 링크한다.
>
> **ntop은 터미널 도구다.** 플로우의 "화면"은 웹 페이지가 아니라 **TUI 뷰/패널/탭/다이얼로그 + CLI 명령
> 출력**이다(spec §4-0).
>
> **플로우 ID 표기:** 본 문서는 플로우를 **`UF-1`~`UF-6`**(User Flow)으로 부른다. 기능 ID(`F1`~`F21`,
> 10-prd)와의 **혼동을 피하기 위해** 일부러 다른 접두어를 쓴다(연구 산출물 `research-s3`의 플로우
> "F1~F6"에 1:1 대응 — §1 인덱스 참조).
>
> **참조(재정의 0):** 화면 ID는 **`12-ia`**, 기능 ID(F1~F21)·우선순위는 **`10-prd`**, **키 바인딩의
> 정규 출처는 `21-screen-spec`**(현재 `research-s3` §7), 문제(PRB-1~8)는 **`research-s2`(→03 §2)**,
> 페르소나(P1 박서연·P2 정태호·P3 이준)는 **`03-personas`**, Config 기본값(`graceful_timeout=10` 등)·
> `GracefulResult`/`KillSignal` enum은 **`31-erd`**, kill 처리 로직 정규 출처는 **`30-functional-spec`**
> 이다. 본 문서는 **플로우**를 소유하고 나머지는 인용만 한다(spec §4-3).
>
> **전제:** ntop은 **이미 v0.2.0까지 구현된 MIT 오픈소스 도구**다(codebase-facts §0). 모든 플로우는
> 신규 설계가 아니라 **이미 구현된 동작을 역설계**한 것이다(창작 0). 사실은 codebase-facts(이하 "도시에")
> §4·§5·§6·§7 **또는 실제 소스코드**(`src/tui/ui.rs`·`event.rs`·`process/killer.rs`·`main.rs`)에 역추적된다.
>
> **용어 풀이(첫 등장):** **유저 플로우** = 사용자가 목적을 이루기까지 거치는 화면·동작의 순서 / **2-pass
> 스캔** = 한 번 훑어 서버를 모으고(1패스) 다시 훑어 그 부모를 채우는(2패스) 2단계 스캔 / **graceful
> 종료(정상 종료)** = "정리하고 꺼져라"라고 **요청**(SIGTERM)하는 것, 강제 종료(SIGKILL)와 다름 /
> **에스컬레이션(escalation)** = 정상 요청이 안 통하면 강제로 **단계 올림** / **폴링(polling)** = 일정
> 간격으로 "아직 살아있나?"를 반복 확인 / **트리 종료** = 부모-자식을 묶어 함께 종료 / **헤드리스
> (headless)** = 화면(GUI) 없이 명령만으로 쓰는 환경(예: SSH 서버) / **상태 다이어그램** = 시스템이
> 거치는 상태와 그 사이 전이를 그린 그림 / **시퀀스 다이어그램** = 참여자(actor) 사이에 오가는 메시지를
> 시간순으로 그린 그림.

---

## Sprint Contract (self-proposed checks)

이 문서(g2의 `13-user-flows`)가 만족시킬 **관찰 가능한 체크**(playbook §S4-g2 관찰 바 "13=핵심 플로우·
분기·화면·기능 ID 링크·플로우차트+시퀀스" + spec §5(13행)·§8에서 도출).

| # | 체크 | 어떻게 충족하나 | 본문 위치 |
|---|---|---|---|
| **L-1** | **핵심 플로우 6종**(research-s3 F1~F6) 플로우차트 | §2~§7 UF-1~UF-6 ASCII 플로우차트(분기 `◇`) | §2~§7 |
| **L-2** | **각 단계 → 화면 ID(12) + 기능 ID(10) + PRB** 링크 | 각 플로우 단계 라벨 + §9 플로우×화면×기능×PRB 추적표 | §2~§7·§9 |
| **L-3** | **핵심 세션 상태 다이어그램** — kill graceful→force 에스컬레이션 | §8-1 상태기계(SIGTERM→200ms 폴링→graceful_timeout 10s→Terminated/TimedOut→SIGKILL) | §8-1 |
| **L-4** | **핵심 세션 시퀀스 다이어그램** — User↔App↔ProcessKiller↔OS↔Process | §8-2 시퀀스(GracefulResult 분기·TUI/CLI 에스컬레이션 차이) | §8-2 |
| **L-5** | **분기(branch) 명시** — graceful 결과·필터 종료·CLI 확인 등 | 각 플로우 `◇` 분기 + GracefulResult 5분기 | §2~§8 |
| **L-6** | **사실 정정 반영** — 세로 레이아웃(목록 위/상세 아래)·목록에 FRAMEWORK 컬럼 없음 | §0 표기 규약 + UF-1 단계 | §0·§2 |
| **L-7** | **시각화(G-g)** — 모든 플로우/세션 ASCII 다이어그램(외부 렌더러 0) | §2~§8 전부 ASCII | 전체 |
| **L-8** | **단일 출처 준수** — 화면=12·기능=10·키=21·문제=research-s2·Config값=31 참조 | 본문 참조 표기, 재정의 0 | 전체 |

> 게이트 공급: **G-g**(플로우차트·시퀀스/상태 다이어그램 적합), **C1 일부**(플로우↔문제 일관, 각 단계 PRB 링크), **G-a 보조**(기능 ID 링크).

---

## 0. 표기 규약 (코드-진실 포함)

```
 기호      의미
 ────────────────────────────────────────────────────────────
 ─▶        순방향 진행                ◇        분기(조건/사용자 결정)
 │ ▼       흐름 연결                  ↻        탭/상태 순환
 [화면ID]  12-ia의 화면(V-/TAB-/DLG-/CLI-)      (Fn)  10-prd 기능 ID
 PRB-n     research-s2 문제 번호      `키`     키 입력(정규 출처=21-screen-spec)
 ────────────────────────────────────────────────────────────
 ★ 코드-진실(반드시 준수, 12-ia §0):
   · 레이아웃 = 세로(상/하): V-LIST 위(55%) / V-DETAIL 아래(45%)  (ui.rs L37–44)
   · TUI 목록(V-LIST)에는 별도 FRAMEWORK 컬럼이 없다 → 별도 열로는
     상세 Info 탭(TAB-INFO) 또는 CLI `ntop list` 테이블에서만 확인.
   · 단, 목록 NAME 컬럼은 인라인 배지로 프레임워크를 노출한다
     (`next-server (Next.js)` / `uvicorn [Python]`, `process_list.rs` L121·L127).
     → "별도 FRAMEWORK 컬럼 없음"일 뿐, 프레임워크명은 NAME에서도 보인다.
```

---

## 1. 이 문서가 답하는 것 + 플로우 인덱스

ntop 사용은 **세 가지 가치 동작**으로 수렴한다 — **식별 → 점검 → 안전 종료**(research-s2 §5의 3대
능력군, 01 §4 가치 루프). 6개 핵심 플로우가 이 루프 위에 있다.

### 1-1. 플로우 인덱스 (UF-1~UF-6 ↔ research-s3 F1~F6)

| 플로우 ID | (research-s3) | 플로우 | 진입점 | 주 화면 | 주 기능 | 주 PRB | 페르소나 |
|---|---|---|---|---|---|---|---|
| **UF-1** | F1 | 기동→스캔(2-pass)→선택→상세 4탭 점검→종료 | `ntop` | V-EMPTY·V-LIST·V-DETAIL(4탭) | F1·F2·F5·F10·F15·F18 | 1·2·6·8 | P1 |
| **UF-2** | F2 | 필터(`/`)/정렬(`s`,`r`)/Node-only(`n`) 토글 | V-LIST | V-FILTER·V-LIST | F11·F12·F13 | 1·4 | P1·P3 |
| **UF-3** | F3 | kill 단건 (`x` → SIGTERM 1회; 자동 force는 CLI 전용) | V-LIST/V-DETAIL | DLG-KILL | F7 | 3·5 | P1 |
| **UF-4** | F4 | kill 트리 (`K`, 자식부터 역순) | V-LIST | DLG-KILLTREE | F8 | 5 | P2 |
| **UF-5** | F5 | SignalPicker (`S`)로 임의 시그널 | V-LIST/V-DETAIL | DLG-SIGNAL | F9 | 5 | P1 |
| **UF-6** | F6 | CLI: `list --json` → `kill --tree` | 셸/스크립트 | CLI-LIST·CLI-KILL | F19·F7·F8 | 7·5 | P2 |

### 1-2. 가치 루프 위의 6 플로우 (한눈)

```
                          ┌──────────────── ntop 가치 루프 ────────────────┐
                          │  식별  ──▶  점검  ──▶  안전 종료  ──▶ (반복)    │
                          └────────────────────────────────────────────────┘
            ┌──────────────┴──────────────┐        ┌────────┴────────┐
            ▼                              ▼        ▼                 ▼
  UF-1 기동→스캔→식별→점검→종료    UF-2 필터/정렬/Node-only   UF-3 kill 단건   UF-4 kill 트리
        (척추 플로우)                  (식별 좁히기)         (SIGTERM 1회)     (고아 방지)
                                                              UF-5 SignalPicker(정밀 제어)
                                                              UF-6 CLI 자동화(헤드리스, PRB-7)
```

---

## 2. UF-1 — 기동 → 스캔(2-pass) → 선택 → 상세 4탭 점검 → 종료

> 척추 플로우. research-s3 F1. PRB-1(식별)·PRB-2(포트)·PRB-6(점검)·PRB-8(왕복 제거). 화자 P1 박서연.

```
[기동]                                              [스캔: 2-pass + 포트]
$ ntop ─▶ 터미널 셋업(raw/AltScreen) ─▶ 이벤트 루프 ──Tick──▶ do_scan()       (F1·F5 / PRB-1·2)
  (CLI 진입 [A])   (main.rs run_tui)                 (event.rs)     │
                                                                   ▼
                          Pass1: 전체 열거 → classify → 서버(runtime=Some) 수집   (F1)
                          Pass2: 서버의 부모(launchd/claude/shell) backfill(runtime=None) (F10)
                          포트:  NetworkInspector LISTEN만 PID별 채움(분류 후 별도) (F5 / PRB-2)
                                                                   │
                           ┌─────────── ◇ flat_list 비었나? ───────┤
                           │ 예                                   아니오
                           ▼                                       ▼
                  [V-EMPTY] 스피너 "Scanning…"          [V-LIST] 트리 표시(first_load면 전체 확장)
                  (F1 / PRB-1 대기)                     상단바 [BAR-TOP] Servers:n  (F18 / PRB-1 식별 시작)
                           │ (서버 뜨면 다음 Tick에)             │
                           └─────────────────────────────────────┤
                                                                  ▼
[선택]  [V-LIST] 포커스 ─ `↑↓/jk` 이동 · `Enter` 트리 토글 · `→/l` 확장 · `←/h` 축소 ·   (F10·F12)
                          `Space` 다중선택 · `s` 정렬 · `/` 필터(→UF-2)                   / PRB-1·4
                          ※ 목록 컬럼: [✓]PID·NAME·PORT·THR·CPU·MEM·USER·STS·UPTIME (FRAMEWORK 없음)
                                                                  │  `Tab`
                                                                  ▼
[점검]  [V-DETAIL] 포커스 ─ `Tab`/`→/l` 다음 탭 · `BackTab`/`←/h` 이전 탭 · `↑↓` 스크롤    (F15 / PRB-1·6)
        ┌────────────────┬────────────────┬────────────────┬────────────────┐
        ▼                ▼                ▼                ▼
   [TAB-INFO]        [TAB-LOG]        [TAB-NET]        [TAB-ENV]
   프레임워크 정체·  실시간 로그 tail  LISTEN 포트·     환경변수
   CPU·메모리        (F16)            TCP 연결         (민감값 마스킹)
   (F1·F18/PRB-1·6)                  (F6 / PRB-2)     (F17 / PRB-6)
        │                │                │                │
        └────────────────┴───────┬────────┴────────────────┘   (`Esc` → [V-LIST] 복귀)
                                  ▼
[종료]  `q` 또는 `Ctrl+C` ─▶ should_quit ─▶ 터미널 복원(raw 해제·AltScreen 나감·커서 표시)  (F20 / PRB-8)
```

**UF-1이 말하는 것:** 한 화면(V-MAIN) 안에서 식별(V-LIST)→점검(V-DETAIL 4탭)→종료(`q`)가 끝난다.
프레임워크 정체는 목록이 아니라 **상세 Info 탭**에서 확인한다(코드-진실). 별도 GUI로 나갈 필요가 없어
PRB-8을 제거한다(F20). kill로 이어지면 UF-3로 분기한다.

---

## 3. UF-2 — 필터(`/`) / 정렬(`s`,`r`) / Node-only(`n`) 토글

> research-s3 F2. PRB-1(식별 좁힘)·PRB-4(멀티런타임↔Node 집중). 모두 화면 전환 없이 **인플레이스 갱신**
> (`rebuild_view()`). 화자 P1 박서연 / Node-only는 P3 이준.

```
 [V-LIST] (normal 포커스)
   │
   ├── `/` ─▶ [V-FILTER] (모드)  ─ `Char` 입력마다 rebuild_view() 실시간 ──┐   (F11 / PRB-1·4)
   │            하단바 " /{텍스트}█   [Enter]Apply  [Esc]Cancel"           │
   │            매칭: name·command·pid·framework·runtime·ports 부분일치(대소문자 무시)
   │                                                                       │
   │          ◇ 종료 방식                                                  │
   │          ├ `Enter` ─▶ 필터 **유지**하고 normal 복귀 (좁혀진 목록 유지)
   │          └ `Esc`   ─▶ filter_text **비우고** normal 복귀 (전체로) ────┘
   │
   ├── `s` ─▶ sort_column 순환 (PID→NAME→PORT→THR→CPU→MEM→USER→STATUS→UPTIME) ─▶ rebuild_view()  (F12 / PRB-1)
   ├── `r` ─▶ sort_ascending 반전 (헤더 ^/v 표시) ───────────────────────────▶ rebuild_view()  (F12 / PRB-1)
   │
   └── `n` ─▶ node_only 토글 ─▶ rebuild_view()                                                  (F13 / PRB-4)
              (Node 서버만 + 트리 부모 유지 · Deno/Bun은 숨김)
              상단바 [BAR-TOP] 라벨 Servers↔Nodes · `[Node-only]` 표식 토글
```

**분기 포인트:** 필터의 `Enter`(유지) vs `Esc`(비움)가 핵심 차이다. 정렬·Node-only는 토글이라 분기
없이 즉시 반영된다. 셋 다 **다음 스캔 tick을 기다리지 않고** 키 입력 즉시 목록이 바뀐다(저마찰).

---

## 4. UF-3 — kill 단건 (`x` → KillConfirm → `Enter` → SIGTERM 1회 직접; 폴링·자동 force 없음)

> research-s3 F3. PRB-3(오kill)·PRB-5(데이터 유실). 가장 위험한 플로우 → **확인 게이트 + SIGTERM 우선
> (정리 시간 부여)**의 안전. 화자 P1 박서연. **TUI는 SIGTERM을 1회만 보내고 자동 force 승격을 하지 않는다**
> (자동 graceful→force는 CLI 단건 전용 — §7·§8). 처리 로직 정규 출처 `30` §2-6·§8-3 / 정책 `33` §2-1.
> (상태기계/시퀀스 상세는 §8.)

```
 [V-LIST]/[V-DETAIL] ─ `x` ─▶ [DLG-KILL] (KillConfirm · 모달·위험)               (F7 / PRB-3·5)
                               │ "대상 PID·NAME·포트 표시 + [Enter]Confirm [Esc]Cancel"
                  ┌───────────◇ 사용자 결정
                  │ `Esc`                          │ `Enter`
                  ▼                                ▼
              취소(원상 복귀)            send_signal(pid, SIGTERM) 1회 직접 (ui.rs L95-108)
                                                   │  · 폴링 없음 · needs_rescan=true
                       ┌───────────────────────────┼───────────────────────┐
                       │ 전송 성공                  │ EPERM(권한 거부)        │ ESRCH(이미 죽음)
                       ▼                            ▼                        ▼
                 다음 스캔에서 종료 반영        PermissionDenied         AlreadyDead
                 (목록서 사라짐)               메시지 노출(권한 부족)    (이미 정리됨)
   ↳ 안전장치: ① 확인 게이트(PRB-3 오발사 차단) ② SIGTERM(graceful) 우선 — 프로세스에 정리 시간(PRB-5)
   ↳ 자동 force 승격(graceful→force)은 **CLI `ntop kill` 단건 전용**(§7). TUI 강제 모달 ForceKillPrompt
      (DLG-FORCE) 핸들러는 코드에 있으나 **현재 트리거 경로가 없는 미도달 코드**다(`40` M1).
```

**TUI vs CLI 차이(중요·코드-진실):** **TUI `KillConfirm`은 `send_signal(SIGTERM)`을 1회 직접 보내고 끝**이다
— 폴링도, 자동 SIGKILL 승격도 없다(`needs_rescan=true`만 세팅, `ui.rs` L95-108). **자동 graceful→force
에스컬레이션은 CLI(`ntop kill <PID>`) 단건 경로에서만** 일어난다(`main.rs` `graceful_kill`: SIGTERM→200ms
폴링→`graceful_timeout` 초과 시 자동 SIGKILL, §7·§8). `GracefulResult` 5분기(Terminated/TimedOut/
AlreadyDead/PermissionDenied/Error)는 CLI 단건 경로의 반환값이다(§8-1).

---

## 5. UF-4 — kill 트리 (`K`, KillTreeConfirm · 자식부터 역순)

> research-s3 F4. PRB-5(고아 프로세스 방지). 화자 P2 정태호(플랫폼/운영).

```
 [V-LIST] ─ `K` ─▶ [DLG-KILLTREE] (KillTreeConfirm · 모달·위험)                  (F8 / PRB-5)
                    │ "부모+자식 전체 트리 PID 표시 + [Enter]Confirm [Esc]Cancel"
        ┌──────────◇ 사용자 결정
        │ `Esc`                    │ `Enter`
        ▼                          ▼
     취소(원상)            kill_tree(pids, SIGTERM)
                              │  pids를 **역순(자식부터)** 으로 시그널 → Vec<(pid, KillResult)>
                              ▼
                      자식 먼저 정리 → 부모 정리  ⇒  고아 프로세스 0  (PRB-5 해소)
                      (각 PID 결과는 KillResult: Success/AlreadyDead/PermissionDenied/Error)
```

**왜 역순인가:** 부모를 먼저 죽이면 자식이 고아(orphan)로 남아 포트를 계속 점유할 수 있다. `kill_tree`는
트리 PID를 **자식부터** 보내 이 사고를 구조적으로 막는다(도시에 §5). CLI는 `ntop kill <PID> --tree`로
동일 동작(UF-6).

---

## 6. UF-5 — SignalPicker (`S`)로 임의 시그널

> research-s3 F5. PRB-5(정밀 제어 — 모든 제어가 "종료"는 아님). 화자 P1 박서연.

```
 [V-LIST]/[V-DETAIL] ─ `S` ─▶ [DLG-SIGNAL] (SignalPicker · 모달)                 (F9 / PRB-5)
                               │ 대상: PID·NAME 표시 + 시그널 목록(KillSignal::all)
                               │   Unix 6종 : Term · Kill · Hup · Int · Usr1 · Usr2
                               │   Windows 3종: Term · Kill · Int   (HUP/USR1/USR2 없음 · 도시에 §12)
                               │   각 항목 설명(예: Hup="reload config", Usr1="Node.js 디버거 활성")
                   ┌───────────◇ 조작
                   │ `Up/k`·`Down/j` 이동           │ `Esc` 취소
                   ▼                                ▼
              `Enter` ─▶ send_signal(pid, 선택 시그널)   원상 복귀
                         (예: SIGHUP=설정 리로드 · SIGUSR1=디버거 활성 — graceful/force 대신 정밀 제어)
```

**분기 포인트:** 시그널 선택(Up/Down) → `Enter` 전송 / `Esc` 취소. 종료가 아니라 **임의 제어**라 graceful
폴링·force 에스컬레이션이 없다(단발 전송). 시그널 가용성(Unix 6 / Win 3)의 정규 출처는 `31-erd`/`33-policy`.

---

## 7. UF-6 — CLI: `list --json` → `kill --tree <PID>` (헤드리스 자동화)

> research-s3 F6. PRB-7(헤드리스·스크립트 자동화). 화자 P2 정태호(SSH 운영 시나리오).

```
[자동화/SSH 헤드리스]                                  (research-s2 P2 운영자 시나리오)
$ ntop list --json                                                              (F19 / PRB-7)
  [CLI-LIST] ─▶ scan_blocking()(sleep+scan 1회) ─▶ classify ─▶ NetworkInspector
             ─▶ JSON 출력(모든 필드 + depth)
                           │  jq 등으로 파싱 → 대상 PID 추출
                           ▼
$ ntop kill --tree <PID> [--no-confirm]                                         (F19·F8·F7 / PRB-5·7)
  [CLI-KILL] ─▶ 재귀로 트리 PID 전부 수집
             ─▶ ◇ 확인 프롬프트  ── `--no-confirm`이면 생략
             ─▶ 각 PID graceful(SIGTERM → timeout 초과 시 **자동** SIGKILL)
             ─▶ 종료 결과 출력 → 종료코드 반환
   ↳ 동일 코어(ProcessScanner+NetworkInspector)를 TUI와 공유 → 스크립트에 그대로 연결(PRB-7)
```

**TUI와의 대비:** CLI는 확인을 **프롬프트(또는 `--no-confirm` 생략)** 로, 에스컬레이션을 **자동**으로
처리한다 — 대화형 모달(DLG-*)이 없다. 출력 스키마(JSON 필드·CSV 9컬럼·table 컬럼)의 정규 출처는
`32-api-spec`이다.

---

## 8. 핵심 세션 — kill graceful→force 상태/시퀀스 다이어그램

> ntop의 가장 위험하고 중요한 세션 = **종료**. **경로별로 동작이 다르다(코드-진실)**: **CLI 단건**
> (`ntop kill <PID>`)만 `graceful_kill`로 SIGTERM→200ms 폴링→`graceful_timeout` 초과 시 **자동 SIGKILL
> 에스컬레이션**한다. **TUI `KillConfirm`은 `send_signal(SIGTERM)`을 1회 직접** 보내고(폴링·자동 force
> 없음, `needs_rescan`만), TUI 강제 모달 `ForceKillPrompt`(DLG-FORCE)는 코드에 있으나 **현재 트리거 경로가
> 없는 미도달 코드**다(`40` M1). 처리 로직 정규 출처는 `30-functional-spec`(§2-6·§8-3), 정책은 `33`(§2-1),
> enum 정의는 `31-erd`.

### 8-1. 상태 다이어그램 (state machine) — L-3

```
                    ┌─────────┐
                    │  Idle   │
                    └────┬────┘
                         │  `x`  또는  `kill <PID>`   (사용자 종료 의도)
                         ▼
                  ┌──────────────┐
                  │  Confirming  │   TUI=[DLG-KILL]  ·  CLI=`[y/N]`
                  └──────┬───────┘   `Esc`/취소 → Idle
                         │  `Enter` / `y`  (또는 --no-confirm)
                         ▼
        ┌─────── 경로 분기 (코드-진실: 동작이 다름) ───────┐
        │                                                 │
  ① TUI KillConfirm                      ② CLI `ntop kill <PID>` 단건
        │                                                 │
 ┌──────▼───────────────┐          ┌──────────────────────▼──────────┐
 │ send_signal(pid,TERM)│          │ graceful_kill(pid, 10s)         │
 │ once, direct (L99)   │          │   step1: send SIGTERM           │
 │ no poll / no force   │          └──────────────┬──────────────────┘
 │ needs_rescan = true  │                         ▼
 └──────────┬───────────┘          ┌─────────────────────────────────┐ alive &&
            │                       │ poll is_alive @200ms            │◀─┐ !timeout
   next scan → 목록서 제거          │ until graceful_timeout 10s      │  │ → 반복
            │                       └──┬──────┬───────┬───────────┬───┘──┘
            ▼                         die    EPERM   ESRCH   timeout && alive
       (→ Idle)                        ▼      ▼       ▼          ▼
                          ┌───────────┐┌──────────┐┌─────────┐┌──────────────┐
                          │Terminated ││PermDenied││AlreadyD.││ TimedOut     │
                          │           ││ (EPERM)  ││ (ESRCH) ││ →force_kill  │
                          └─────┬─────┘└────┬─────┘└────┬────┘│  SIGKILL auto│
                                │           │           │     └──────┬───────┘
                                └───────────┴───────────┴────────────┘
                                        (모두 → 메시지 / needs_rescan → Idle)

   ※ TUI 강제 모달 DLG-FORCE(ForceKillPrompt)는 ①에 등장하지 않는다 — 핸들러·위젯은 코드에 있으나
     이 모달을 띄우는 경로가 현재 없음(미도달 죽은 코드, `40` M1). 자동 SIGKILL 승격은 ②(CLI 단건)에만 있다.
```

`GracefulResult` enum 매핑(killer.rs · 도시에 §5 · **CLI 단건 `graceful_kill`의 반환값** · 정의 정규 출처
31): **Terminated**(timeout 내 사망) · **TimedOut**(살아남음→**CLI 자동** force 에스컬레이션) ·
**AlreadyDead**(이미 죽음, ESRCH) · **PermissionDenied**(EPERM) · **Error**(기타). (TUI `KillConfirm`은
`graceful_kill`을 호출하지 않고 `send_signal`만 쓰므로 이 분기를 거치지 않는다 — `30` §2-6·§8-3.)

### 8-2. 시퀀스 다이어그램 (actor 간 메시지) — L-4

```
 ── 경로 ①: TUI 단건 (KillConfirm) — 코드-진실: SIGTERM 1회, 폴링·자동 force 없음 ──
 User      App(ui.rs)          ProcessKiller(killer.rs)     OS(nix kill / taskkill)   Process
  │           │                        │                            │                   │
  │─ `x` ────▶│ [DLG-KILL] 표시        │                            │                   │
  │─ `Enter`─▶│ send_signal(pid,Term) ▶│ signal::kill(pid,SIGTERM) ▶│── SIGTERM ───────▶│ (정리 시도)
  │           │ needs_rescan=true      │◀ KillResult::Success ──────│                   │
  │◀ 다음 스캔에서 목록서 사라짐 ──────│  (폴링/추가 시그널 없음)    │              (종료됨)✗
  │   [권한거부] send_signal → EPERM ─▶ KillResult::PermissionDenied ─▶ 사용자에 표시        │
  │   ※ 강제(SIGKILL) 모달 ForceKillPrompt는 핸들러만 있고 트리거 경로 없음 → 미도달(40 M1)   │

 ── 경로 ②: CLI 단건 (`ntop kill <PID>`) — 자동 graceful→force 에스컬레이션 ──
 User      App(main.rs)        ProcessKiller(killer.rs)     OS(nix kill / taskkill)   Process
  │─ `kill`─▶│ "[y/N]" 확인 ─▶ y     │ graceful_kill(pid,10s)     │                   │
  │           │                      ─▶│ signal::kill(pid,SIGTERM) ▶│── SIGTERM ───────▶│ (정리 시도)
  │           │ (200ms 폴링 루프)      │── is_alive(pid)? ─────────▶│── 존재확인 ──────▶│
  │   [경우 A: timeout 내 사망 → Terminated] ─▶ "terminated gracefully"          (종료됨)✗
  │   [경우 B: graceful_timeout 초과·생존 → TimedOut → ★자동 에스컬레이션(사용자 확인 없음)]   │
  │           │                        │ force_kill(pid) ─▶ signal::kill(SIGKILL) ▶── SIGKILL ▶│ (즉사)✗
  │   [Windows]  Term/Int→`taskkill /PID` · Kill→OpenProcess+TerminateProcess (도시에 §5)     │
```

**이 세션이 안전한 이유(PRB-3·5 해소):** ① **확인 게이트**가 오발사(PRB-3)를 막고, ② **SIGTERM(graceful)
우선** — 프로세스에 정리 시간을 줘 데이터 유실(PRB-5)을 막으며(TUI는 SIGTERM 1회만, CLI 단건은 추가로
200ms 폴링·timeout 대기), ③ **CLI 단건은 timeout 후 자동 SIGKILL 승격**으로 비대화형에서도 확실히 종료된다.
TUI에서 강제 종료가 필요하면 사용자가 직접 다음 단계를 결정한다(자동 즉사 없음). 트리 종료(UF-4)는 자식부터
역순이라 고아가 안 남는다.

---

## 9. 플로우 × 화면 × 기능 × PRB 추적표 — L-2

> 모든 플로우 단계가 화면 ID(12)·기능 ID(10)·PRB(research-s2)로 역추적됨을 확인한다. 키의 정규 출처는
> 21-screen-spec.

### 9-1. 플로우 단계 추적표

| 플로우 | 단계(요지) | 화면 ID(12) | 진입 키/명령 | 기능 ID(10) | PRB |
|---|---|---|---|---|---|
| **UF-1** | 기동 | (CLI 진입 [A]) | `ntop` | F20 | 8 |
| UF-1 | 스캔 2-pass + 포트 | — (do_scan) | Tick | F1·F5·F10 | 1·2 |
| UF-1 | 빈/로딩 | V-EMPTY | (자동) | F1 | 1 |
| UF-1 | 선택 | V-LIST | `↑↓/jk`·`Enter`·`Space` | F2·F10·F12 | 1·4 |
| UF-1 | 점검 4탭 | V-DETAIL·TAB-INFO/LOG/NET/ENV | `Tab`·`←→` | F15·F1·F6·F16·F17·F18 | 1·2·6 |
| UF-1 | 종료 | — | `q`/`Ctrl+C` | F20 | 8 |
| **UF-2** | 필터 | V-FILTER | `/` | F11 | 1·4 |
| UF-2 | 정렬 | V-LIST | `s`·`r` | F12 | 1 |
| UF-2 | Node-only | V-LIST·BAR-TOP | `n` | F13 | 4 |
| **UF-3** | 확인 | DLG-KILL | `x` | F7 | 3·5 |
| UF-3 | SIGTERM 1회 직접 | (needs_rescan) | `Enter` | F7 | 5 |
| CLI 단건(§7) | 자동 graceful→force | (graceful_kill) | (timeout 초과 시 자동) | F7 | 5 |
| **UF-4** | 트리 확인 | DLG-KILLTREE | `K` | F8 | 5 |
| UF-4 | 역순 종료 | (kill_tree) | `Enter` | F8 | 5 |
| **UF-5** | 시그널 선택 | DLG-SIGNAL | `S` | F9 | 5 |
| **UF-6** | JSON 추출 | CLI-LIST | `ntop list --json` | F19 | 7 |
| UF-6 | 트리 종료 | CLI-KILL | `ntop kill --tree <PID>` | F19·F8·F7 | 5·7 |

### 9-2. 양방향 커버리지 확인

```
 ① 모든 플로우 단계 → 화면·기능·PRB 매핑 있음 (위 §9-1 전 행에 값) ✔
 ② PRB-1~8 플로우 커버:
    PRB-1 식별 불가   → UF-1·UF-2                  ✔
    PRB-2 포트 역추적 → UF-1(TAB-NET·PORT 컬럼)     ✔
    PRB-3 오kill      → UF-3(확인 게이트)            ✔
    PRB-4 다중런타임  → UF-1·UF-2(Node-only)        ✔
    PRB-5 거친 종료   → UF-3·UF-4·UF-5·UF-6         ✔
    PRB-6 메모리/Env  → UF-1(TAB-INFO/ENV)          ✔
    PRB-7 헤드리스    → UF-6                          ✔
    PRB-8 GUI 왕복    → UF-1(단일 메인 뷰)            ✔
 ③ 주요 기능 커버: F1·F2·F5·F6·F7·F8·F9·F10·F11·F12·F13·F15·F16·F17·F18·F19·F20 등장
    (F3 fs미독·F4 멀티런타임·F14 다중선택·F21 설정은 플로우의 전제/단계로 내재 — 정규 출처 10-prd)
 ⇒ 고아 단계·미링크 플로우 없음.
```

---

## 자체 검증 (Self-verification)

| # | 체크 | 결과 | 어디서 충족 |
|---|---|---|---|
| **L-1** | 핵심 플로우 6종(F1~F6) 플로우차트 | **PASS** — §2~§7 UF-1~UF-6 ASCII 플로우차트(분기 `◇`) | §2~§7 |
| **L-2** | 각 단계 → 화면 ID(12)+기능 ID(10)+PRB 링크 | **PASS** — 단계 라벨 + §9-1 추적표 + §9-2 양방향 커버리지 | §2~§7·§9 |
| **L-3** | 핵심 세션 상태 다이어그램(graceful→force) | **PASS** — §8-1 상태기계(SIGTERM→200ms 폴링→10s→Terminated/TimedOut/EPERM/AlreadyDead→SIGKILL) | §8-1 |
| **L-4** | 핵심 세션 시퀀스 다이어그램 | **PASS** — §8-2 시퀀스(User↔App↔Killer↔OS↔Process), GracefulResult 분기·TUI/CLI 차이 | §8-2 |
| **L-5** | 분기 명시 | **PASS** — 필터 Enter/Esc(§3)·graceful 5분기(CLI 단건 §4·§8-1)·DLG-FORCE(미도달 핸들러) Enter/Esc·CLI 단건 자동 force(§7) | §3·§4·§7·§8 |
| **L-6** | 사실 정정(세로 레이아웃·FRAMEWORK 컬럼 없음) | **PASS** — §0 표기 규약 + UF-1 단계(프레임워크는 TAB-INFO에서) | §0·§2 |
| **L-7** | 시각화(G-g, 외부 렌더러 0) | **PASS** — §1~§9 전부 순수 ASCII(플로우차트 6·상태기계·시퀀스·가치 루프·커버리지), Mermaid 미사용 | 전체 |
| **L-8** | 단일 출처 준수(화면=12·기능=10·키=21·문제=research-s2·Config=31) | **PASS** — 본문 참조 표기, 플로우만 소유·나머지 재정의 0 | 전체 |

**추가 원칙 점검:**
- **창작 금지(spec §7):** 결제·정산·DB·REST·회원 플로우 0. 없는 화면/명령 지어내지 않음 — 실제 키·CLI·killer 로직만(도시에 §4·§5·§6·§7).
- **사실 역추적성:** 스캔 2-pass(do_scan)·graceful 폴링 200ms·graceful_timeout 10s·kill_tree 역순·GracefulResult 5분기·TUI vs CLI 에스컬레이션 차이 모두 도시에 §4·§5·§7과 일치. 코드: `event.rs`(Tick), `killer.rs`(graceful_kill/kill_tree/force_kill), `ui.rs`(DLG 분기).
- **플로우 ID 충돌 회피:** 플로우를 `UF-1~UF-6`로 표기해 기능 ID `F1~F21`(10-prd)과 혼동 차단 — research-s3 F1~F6에 1:1 대응(§1-1).
- **쉬운 설명(probe):** 첫 등장 용어(유저 플로우·2-pass 스캔·graceful·에스컬레이션·폴링·트리 종료·헤드리스·상태/시퀀스 다이어그램) 풀이 블록 제공.
- **C1 일부(플로우↔문제 일관):** 모든 플로우 단계에 PRB 링크, §9-2에서 PRB-1~8 전부 커버 확인.

**알려진 한계(후속 보완):**
- **키 바인딩 정규 출처는 `21-screen-spec`**(현재 `research-s3` §7). 본 문서의 키는 플로우 단계 표기용 인용이며 정규 키표를 재정의하지 않는다.
- **kill 처리 로직 정규 출처는 `30-functional-spec`**, `GracefulResult`/`KillSignal` enum 정의·Config 값(`graceful_timeout=10`·폴링 200ms)은 `31-erd` — 본 §8은 의미만 인용.
- CLI 출력 스키마(JSON 필드·CSV 9컬럼·table 컬럼) 상세는 `32-api-spec`이 단일 출처 — UF-6은 요지만 인용.
