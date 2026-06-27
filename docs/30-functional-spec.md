# 30 · 기능 명세 (Functional Spec) — 동작·알고리즘 단일 출처 (ntop)

> **이 문서는 단일 출처(single source)다.** ntop의 **동작·알고리즘**(탐지 2단계 분류·스캔 2-pass·CPU
> delta·graceful_kill 폴링·트리 역순 종료·네트워크 파싱·로그 스트리밍·health 판정·필터/정렬)은 **여기서만
> 절차로 확정**한다. `10-prd`(기능 ID F1~F21)·`21-screen-spec`(키 바인딩)·`32-api-spec`(외부 계약)·`41`은
> 본 문서의 알고리즘을 **참조**한다.
>
> **값은 여기서 정의하지 않는다.** 구조체/enum 필드와 Config 기본값(`refresh_interval=3`·
> `graceful_timeout=10`·`MAX_BUFFER_LINES=1000` 등)의 정규 출처는 **`31-erd`**, 안전·프라이버시 정책 규칙
> (확인 게이트·시그널 권한 매트릭스·마스킹 규칙·fs 미독 원칙)은 **`33-policy`**다. 본 문서는 그 값을 인용만
> 한다(중복정의 0).
>
> **전제:** ntop은 **이미 v0.2.0까지 구현된 MIT 오픈소스 도구**다(codebase-facts §0). 본 명세는 "앞으로
> 만들 기능"이 아니라 **이미 구현된 동작을 역설계한 입력/처리/출력/제약/검증/예외 기술**이며, 모든 절차는
> 실제 소스코드(`src/process/framework.rs`·`scanner.rs`·`killer.rs`·`network.rs`·`log/streamer.rs`·
> `tui/app.rs`·`main.rs`)에서 추출했다(창작 0).
>
> **용어 풀이(첫 등장):** **분류(classify)** = 프로세스가 어느 런타임/프레임워크 서버인지 판정 / **매처
> (matcher)** = 일치 여부를 검사하는 규칙 조각 / **delta** = 두 시점 측정값의 차이 / **graceful 종료** =
> "정리하고 꺼져라"라고 요청(SIGTERM), 강제 종료(SIGKILL)와 다름 / **에스컬레이션** = 요청이 안 통하면
> 강제로 단계 올림 / **폴링(polling)** = 일정 간격으로 반복 확인 / **field-mode** = lsof의 한 줄 한 필드
> 출력 형식 / **tail** = 파일 끝에 새로 붙는 줄만 따라 읽기 / **글롭(glob)** = `*.log` 같은 패턴 파일 검색.

---

## Sprint Contract (self-proposed checks)

이 문서(g4의 `30-functional-spec`)가 만족시킬 **관찰 가능한 체크**(playbook §S4-g4 관찰 바 + spec §5(30행)·§8).

| # | 체크 | 어떻게 충족하나 | 본문 위치 |
|---|---|---|---|
| **S-1** | **탐지 2단계 분류** 절차 — FRAMEWORK→RUNTIME→config dev runner, 매처 name_exact→command_binary→command_contains, None=미표시, normalize_name | §2-1 입력/처리/출력/예외 + §8-1 의사결정 플로우차트 | §2-1, §8-1 |
| **S-2** | **스캔 2-pass** — pass1 분류 서버(runtime=Some), pass2 부모 backfill(None), persistent System CPU delta, 포트 별도 채움 | §2-3 + §8-2 2-pass 플로우 | §2-3, §8-2 |
| **S-3** | **graceful_kill 처리** — SIGTERM→200ms 폴링→graceful_timeout(31값)→SIGKILL 에스컬레이션, GracefulResult 분기; **kill_tree 자식부터 역순** | §2-6·§2-7 + §8-3 상태도/플로우 | §2-6·§2-7, §8-3 |
| **S-4** | **네트워크 파싱** — lsof field-mode(unix)/netstat(win), parse_addr 폼, 부재 시 빈 결과 | §2-5 + §8-4 파싱 플로우 | §2-5, §8-4 |
| **S-5** | **로그 스트리밍** — LOG_PATTERNS 글롭·최신 파일·seek end·MAX_BUFFER_LINES(31값)·Linux /proc fallback | §2-9 + §8-5 플로우 | §2-9, §8-5 |
| **S-6** | **health 판정** — CPU/MEM ≥90 Critical/≥80 Warning, Zombie/Dead Critical | §2-12 (임계 정규 출처) | §2-12 |
| **S-7** | **시각화(G-g)** — 분류 플로우차트·스캔 2-pass·graceful→force 상태도·파싱·로그 플로우(외부 렌더러 0) | §8 전체 ASCII | §8 |
| **S-8** | **단일 출처 준수** — 값=31·정책=33·키=21 참조, 알고리즘만 소유 | 본문 곳곳 참조 표기 | 전체 |

---

## 0. 사실 정정(코드-진실) + 이 문서가 소유하는 것

> 일부 선행 표현이 "목록 NAME/FRAMEWORK 컬럼"·"좌/우 분할"로 적었으나 **코드와 불일치**다(10-prd §0과 동일
> 정정). 본 문서의 화면 연결 표기는 아래 코드-진실을 따른다.

```
 ① 레이아웃 = 세로(상/하) 분할          ② TUI 목록에 FRAMEWORK 컬럼 없음
    위 55% = 프로세스 목록                  목록 컬럼: [✓/health] PID·NAME·PORT·THR·CPU·MEM·USER·STS·UPTIME
    아래 45% = 상세 패널(Info/Log/Net/Env)  프레임워크 정체는 → 상세 Info 탭 + CLI `ntop list` 테이블에서만
```

**이 문서가 소유(정규 정의)하는 알고리즘:** 분류(§2-1)·display_name(§2-2)·스캔 2-pass+CPU delta(§2-3)·
트리 빌드(§2-4)·네트워크 파싱(§2-5)·graceful_kill+force(§2-6)·kill_tree(§2-7)·단건 시그널(§2-8)·로그
스트리밍(§2-9)·메모리 보고(§2-10)·env 마스킹 동작(§2-11, 규칙은 33)·health(§2-12)·필터(§2-13)·정렬
(§2-14)·Node-only(§2-15)·CLI 처리 골격(§2-16, 스키마는 32).

**참조(여기서 정의 안 함):** 모든 구조체/enum/Config 값 → **31** · 정책 규칙(권한·마스킹 대상·fs 미독) →
**33** · 키 바인딩 → **21** · 출력 스키마 → **32** · 기능 ID(F#)·우선순위·문제(PRB#) → **10-prd**.

---

## 2. 기능별 명세 (입력 / 처리 / 출력 / 제약 / 검증 / 예외)

### 2-1. 탐지 2단계 분류 — `FrameworkDetector::classify` (F1·F3·F4) · `framework.rs`

- **입력:** `name`(프로세스 이름), `command`(전체 커맨드라인), `&Config`. **파일시스템을 읽지 않는다**
  (cwd/package.json 미참조 — fs 미독 원칙, 정책 근거는 33).
- **출력:** `Option<(Runtime, FrameworkKind)>` — 서버면 `Some`, 어느 규칙에도 안 맞으면 **`None` →
  표시·집계에서 제외**(스캔 pass1이 `continue`).
- **처리(3티어 순서):**
  1. **티어1 — `FRAMEWORK_RULES`**(구체 프레임워크) 먼저 `match_tier`. 매칭되면 그 규칙의 (runtime,
     framework) 반환. (구체 FW가 맨몸 런타임을 이기게 — 예: `fastapi`가 `python`을 이김.)
  2. **티어2 — `RUNTIME_RULES`**(맨몸 인터프리터) `match_tier`. 매칭되면 (runtime, `Generic`) 반환.
  3. **티어3 — config 게이트 dev runner:** `config.filter.include_tsx`(31값)가 true이고 정규화 이름 또는
     커맨드 basename이 `tsx`면 (Node, Generic); `include_ts_node`면 `ts-node`도 동일. (둘 다 기본 false → 31.)
  4. 어느 것도 아니면 `None`.
- **매처 우선순위(`match_tier` — 한 티어 안에서):** **① `name_exact`**(정규화 이름 정확 일치) → **②
  `command_binary`**(커맨드 첫 토큰 basename 정확 일치) → **③ `command_contains`**(전체 커맨드 부분
  문자열). 신뢰도 높은 순. 먼저 맞는 규칙이 이긴다.
- **`normalize_name`:** macOS가 `comm`을 16자로 잘라 `"next-server (v16…)"`이 잘려 와도, **첫 공백 토큰만**
  취해(`name.split_whitespace().next()`) 뒤 버전 블롭을 제거.
- **`command_binary`:** 커맨드 첫 토큰의 `/` 뒤 basename(빈 문자열이면 `None`).
- **제약/검증:** 규칙 테이블이 유일한 진실(아래 표). 새 런타임/FW = `Rule` 1개 + enum variant 1개(그 외
  코드 변경 0). 비서버(규칙 미일치)는 절대 표시 안 됨 → 죽일 후보 풀이 "서버"로 한정(안전성, PRB-3).
- **예외:** 입력이 비어도 패닉 없음(빈 매칭 → `None`). `Express`/`Fastify`/`Koa`/`Hapi`는 enum만 있고
  규칙 미등록이라 **현재 탐지 불가**(31 §3-2, 40-backlog).

**현재 등록 규칙(정규값):**

*FRAMEWORK_RULES (먼저 검사):*
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

*RUNTIME_RULES (나중에 검사 · framework=Generic):*
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

→ 의사결정 플로우차트는 **§8-1**.

### 2-2. display_name 추출 — `ProcessInfo::display_name` (F2) · `mod.rs`

- **입력:** `self.command`(+ fallback `self.name`).
- **출력:** 사람이 읽을 짧은 이름(목록 NAME 컬럼).
- **처리:** ① `command` 비면 `name` 반환. ② 첫 토큰의 basename = `binary`. ③ 인자에서 `node_modules/`
  포함 항목을 `modules_tool`로 따로 추출. ④ 인자를 훑어 **clean_args** 수집 — `{`·`"`·`":`(JSON/env 블롭)
  만나면 중단, `node_modules/` 건너뜀, basename이 서브커맨드(`is_subcommand`: ≤20자·영숫자/`-`/`_`·첫 글자
  알파벳)·스크립트(`.js/.ts/.mjs/.cjs/.sh`)·패키지(`@…` ≤40자)면 채택(최대 2개). ⑤ `binary`가 `node`가
  아니면 `"{binary} {args}"`; `node`면 `modules_tool` 우선(예: `next` 등), 없고 args 있으면 `"node {args}"`,
  아무것도 없으면 `name`.
- **제약:** 순수 문자열 파싱(fs 미독). JSON/환경 블롭에서 안전하게 끊어 노이즈 제거.

### 2-3. 스캔 2-pass + CPU delta — `ProcessScanner::scan` / `do_scan` (F1·F5·F10) · `scanner.rs`·`main.rs`

- **입력:** persistent `System`(sysinfo), `&Config`.
- **출력:** `Vec<ProcessInfo>` — 분류된 서버(runtime=Some) + 트리 부모(runtime=None), 이후 포트 채움.
- **CPU delta 제약(버그처럼 보이는 핵심):** sysinfo는 **두 refresh 간 차이**로 per-process CPU를 계산한다.
  `System`/`ProcessScanner`를 매 tick 재생성하면 모든 CPU가 영원히 `0.0%`. 그래서 **`System`은 TUI 루프
  내내 long-lived**. `new()`가 **2번 refresh + `MINIMUM_CPU_UPDATE_INTERVAL` sleep**으로 prime,
  `scan_blocking()`(1회성 CLI용)이 sleep 후 3번째 측정 refresh 추가.
- **처리:**
  1. `refresh_processes`(cpu·memory·cmd·cwd·environ·user 갱신).
  2. **Pass1:** 전체 프로세스 열거 → `classify(name,command,config)` → `None`이면 `continue`, `Some`이면
     `collect_process_info`로 메트릭 채우고 `runtime=Some`·`framework` 세팅, `server_pids`에 등록.
  3. **Pass2:** pass1 서버들의 `ppid` 중 **server_pids에 없는** 것(launchd·claude·shell 등)을 backfill —
     `collect_process_info`로 추가하되 `runtime=None`(트리 컨텍스트 부모, 서버 아님).
  4. **포트는 분류 후 별도로:** `do_scan`/`cmd_list`가 `NetworkInspector::connections_by_pid()` 호출 →
     각 프로세스 `pid`로 매칭해 **LISTEN 상태만** 골라 `ports`에 채움(비면 유지). 스캐너는 포트를 안 가져옴.
- **메트릭(`collect_process_info`):** cpu_percent, memory_rss(macOS phys_footprint 우선 → §2-10), memory_vms,
  status(`{:?}`), uptime, threads/open_fds(platform FFI), env_vars((k,v) 분해), user(없으면 `"unknown"`),
  ppid/command/cwd.
- **예외:** pass2에서 부모 PID가 이미 사라졌으면 그냥 건너뜀(`process(pid)`가 `None`).

→ 2-pass 플로우는 **§8-2**.

### 2-4. 트리 빌드 — `TreeBuilder` (F10) · `tree.rs`

- **입력:** flat `Vec<ProcessInfo>`. **출력:** roots `Vec<ProcessInfo>`(자식 중첩).
- **처리:** ① 전 PID를 집합으로 수집, PID로 정렬(결정적 순서). ② `ppid`가 집합에 **없으면 root, 있으면
  child**로 분할(`partition`). ③ child를 `ppid`별로 그룹핑(HashMap). ④ root부터 재귀로 자식 부착
  (`attach_children`). 
- **보조:** `flatten_with_depth`(DFS, (참조,깊이)) · `sort_recursive`(레벨별 정렬, §2-14) · `collect_pids`
  (노드+모든 자손 PID, kill_tree 입력).
- **App 표시 평탄화(`flatten_with_expand`):** `expanded_pids`에 든 노드만 자식을 펼침. 접힌 노드는 자식을
  스텁(빈 자식)으로만 둬 트리 인디케이터를 그림.
- **제약:** 빈 입력 → 빈 결과. 첫 비어있지 않은 스캔에서 `first_load`면 전 PID를 `expanded_pids`에 넣어
  전부 펼친 상태로 시작.

### 2-5. 네트워크 파싱 / 포트 매핑 — `NetworkInspector` (F5·F6) · `network.rs`

- **입력:** 시스템 명령 출력(외부 의존). **출력:** `HashMap<u32, Vec<NetworkConnection>>`(PID→연결들).
- **Unix(`parse_lsof`):** `lsof -iTCP -sTCP:LISTEN,ESTABLISHED -nP -F pcnT` 실행 후 **field-mode**(한 줄 한
  필드) 파싱 — `p`=PID, `c`=커맨드(무시), `n`=주소(name), `TST=`=상태. 새 `p`/`n`을 만나면 직전 (name,state,
  pid)로 `parse_connection`을 만들어 누적. `name`에 `->`가 있으면 로컬/원격 분리.
- **Windows(`parse_netstat`):** `netstat -ano -p TCP` 파싱 — `TCP` 시작 줄을 공백 분해(local·remote·state·
  pid), 상태 `"LISTENING"`→`"LISTEN"` 정규화.
- **`parse_addr` 폼:** `*:port`→`0.0.0.0:port`(IPv4 unspecified) · `[::1]:port`/`[::]:port`→IPv6 · 평문
  `host:port`→마지막 `:` 기준 IPv4. 파싱 실패 시 그 항목 `None`(건너뜀).
- **포트 결합:** 상위(do_scan/cmd_list)가 결과를 받아 **LISTEN만** 포트로 추려 `ProcessInfo.ports`에 채움.
- **제약/예외:** **명령 부재·비정상 종료(`status` 실패) → 빈 `HashMap` 반환(크래시 0).** 따라서 lsof/netstat
  가 없으면 포트/Net 탭이 비는 정도로 graceful degradation. (계약 상세는 32, 안정성 NFR은 10-prd.)

→ 파싱 플로우는 **§8-4**.

### 2-6. graceful 종료 + force 에스컬레이션 — `graceful_kill`/`force_kill` (F7) · `killer.rs`·`main.rs`

- **입력:** `pid`, `timeout: Duration`(= `config.graceful_duration()`, 기본 10초 — 31값).
- **출력:** `GracefulResult` { `Terminated` · `TimedOut` · `AlreadyDead` · `PermissionDenied` · `Error` }.
- **처리(`graceful_kill`):**
  1. SIGTERM 전송. (Unix: `nix signal::kill` — `ESRCH`→`AlreadyDead`, `EPERM`→`PermissionDenied`, 기타
     `Error`로 즉시 반환. Windows: `taskkill /PID` 결과 매핑.)
  2. 전송 성공이면 **200ms 간격 폴링** — 매번 `is_alive(pid)` 확인. 죽었으면 `Terminated` 반환.
  3. 누적 경과가 `timeout`에 도달할 때까지 안 죽으면 `TimedOut` 반환.
- **에스컬레이션(CLI `ntop kill <PID>` 단건 경로 — `main.rs`):** `graceful_kill` 결과가 `TimedOut`이면
  `"Graceful kill timed out. Force killing..."` 출력 후 **`force_kill`(=SIGKILL) 자동 호출**. 다른 분기는
  메시지 출력만. → 정상 요청이 안 통하면 강제로 단계가 올라간다(에스컬레이션).
- **TUI 경로(코드-진실 구분):** TUI `KillConfirm`(Enter)은 **`send_signal(pid, SIGTERM)`을 직접** 보내고
  `needs_rescan=true`만 세팅한다 — 즉 **TUI는 graceful_kill 폴링/자동 에스컬레이션을 쓰지 않는다.** 강제
  종료는 **별도 수동 모달 `ForceKillPrompt`(Enter→`force_kill`)**로 분리돼 있다. (자동 graceful→force 폴링
  에스컬레이션은 CLI 단건 경로에서만 일어난다.) 확인 게이트·2단계 정책 의미는 33, 키는 21.
- **제약:** 폴링 간격 `200ms`는 **알고리즘 상수**(Config 아님). `timeout`은 31의 `graceful_timeout`.

→ 상태도/플로우는 **§8-3**.

### 2-7. 트리 종료 — `kill_tree` (F8) · `killer.rs`

- **입력:** `pids: &[u32]`(루트+자손, `collect_pids`로 수집), `signal`(기본 SIGTERM).
- **출력:** `Vec<(u32, KillResult)>`.
- **처리:** **`pids.iter().rev()` — 역순(자식부터)** 으로 각 PID에 `send_signal`. 부모를 먼저 죽여 자식이
  고아가 되는 것을 방지(PRB-5).
- **호출 경로:** CLI `kill --tree`(확인 프롬프트 → `collect_pids` → `kill_tree(pids, kill_signal)`), TUI
  `KillTreeConfirm`(Enter → `kill_tree(pids, SIGTERM)`). **트리/all 경로는 graceful 폴링 없이 선택 시그널을
  1회씩** 보낸다(에스컬레이션은 단건 경로 전용 — §2-6).
- **검증:** `collect_pids`는 트리 DFS로 노드+모든 자손 PID를 모은다(누락 0).

### 2-8. 단건 시그널 전송 — `send_signal` (F9) · `killer.rs`

- **입력:** `pid`, `KillSignal`(SignalPicker 선택 또는 CLI `--signal`). **출력:** `KillResult` { `Success`·
  `AlreadyDead`·`PermissionDenied`·`Error` }.
- **처리:** Unix=`nix signal::kill`(`ESRCH`→AlreadyDead, `EPERM`→PermissionDenied). Windows: `Term`/`Int`→
  `taskkill /PID`(stderr "not found"→AlreadyDead, "Access"→PermissionDenied), `Kill`→`OpenProcess`+
  `TerminateProcess`(에러 5→PermissionDenied, 87→AlreadyDead).
- **가용 시그널:** `KillSignal::all()` — Unix 6종 / Windows 3종(31 §3-4). 권한/가용성 매트릭스는 33.
- **`is_alive(pid)`:** Unix=`kill(pid, None)` 성공 여부, Windows=`OpenProcess`+`WaitForSingleObject` 비신호.

### 2-9. 로그 자동 감지·tail — `LogStreamer` (F16) · `log/streamer.rs`

- **입력:** 선택 프로세스의 `cwd`(+ Linux는 `pid`). **출력:** 새로 추가된 줄들(`Vec<String>`) + 내부 버퍼.
- **LOG_PATTERNS(정규 목록):** `*.log`, `.next/server/app/**/*.log`, `.next/trace`, `logs/*.log`,
  `log/*.log`, `npm-debug.log`, `yarn-error.log`.
- **처리(`detect_and_open`):** ① `find_log_file`이 cwd 기준 각 패턴을 글롭 → 후보 수집 → **수정시각 최신
  순 정렬 → 첫 번째(가장 최근)** 선택. ② 그 파일 open, **`SeekFrom::End(0)`로 끝으로 이동**(과거 줄 무시,
  새 줄만). ③ `poll_new_lines`가 `read_line`을 EOF까지 반복, `\n`/`\r` 제거 후 버퍼 push_back +
  반환 목록에 추가, **버퍼가 `MAX_BUFFER_LINES`(1000, 31값) 초과 시 앞에서 pop_front**(메모리 상한).
- **Linux fallback(`detect_and_open_with_proc`):** cwd 글롭이 소스를 못 찾으면 `/proc/<pid>/fd/1`(stdout)을
  열어 동일하게 tail. (macOS/Windows엔 이 fallback 없음.)
- **호출:** CLI `ntop log <PID>`(200ms 폴링, 프로세스 사망 시 종료) / TUI Log 탭(선택 변경 시 재오픈).
- **예외:** 로그 파일·소스 없으면 `has_source()=false`(빈 스트리머) — 탭은 "소스 없음" 상태.

→ 플로우는 **§8-5**.

### 2-10. 정확한 메모리 보고 — `phys_footprint` (F18) · `platform.rs`·`scanner.rs`

- **처리:** `memory_rss`를 채울 때 macOS는 **`phys_footprint(pid)`(via `proc_pid_rusage`, RUSAGE_INFO_V2의
  `ri_phys_footprint`)** 를 우선 사용하고, 실패(`None`) 시 sysinfo `process.memory()`로 fallback.
- **이유:** sysinfo RSS는 압축 메모리를 빼 유휴 Node를 최대 100배 과소보고 → Activity Monitor "메모리"
  컬럼과 일치하는 `phys_footprint`를 씀(PRB-6). 非macOS는 `phys_footprint`가 `None` → 항상 sysinfo 값.
- **검증:** 동일 PID `ntop info` 값 ≈ Activity Monitor(10-prd NFR 정확성).

### 2-11. 환경변수 마스킹 점검 — Env 탭 (F17) · 동작 개요

- **동작:** Env 탭은 `ProcessInfo.env_vars`((k,v) 목록)를 표시하되, `display.mask_env_values`(기본 true,
  31값)가 켜져 있으면 민감해 보이는 키(예: PASSWORD/SECRET/TOKEN 등)의 값을 가린다.
- **소유 경계:** "어떤 키를 마스킹 대상으로 보는지(규칙)"와 동의/적용 정책은 **33-policy**가 단일 출처.
  본 문서는 "토글에 따라 켜고 끈다"는 동작만 명시. env_vars 추출은 §2-3.

### 2-12. health 판정 — `HealthStatus` / `ProcessInfo::health` (점검·CLI info) · `mod.rs`

> **이 절이 health 임계값의 정규 출처다.**

- **`from_cpu_mem(cpu, mem)`:** `cpu≥90.0 || mem≥90.0` → **Critical**; 아니면 `cpu≥80.0 || mem≥80.0` →
  **Warning**; 아니면 **Healthy**.
- **`from_process_status(status)`:** `"Zombie"` 또는 `"Dead"` → **Critical**; 그 외 → **Healthy**.
- **`health()` 결합:** 먼저 상태 기반 판정 → Critical이면 즉시 Critical 반환; 아니면 `from_cpu_mem`을 호출.
- **출력:** `HealthStatus`(Healthy/Warning/Critical, 31 §3-3) — 목록 health dot, CLI `info` health 행.
- **검증/예외:** 상태가 Zombie/Dead면 리소스와 무관하게 Critical이 이긴다.
- **코드-진실 주의:** `health()`가 `from_cpu_mem`에 넘기는 두 번째 인자는 **백분율이 아니라 `memory_rss`를
  MB로 환산한 값**(`memory_rss as f32 / 1_048_576.0`)이다. 즉 메모리 분기는 사실상 "RSS ≥ 90MB→Critical,
  ≥80MB→Warning"으로 동작한다. 함수의 계약(임계 90/80)은 위와 같으나, 실제 호출 인자가 MB라는 점은 실제
  동작 그대로 기록한다(개선 여지 = 40-backlog, 창작 아님).

### 2-13. 실시간 필터 — `App::matches_filter` (F11) · `app.rs`

- **입력:** `ProcessInfo`, `filter`(소문자화). **출력:** bool.
- **처리:** 빈 필터 → 전부 true. 아니면 대소문자 무시 **부분 문자열**을 다음 중 하나라도 만족: `name` ·
  `command` · `pid`(문자열) · `framework`(Display) · `runtime`(Some일 때 Display) · `ports`(각 포트 문자열).
- **갱신 제약:** 필터 키 입력마다 `rebuild_view()` 호출 → 다음 스캔 tick을 안 기다리고 즉시 재필터·재정렬·
  재평탄화(응답성 NFR).

### 2-14. 정렬 — `toggle_sort` / `rebuild_view` 비교자 (F12) · `app.rs`

- **`toggle_sort`(키 `s`):** `sort_column.next()`로 9컬럼 순환(31 §3-5). 같은 컬럼으로 되돌면 방향 반전,
  새 컬럼이면 오름차순 리셋. (키 `r`은 방향 반전 — 21 소유.)
- **비교자:** 컬럼별로 — Pid/Threads/Memory/Uptime는 수치, Cpu는 `partial_cmp`(NaN→Equal), Name/User/
  Status는 문자열(Name은 소문자), Port는 `ports.first()`(없으면 0). `sort_ascending`이면 그대로, 아니면
  `reverse()`. `TreeBuilder::sort_recursive`로 **레벨별** 정렬(트리 구조 유지).

### 2-15. Node-only 토글 — `rebuild_view` 필터 (F13) · `app.rs`

- **`toggle_node_only`(키 `n`):** `node_only` 반전 후 `rebuild_view`.
- **필터 처리:** `node_only`면 **`is_node()`(strictly Node)이거나 `runtime.is_none()`(트리 부모)** 인 것만
  유지. Deno/Bun은 별도 런타임이라 숨겨진다. (멀티런타임 ↔ Node 집중 전환, PRB-4.)
- **`rebuild_view` 전체 순서:** raw_processes → (node-only 필터 → 텍스트 필터) → 트리 빌드 → 레벨별 정렬
  → expanded 반영 평탄화 → selected_index 클램프 → table_state 동기화.

### 2-16. CLI 처리 골격 (F19) · `main.rs` (스키마는 32)

- **공통:** CLI 핸들러는 `do_scan`과 동일하게 `ProcessScanner`(+`scan_blocking` 1회 측정)와
  `NetworkInspector`를 재사용한다. `list`는 스캔→포트 채움→트리 빌드→평탄화→포맷 출력(table/json/csv).
  `kill`은 §2-6/2-7/2-8 경로 분기(`--all`/`--tree`/단건, `--no-confirm`·`confirm_before_kill` 게이트는 33).
  `info`/`log`/`config`는 단일 프로세스 상세·로그 tail·설정 출력.
- **소유 경계:** 서브커맨드/플래그/종료코드/출력 컬럼·JSON·CSV 스키마는 **32-api-spec** 단일 출처. 여기서는
  "어떤 처리 단계를 거치는가"만 기술.

---

## 8. 시각화 (G-g) — 알고리즘 플로우차트 (외부 렌더러 0)

### 8-1. 탐지 2단계 분류 의사결정 플로우차트 (§2-1)

```
              ┌─────────────────────────────────────────────┐
  입력 ─────▶ │ name(이름) · command(커맨드) · &Config       │
              │ normalize_name(name) = 첫 공백 토큰 (mac 16자)│
              └───────────────────┬─────────────────────────┘
                                  ▼
        ╔═════════════════ 티어1: FRAMEWORK_RULES (구체 FW 먼저) ═════════════════╗
        ║  매처 순서(한 티어 내):                                                 ║
        ║   ① name_exact 정확일치?  ──예──▶ (runtime, framework) 반환 ─────────┐  ║
        ║        │아니오                                                       │  ║
        ║   ② command_binary(첫토큰 basename) 정확일치? ──예──▶ 반환 ─────────┤  ║
        ║        │아니오                                                       │  ║
        ║   ③ command_contains 부분문자열 포함? ──예──▶ 반환 ─────────────────┤  ║
        ╚════════│아니오════════════════════════════════════════════════════════│══╝
                 ▼                                                              │
        ╔═════════════════ 티어2: RUNTIME_RULES (맨몸 런타임) ════════════════╗ │
        ║   ①name_exact → ②command_binary → ③command_contains 동일 순서       ║ │
        ║   매칭 시 (runtime, Generic) 반환 ──────────────────────────────────╫─┤
        ╚════════│아니오════════════════════════════════════════════════════════╝ │
                 ▼                                                              │
        ┌──────────── 티어3: config 게이트 dev runner ────────────┐           │
        │ include_tsx(31)=true & (이름/binary == tsx) ? ──예──▶(Node,Generic)─┤
        │ include_ts_node(31)=true & == ts-node ?       ──예──▶(Node,Generic)─┤
        └──────────────────────│아니오────────────────────────────┘           │
                               ▼                                              ▼
                        ┌─────────────┐                          ┌──────────────────────┐
                        │  None 반환  │                          │ Some((Runtime, FW))  │
                        │ = 표시 안 함 │                          │ = 서버로 수집(pass1) │
                        └─────────────┘                          └──────────────────────┘
                          (fs 미독: cwd/package.json 절대 안 읽음 — 정책 33)
```

### 8-2. 스캔 2-pass 플로우 (§2-3)

```
 persistent System (long-lived!) ── new(): refresh×2 + sleep(MIN_CPU_INTERVAL) [prime]
        │   (재생성 금지 — 안 그러면 CPU 영원히 0.0%)
        ▼
   refresh_processes()  (cpu·mem·cmd·cwd·environ·user)
        │
        ▼
 ┌──── PASS 1: 분류된 서버 수집 ───────────────────────────────────┐
 │ for each 프로세스:                                              │
 │    classify(name, command, config)  ──None──▶ continue(제외)    │
 │              │Some((rt, fw))                                    │
 │              ▼                                                  │
 │    collect_process_info() → runtime=Some(rt), framework=fw      │
 │    server_pids += pid ; results += info                         │
 └──────────────────────────────┬─────────────────────────────────┘
                                 ▼
 ┌──── PASS 2: 트리 부모 backfill ─────────────────────────────────┐
 │ 서버들의 ppid 중 server_pids에 없는 것(launchd·claude·shell):     │
 │    process(ppid) 있으면 collect_process_info() → runtime=None    │
 │    (= 트리 컨텍스트 부모, 서버 아님)  / 없으면 건너뜀            │
 └──────────────────────────────┬─────────────────────────────────┘
                                 ▼
 ┌──── 포트 채움 (분류와 분리) ─────────────────────────────────────┐
 │ NetworkInspector::connections_by_pid() → pid별 LISTEN만 추려     │
 │ ProcessInfo.ports 에 결합 (§2-5 / §8-4)                          │
 └──────────────────────────────┬─────────────────────────────────┘
                                 ▼
                   app.update_processes() → rebuild_view() (§2-15)
```

### 8-3. graceful → force 처리 상태도 + 플로우 (§2-6·§2-7)

```
 [상태도: graceful_kill(pid, timeout=graceful_timeout/31=10s)]

   (시작)
     │ SIGTERM 전송
     ├── ESRCH ───────────────▶ (AlreadyDead)
     ├── EPERM ───────────────▶ (PermissionDenied)
     ├── 기타 에러 ───────────▶ (Error)
     │ 성공
     ▼
   [폴링 루프] ── 200ms sleep ──▶ is_alive(pid)?
     │                              │죽음 ─────▶ (Terminated)
     │                              │살아있음
     │◄─────── elapsed < timeout ───┘
     │ elapsed ≥ timeout
     ▼
   (TimedOut)

 [에스컬레이션 — 경로별로 다름 (코드-진실)]

   ┌ CLI `ntop kill <PID>` 단건:  graceful_kill 결과
   │     Terminated/AlreadyDead/PermissionDenied/Error ─▶ 메시지 출력
   │     TimedOut ─▶ "Force killing..." ─▶ force_kill(SIGKILL) [자동 에스컬레이션]
   │
   ├ TUI KillConfirm(Enter):     send_signal(SIGTERM) 직접 1회 (폴링/자동 force 없음)
   │     └ 강제 종료는 별도 수동 모달 ForceKillPrompt(Enter)→force_kill
   │
   ├ CLI kill --tree / TUI KillTreeConfirm: kill_tree(pids, signal)
   │     └ pids.rev() 역순(자식부터) send_signal — 고아 방지, 폴링 없음
   │
   └ CLI kill --all: 각 서버에 send_signal(signal) 1회씩
   (확인 게이트·시그널 권한·2단계 정책 = 33 / 키 = 21)
```

### 8-4. 네트워크 파싱 플로우 (§2-5)

```
 connections_by_pid()
        │
    ┌───┴────────────────┬───────────────────────┐
   Unix                 Windows                 (명령 없음/실패)
    ▼                     ▼                          ▼
 lsof -iTCP -sTCP:        netstat -ano -p TCP     빈 HashMap 반환
 LISTEN,ESTABLISHED        │                       (크래시 0,
 -nP -F pcnT               │ "TCP " 줄 공백분해     포트/Net 탭만 빔)
    │ field-mode:          │  local·remote·state·pid
    │  p=PID c=cmd(무시)   │ state "LISTENING"→"LISTEN"
    │  n=주소 TST=상태     ▼
    │ 새 p/n 만나면        parse_netstat_addr()
    │ 직전(name,state,pid) │
    │ → parse_connection   ▼
    ▼                    NetworkConnection{local,remote,state,pid}
 parse_connection
  └ name에 "->" 있으면 local/remote 분리
  └ parse_addr():  *:port→0.0.0.0  / [::1]:port→IPv6 / host:port→IPv4 (실패→None)
        ▼
 HashMap<pid, Vec<NetworkConnection>>  ──(상위)── LISTEN만 추려 ProcessInfo.ports
```

### 8-5. 로그 스트리밍 플로우 (§2-9)

```
 detect_and_open(cwd)                         (Linux: detect_and_open_with_proc)
        │                                          │ cwd 결과 없으면
        ▼                                          ▼
 find_log_file(cwd):                          /proc/<pid>/fd/1 (stdout) 열기
   각 LOG_PATTERN 글롭(*.log, .next/…, logs/…)      │
   → 후보 수집 → 수정시각 최신순 정렬 → 첫 파일  ◄──┘ (fallback)
        │ 없으면 → 빈 스트리머(has_source=false)
        ▼
   open + SeekFrom::End(0)  (과거 줄 무시, 새 줄만 tail)
        │
        ▼  (200ms 폴링: CLI / tick: TUI Log 탭)
   poll_new_lines(): read_line EOF까지
        │  \n,\r 제거 → buffer.push_back + 반환목록
        │  buffer.len() > MAX_BUFFER_LINES(1000/31) ? → pop_front (앞에서 폐기)
        ▼
   새 줄 표시 (CLI stdout / TUI Log 탭) — 프로세스 사망 시 CLI 종료
```

---

## 자체 검증 (Self-verification)

| # | 체크 | 결과 | 어디서 충족 |
|---|---|---|---|
| **S-1** | 탐지 2단계 분류 절차 + 매처 순서 + None + normalize_name | **PASS** — §2-1 입력/처리/티어/매처/예외 + 규칙 정규표 + §8-1 플로우차트 | §2-1, §8-1 |
| **S-2** | 스캔 2-pass + persistent System CPU delta + 포트 별도 | **PASS** — §2-3 pass1/pass2/메트릭/CPU delta 제약 + §8-2 플로우 | §2-3, §8-2 |
| **S-3** | graceful_kill(SIGTERM→200ms→timeout→KILL) + GracefulResult 분기 + kill_tree 역순 | **PASS** — §2-6(폴링·에스컬레이션·TUI/CLI 경로 구분)·§2-7(역순) + §8-3 상태도 | §2-6·2-7, §8-3 |
| **S-4** | 네트워크 파싱 lsof/netstat + parse_addr 폼 + 부재 빈 결과 | **PASS** — §2-5 + §8-4 파싱 플로우 | §2-5, §8-4 |
| **S-5** | 로그 LOG_PATTERNS·최신·seek end·MAX_BUFFER_LINES·/proc fallback | **PASS** — §2-9 + §8-5 플로우 | §2-9, §8-5 |
| **S-6** | health CPU/MEM ≥90 Critical/≥80 Warning, Zombie/Dead Critical | **PASS** — §2-12(임계 정규 출처 + 결합 순서 + MB 인자 정직 표기) | §2-12 |
| **S-7** | 시각화(분류·2-pass·graceful→force·파싱·로그, 외부 렌더러 0) | **PASS** — §8-1~§8-5 순수 ASCII | §8 |
| **S-8** | 단일 출처 준수(값=31·정책=33·키=21·스키마=32 참조) | **PASS** — 본문 곳곳 "→ 31/33/21/32", 알고리즘만 정의 | 전체 |

**추가 원칙 점검:**
- **창작 금지(spec §7):** 결제/정산/DB/REST/회원 0. 모든 절차는 실제 소스(`framework.rs`·`scanner.rs`·
  `killer.rs`·`network.rs`·`streamer.rs`·`app.rs`·`main.rs`)에서 추출.
- **코드-진실 정직 표기:** ① TUI KillConfirm은 graceful 폴링이 아니라 SIGTERM 직접 전송, 자동 force는 CLI
  단건 전용(§2-6/§8-3); ② `health()`의 mem 인자가 MB라는 실제 동작(§2-12); ③ Express/Fastify/Koa/Hapi
  규칙 미등록(§2-1) — 모두 실제 코드 상태(개선은 40-backlog).
- **중복정의 0:** graceful_timeout=10·MAX_BUFFER_LINES=1000·Config 기본값은 값으로 적지 않고 "31값"으로
  인용. 폴링 200ms·MINIMUM_CPU_UPDATE_INTERVAL만 알고리즘 상수로 본 문서 소유.
- **fs 미독 원칙 유지:** §2-1·§8-1에서 분류가 cwd/package.json을 읽지 않음을 명시(정책 단일 출처는 33).
- **쉬운 설명:** 분류·매처·delta·graceful·에스컬레이션·폴링·field-mode·tail·글롭 첫 등장 풀이.

**알려진 한계(후속 보완):**
- 키 바인딩(전역/리스트/상세/필터/다이얼로그)의 정규 출처는 `21-screen-spec`, 출력 스키마는 `32-api-spec`
  이다. 본 문서는 처리 단계만 소유하고 그 둘을 참조로만 부기한다.
- 임계 상수(graceful_timeout·MAX_BUFFER_LINES·health 90/80)는 코드/`31-erd` 변경 시 그 값을 따른다.
