# 41 · QA 테스트 케이스 — 핵심 플로우 + 엣지/예외 + 테스트 모듈 매핑 (ntop)

> **이 문서는 QA 케이스의 단일 출처(single source)다.** ntop의 **핵심 플로우(UF-1~UF-6)·핵심 기능
> (F1~F21)·엣지/예외**를 (전제 / 입력 / 기대결과 / 우선순위) 형식의 테스트 케이스로 정리하고, **실제로
> 존재하는 테스트 모듈 10종**(codebase-facts §9)에 매핑한다. 커버리지 **갭**(어디가 단위 테스트로 안
> 덮였는가)도 정직하게 기록한다.
>
> **참조(재정의 0):** 기능 ID(F1~F21)·우선순위는 **`10-prd`**, 플로우(UF-1~6)는 **`13-user-flows`**,
> 문제(PRB-1~8)는 **`research-s2`(→03)**, 화면/키는 **`21-screen-spec`**, 알고리즘은 **`30-functional-spec`**,
> 구조체/enum/Config 값은 **`31-erd`**, 외부 계약은 **`32-api-spec`**, 안전·프라이버시 정책은 **`33-policy`**,
> 로드맵(미구현·버그)은 **`40-backlog`** 가 단일 출처다. 본 문서는 그 위에 **검증 케이스**만 얹는다.
>
> **전제:** ntop은 **이미 v0.2.0까지 구현된 MIT 오픈소스 도구**다(codebase-facts §0). 따라서 본 QA는
> "앞으로 검증할 기능 설계"가 아니라 **이미 존재하는 동작과 이미 존재하는 테스트**를 역설계해 케이스로
> 정리한 것이다. 테스트 모듈·함수명은 실제 `tests/*.rs`에서 그대로 가져왔다(창작 0).
>
> **용어 풀이(첫 등장):** **테스트 케이스(test case)** = "이 상황에서 이렇게 하면 이래야 한다"를 적은
> 검증 단위 / **전제(precondition)** = 케이스 시작 전 갖춰져야 할 상태 / **단위 테스트(unit test)** = 함수
> 하나의 입출력을 코드로 자동 검증 / **통합/E2E** = 빌드된 바이너리를 실제로 돌려 검증(여기선
> `assert_cmd`) / **커버리지 갭(coverage gap)** = 자동 테스트가 닿지 않아 수동/시각 검수에 맡겨진 영역 /
> **EPERM** = 권한 부족 오류(Operation not PERMitted) / **고아 프로세스** = 부모가 먼저 죽어 떠도는 자식 /
> **comm 16자 절단** = macOS가 프로세스 이름(comm)을 16자로 잘라 주는 OS 특성.

---

## Sprint Contract (self-proposed checks)

이 문서(g5의 `41-qa-testcases`)가 만족시킬 **관찰 가능한 체크**(playbook §S4-g5 관찰 바 "41=핵심 플로우별
케이스(전제/입력/기대/우선) + 엣지 + 테스트 모듈 10종 매핑 + 커버리지 맵" + spec §5(41행)·§8에서 도출).

| # | 체크 | 어떻게 충족하나 | 본문 위치 |
|---|---|---|---|
| **T-1** | **핵심 플로우별 케이스(UF-1~6)** — 전제/입력/기대결과/우선순위 | §2 UF-1~UF-6 케이스 표 | §2 |
| **T-2** | **핵심 기능별 케이스(분류·네트워크·로그·설정·health)** | §3 기능별 케이스 표 | §3 |
| **T-3** | **엣지/예외 케이스** — EPERM·lsof/netstat 부재·좀비/고아·comm 절단·npx 오분류·버퍼 상한·부분 설정 | §4 엣지/예외 표 | §4 |
| **T-4** | **★ 실제 테스트 모듈 10종 매핑** — framework/scanner/tree/killer/network/log/cli/config/filter/types | §5-1 모듈 매핑 표(함수명·커버 F#/UF#) | §5-1 |
| **T-5** | **커버리지 갭 명시** — TUI 렌더·실제 종료 동작·platform FFI·env 마스킹 등 미테스트 영역 | §5-2 갭 표 + §6-3 | §5-2·§6-3 |
| **T-6** | **시각화(G-g)** — 테스트 커버리지 맵(마인드맵: 기능군→테스트모듈) + 케이스 우선순위 분포 (외부 렌더러 0) | §6-1 커버리지 맵 + §6-2 우선순위 분포 | §6 |
| **T-7** | **단일 출처 준수** — 기능=10·플로우=13·키=21·값=31·정책=33·로드맵=40 참조 | 본문 곳곳 참조 표기 | 전체 |

---

## 0. 표기 규약

```
 우선순위 (= 기능 우선순위 10-prd × 리스크):
   ● 높음 : 가치 루프 척추(P0) 또는 파괴적/안전 관련(kill·권한·오분류) — 실패 시 치명
   ◐ 중간 : 코어 보조(P1) — 실패 시 사용성 저하
   ○ 낮음 : 편의/표시(P2) 또는 보조 표시 — 실패 시 경미

 테스트 종류:
   [U] 단위(unit, #[test])   [E] E2E(assert_cmd, cli_test)   [M] 수동/시각/통합(자동 테스트 없음 = 갭)

 모듈 표기: framework=framework_test, scanner=scanner_test, tree=tree_test, killer=killer_test,
            network=network_test, log=log_test, cli=cli_test, config=config_test, filter=filter_test,
            types=types_test  (codebase-facts §9, 실제 tests/*.rs)
```

---

## 1. 이 문서가 답하는 것

```
 ┌── "핵심 동선이 동작하나?"       ──▶ §2 UF-1~UF-6 플로우 케이스
 ├── "각 기능이 규칙대로 동작하나?" ──▶ §3 분류·네트워크·로그·설정·health 케이스
 ├── "비정상 상황에서 안전한가?"   ──▶ §4 엣지/예외(EPERM·명령부재·좀비·오분류·상한)
 ├── "실제 테스트가 무엇을 덮나?"   ──▶ §5-1 테스트 모듈 10종 매핑
 ├── "무엇이 안 덮였나(갭)?"        ──▶ §5-2 커버리지 갭 (TUI 렌더·실제 종료·FFI 등)
 └── "전체를 한눈에?"               ──▶ §6-1 커버리지 맵(마인드맵)
```

---

## 2. 핵심 플로우별 테스트 케이스 (UF-1~UF-6)

> 플로우 정의·단계의 단일 출처는 `13-user-flows`. 각 케이스의 `모듈` 열은 그 케이스를 실제로 덮는
> 테스트 모듈(또는 `[M] 갭`).

### 2-1. UF-1 — 기동 → 스캔(2-pass) → 선택 → 상세 4탭 점검 → 종료

| TC | 전제 | 입력 | 기대결과 | 우선 | 종류·모듈 |
|---|---|---|---|---|---|
| **TC-1.1** | 바이너리 빌드됨 | `ntop`(서브커맨드 없음) | TUI 모드 진입(서브커맨드 없으면 run_tui) | ● | [E] cli `test_default_launches_tui` |
| **TC-1.2** | 서버 프로세스 존재 | 스캔 1회 | `Vec<ProcessInfo>` 반환, 분류된 서버는 `runtime=Some` | ● | [U] scanner `test_scanner_returns_vec` |
| **TC-1.3** | 동일 환경 2회 스캔 | 연속 스캔 | 분류 결과 일관(같은 프로세스 같은 분류) | ◐ | [U] scanner `test_scanner_runtime_classification_is_consistent` |
| **TC-1.4** | 서버 0건 | 스캔 결과 빈 목록 | V-EMPTY로 수렴(스피너 `⠹ Scanning…`), 크래시 0 | ◐ | [M] 갭(TUI 렌더 미테스트) |
| **TC-1.5** | Next.js 서버 선택 | 상세 Info 탭 열기 | `Framework=Next.js`, `Version=-`(framework_version 미채움, 40 R7) | ● | [U] types `test_framework_kind_display` + [M] 탭 렌더 |
| **TC-1.6** | RSS 값 보유 프로세스 | Info 탭 메모리 표시 | `memory_display()`가 `128.0 MB`/`1.5 GB` 형식 | ◐ | [U] types `test_process_info_memory_display` |
| **TC-1.7** | uptime 보유 | Info 탭 uptime | `uptime_display()`가 `1h 2m 5s` 형식 | ○ | [U] types `test_process_info_uptime_display` |
| **TC-1.8** | node_modules 툴/서브커맨드/스크립트/JSON블롭 커맨드 | display_name 추출 | 사람이 읽을 이름(툴명·서브커맨드 채택, JSON 블롭에서 중단) | ● | [U] types `test_display_name_*`(8케이스) |

### 2-2. UF-2 — 필터(`/`) / 정렬(`s`,`r`) / Node-only(`n`)

| TC | 전제 | 입력 | 기대결과 | 우선 | 종류·모듈 |
|---|---|---|---|---|---|
| **TC-2.1** | 목록 다수 | 빈 필터 | 전부 매칭(필터 없음 = 전체) | ◐ | [U] filter `test_matches_filter_empty_matches_all` |
| **TC-2.2** | 이름에 대문자 혼재 | 소문자 부분문자열 | 대소문자 무시 매칭 | ● | [U] filter `test_matches_filter_name_case_insensitive` |
| **TC-2.3** | 커맨드라인 보유 | 커맨드 부분문자열 | 커맨드 매칭(대소문자 무시) | ◐ | [U] filter `test_matches_filter_command_case_insensitive` |
| **TC-2.4** | 프레임워크 분류됨 | "next" 등 | 프레임워크명 매칭 | ◐ | [U] filter `test_matches_filter_framework_case_insensitive` |
| **TC-2.5** | PID 알려짐 | PID 일부 문자열 | PID 부분일치 매칭 | ○ | [U] filter `test_matches_filter_pid_substring` |
| **TC-2.6** | 포트 보유 | 포트 일부 문자열 | 포트 부분일치 매칭 | ◐ | [U] filter `test_matches_filter_ports_substring` |
| **TC-2.7** | runtime 분류됨 | "node" 등 | runtime Display 매칭 | ○ | [U] filter `test_matches_filter_runtime` |
| **TC-2.8** | 매칭 없는 검색어 | 무관 문자열 | 빈 목록(매칭 0) | ◐ | [U] filter `test_matches_filter_no_match` |
| **TC-2.9** | Node+Deno+Bun 혼재 | `n`(Node-only) | Node 서버만 + 트리 부모(runtime None) 유지, Deno/Bun 숨김 | ● | [U] filter `test_node_only_toggle_hides_other_runtimes` |
| **TC-2.10** | 필터 모드 활성 | `Enter` vs `Esc` | Enter=필터 유지·Esc=필터 비움(분기) | ◐ | [M] 갭(ui.rs 키 핸들 미테스트) |
| **TC-2.11** | 목록 정렬 | `s` 컬럼 순환·`r` 반전 | 9컬럼 순환·방향 반전, 트리 레벨별 정렬 유지 | ◐ | [M] 갭(정렬 비교자 단위 미테스트) |

### 2-3. UF-3 — kill 단건 (`x` → KillConfirm → SIGTERM → graceful → force)

> ★ 가장 위험한 플로우. **시그널 메타데이터는 killer_test가 덮지만, 실제 종료 동작(graceful_kill 폴링·
> force 에스컬레이션·EPERM)은 단위 테스트가 없다(§5-2 갭).** CLI 디스패치는 cli_test가 덮는다.

| TC | 전제 | 입력 | 기대결과 | 우선 | 종류·모듈 |
|---|---|---|---|---|---|
| **TC-3.1** | — | KillSignal 이름 조회 | `Term→SIGTERM`·`Kill→SIGKILL` 등 정확 | ◐ | [U] killer `test_kill_signal_name` |
| **TC-3.2** | — | 각 시그널 설명 | 설명 문자열 비어있지 않음 | ○ | [U] killer `test_kill_signal_description_non_empty` |
| **TC-3.3** | OS별 | `KillSignal::all()` | Unix 6종/Windows 3종, 중복 없음 | ◐ | [U] killer `test_kill_signal_all_no_duplicates` |
| **TC-3.4** | CLI `--signal` | `"SIGTERM"` 파싱 | SIG 접두어 포함 파싱 성공 | ◐ | [U] killer `test_kill_signal_from_str_with_sig_prefix` |
| **TC-3.5** | CLI `--signal` | `"TERM"` 파싱 | 접두어 없이도 파싱 성공 | ○ | [U] killer `test_kill_signal_from_str_without_prefix` |
| **TC-3.6** | CLI `--signal` | 대소문자 혼재 | 대소문자 무시 파싱 | ○ | [U] killer `test_kill_signal_from_str_case_insensitive` |
| **TC-3.7** | CLI `--signal` | 미지의 시그널명 | `None` 반환(→ SIGTERM 폴백, 32 계약) | ◐ | [U] killer `test_kill_signal_from_str_unknown_returns_none` |
| **TC-3.8** | SIGTERM 무시 프로세스 존재 | `graceful_kill(pid, 10s)` | 200ms 폴링 후 `TimedOut`, CLI 단건은 force_kill 자동 에스컬레이션 | ● | [M] 갭(실제 graceful_kill 미테스트) |
| **TC-3.9** | 종료 가능 PID | `ntop kill <PID>` | kill 디스패치(확인 게이트 경유) | ● | [E] cli `test_kill_command` |
| **TC-3.10** | — | `ntop kill <PID> --no-confirm` | 확인 프롬프트 생략하고 진행 | ● | [E] cli `test_no_confirm_flag` |
| **TC-3.11** | 다른 사용자 소유 프로세스 | kill 시도 | `KillResult::PermissionDenied`(EPERM), 크래시 0 | ● | [M] 갭(EPERM 경로 미테스트) |

### 2-4. UF-4 — kill 트리 (`K`, 자식부터 역순)

| TC | 전제 | 입력 | 기대결과 | 우선 | 종류·모듈 |
|---|---|---|---|---|---|
| **TC-4.1** | 부모-자식 flat 목록 | `TreeBuilder::build` | 부모-자식 트리 구성(중첩) | ● | [U] tree `test_build_tree_simple` |
| **TC-4.2** | 빈 입력 | build | 빈 결과(패닉 0) | ○ | [U] tree `test_build_tree_no_processes` |
| **TC-4.3** | ppid 미존재만 | build | 전부 root | ◐ | [U] tree `test_build_tree_all_roots` |
| **TC-4.4** | 트리 보유 | `flatten_with_depth` | (참조, 깊이) DFS 평탄화 | ◐ | [U] tree `test_flatten_tree` |
| **TC-4.5** | 트리 노드 | `collect_pids` | 노드+모든 자손 PID 수집(kill_tree 입력) | ● | [U] tree `test_collect_subtree_pids` |
| **TC-4.6** | 트리 보유 PID | `ntop kill <PID> --tree` | 트리 종료 디스패치 | ● | [E] cli `test_kill_with_tree` |
| **TC-4.7** | 부모+자식 생존 | `kill_tree(pids)` | **자식부터 역순(`.rev()`)** 종료 → 고아 0 | ● | [M] 갭(역순 종료 동작 미테스트, collect_pids만 덮임) |

### 2-5. UF-5 — SignalPicker (`S`)로 임의 시그널

| TC | 전제 | 입력 | 기대결과 | 우선 | 종류·모듈 |
|---|---|---|---|---|---|
| **TC-5.1** | 종료 가능 PID | `ntop kill <PID> --signal HUP` | 지정 시그널 전송 디스패치 | ◐ | [E] cli `test_kill_with_signal` |
| **TC-5.2** | OS별 | SignalPicker 목록 | Unix 6/Win 3 노출(HUP/USR1/USR2는 Unix만) | ◐ | [U] killer `all()` 계열(목록) |
| **TC-5.3** | TUI SignalPicker 열림 | `Up/Down`→`Enter` | 선택 시그널 전송·`Esc` 취소 | ◐ | [M] 갭(signal_picker.rs 렌더/키 미테스트) |

### 2-6. UF-6 — CLI 자동화 (`list --json` → `kill --tree`)

| TC | 전제 | 입력 | 기대결과 | 우선 | 종류·모듈 |
|---|---|---|---|---|---|
| **TC-6.1** | 서버 존재 | `ntop list` | table 출력(7컬럼: PID/NAME/FRAMEWORK/PORT/CPU/MEM/UPTIME) | ● | [E] cli `test_list_command` |
| **TC-6.2** | — | `ntop list --json` | 모든 필드 + depth JSON | ● | [E] cli `test_list_json` |
| **TC-6.3** | — | `ntop list --format csv` | CSV 9컬럼(PID/PPID/NAME/FRAMEWORK/PORTS/CPU/MEMORY/UPTIME/STATUS) | ◐ | [E] cli `test_list_format_csv` |
| **TC-6.4** | 유효 PID | `ntop info <PID>` | 상세 출력(런타임·프레임워크·포트·메트릭·env·연결) | ◐ | [E] cli `test_info_command` |
| **TC-6.5** | 유효 PID | `ntop log <PID>` | 로그 소스 감지·tail 시작 | ◐ | [E] cli `test_log_command` |
| **TC-6.6** | 서버 다수 | `ntop kill --all` | 전체 서버 열거 후 일괄 kill 디스패치(확인 경유) | ◐ | [E] cli `test_kill_all` |

---

## 3. 핵심 기능별 테스트 케이스 (분류·네트워크·로그·설정·health)

### 3-1. 분류 (F1·F3·F4) — `framework_test`(24케이스)

> ntop의 핵심 추상화. **2단계 분류 + fs 미독 오분류 회피 + macOS comm 절단 처리**가 여기서 검증된다.

| TC | 전제 | 입력 | 기대결과 | 우선 | 종류·모듈 |
|---|---|---|---|---|---|
| **TC-7.1** | — | `next-server` 이름 | Next.js 분류(name_exact) | ● | [U] framework `test_detect_nextjs_by_process_name` |
| **TC-7.2** | — | `nuxt`/`nuxi` 이름 | Nuxt.js 분류 | ◐ | [U] framework `test_detect_nuxt_by_process_name` |
| **TC-7.3** | — | 커맨드에 `node_modules/.bin/next` 등 | 커맨드 기반 분류(command_contains) | ● | [U] framework `test_detect_framework_by_command` |
| **TC-7.4** | python 이름 + fastapi 커맨드 | classify | **구체 FW가 맨몸 런타임을 이김**(FastAPI > python) | ● | [U] framework `test_classify_fastapi_beats_python_name`·`test_detect_combined_priority` |
| **TC-7.5** | 맨몸 `node` | classify | (Node, Generic) | ◐ | [U] framework `test_classify_node_generic`·`test_detect_fallback_to_generic` |
| **TC-7.6** | ★ Next.js cwd 안의 npx MCP 서버 | classify | **Generic 유지**(fs 미독 → cwd로 오분류 안 함, PRB-3) | ● | [U] framework `test_detect_npx_mcp_server_is_generic_even_in_nextjs_cwd`·`test_classify_npx_mcp_in_nextjs_cwd_is_node_generic` |
| **TC-7.7** | ★ macOS comm 16자 절단된 이름 | classify | `normalize_name`(첫 공백 토큰)으로 정상 분류 | ● | [U] framework `test_detect_nextjs_from_truncated_macos_comm` |
| **TC-7.8** | 이름=`node`, 커맨드에 next | classify | command_binary로 Next.js 분류 | ◐ | [U] framework `test_detect_nextjs_from_command_binary_when_name_is_node` |
| **TC-7.9** | 비서버 프로세스 | classify | `None`(목록·집계에서 제외 = 미표시) | ● | [U] framework `test_classify_non_server_is_none` |
| **TC-7.10** | `tsx` dev runner | classify(include_tsx=false/true) | config 게이트(false면 None, true면 Node) | ◐ | [U] framework `test_classify_tsx_is_config_gated` |
| **TC-7.11** | Django/Flask/Rails/Spring/Laravel/.NET/Deno/Bun | classify | 각 규칙대로 분류(Flask=`-m flask`·Rails=`bin/rails` 부분일치 포함) | ◐ | [U] framework `test_classify_django`·`_flask_via_module_flag`·`_rails_via_command_substring`·`_java_generic_and_spring`·`_php_and_laravel`·`_dotnet_deno_bun` |

### 3-2. 네트워크 파싱 (F5·F6) — `network_test`(13케이스)

| TC | 전제 | 입력 | 기대결과 | 우선 | 종류·모듈 |
|---|---|---|---|---|---|
| **TC-8.1** | — | `*:3000` | `0.0.0.0:3000`(IPv4 unspecified) | ◐ | [U] network `test_parse_addr_wildcard`·`_ipv4_unspecified` |
| **TC-8.2** | — | `127.0.0.1:3000` | IPv4 loopback 파싱 | ○ | [U] network `test_parse_addr_ipv4_loopback` |
| **TC-8.3** | — | `[::1]:3000`/`[::]:3000` | IPv6 loopback/unspecified 파싱 | ◐ | [U] network `test_parse_addr_ipv6_loopback`·`_ipv6_unspecified` |
| **TC-8.4** | ★ 견고성 | 잘못된 주소/포트 | `None` 반환(해당 항목 건너뜀, 크래시 0) | ● | [U] network `test_parse_addr_invalid_returns_none`·`_invalid_port_returns_none` |
| **TC-8.5** | LISTEN 라인 | parse_connection | `state=LISTEN`, 로컬 주소 파싱 | ● | [U] network `test_parse_connection_listen`·`_wildcard_listen`·`_ipv6_listen` |
| **TC-8.6** | `->` 포함 라인 | parse_connection | 로컬/원격 분리, `ESTABLISHED` | ◐ | [U] network `test_parse_connection_established_with_remote` |
| **TC-8.7** | 상태 없는 라인 | parse_connection | 상태 기본값 `unknown` | ○ | [U] network `test_parse_connection_none_state_defaults_to_unknown` |
| **TC-8.8** | ★ lsof/netstat 부재·실패 | `connections_by_pid()` | 빈 `HashMap` 반환(크래시 0, 포트/Net 탭만 빔) | ● | [M] 갭(부재 시나리오 직접 미테스트, 파싱만 덮임) |

### 3-3. 로그 스트리밍 (F16) — `log_test`(13케이스)

| TC | 전제 | 입력 | 기대결과 | 우선 | 종류·모듈 |
|---|---|---|---|---|---|
| **TC-9.1** | 새 스트리머 | 생성 직후 | `has_source()=false` | ○ | [U] log `test_new_streamer_has_no_source` |
| **TC-9.2** | 빈 디렉터리 | find_log_file | `None` | ◐ | [U] log `test_find_log_file_returns_none_for_empty_dir` |
| **TC-9.3** | `.log`/`logs/*.log` 존재 | find_log_file | 해당 파일 발견 | ◐ | [U] log `test_find_log_file_finds_dot_log_in_dir`·`_finds_log_in_logs_subdir` |
| **TC-9.4** | 비-로그 파일 혼재 | find_log_file | 비-로그 무시(패턴만) | ◐ | [U] log `test_find_log_file_ignores_non_log_files` |
| **TC-9.5** | ★ 여러 로그 파일 | find_log_file | **가장 최근 수정 파일** 선택 | ● | [U] log `test_find_log_file_picks_most_recently_modified` |
| **TC-9.6** | 로그 파일 존재 | detect_and_open | 파일 열고 끝으로 seek(새 줄만) | ◐ | [U] log `test_detect_and_open_opens_log_file`·`_no_log_file` |
| **TC-9.7** | tail 중 새 줄 추가 | poll_new_lines | 추가분만 반환, 없으면 빈 | ◐ | [U] log `test_poll_new_lines_reads_appended_content`·`_empty_when_no_new_content`·`_no_file_returns_empty` |
| **TC-9.8** | ★ 1000줄 초과 | 다회 poll | 버퍼 `MAX_BUFFER_LINES=1000` 상한(앞에서 폐기) | ● | [U] log `test_buffer_capped_at_1000_lines`·`_accumulates_across_multiple_polls` |

### 3-4. 설정 (F21) — `config_test`(5케이스)

| TC | 전제 | 입력 | 기대결과 | 우선 | 종류·모듈 |
|---|---|---|---|---|---|
| **TC-10.1** | 설정 파일 없음 | `Config::default` | 기본값(refresh=3·SIGTERM·graceful=10·confirm=true·mask=true 등, 31 정규값) | ◐ | [U] config `test_default_config` |
| **TC-10.2** | 완전한 TOML | 파싱 | 모든 섹션 정상 로드 | ◐ | [U] config `test_parse_toml_config` |
| **TC-10.3** | ★ 일부 키만 있는 TOML | 파싱 | 누락 키는 기본값(serde default — 부분/부재 안전) | ● | [U] config `test_partial_toml_uses_defaults` |
| **TC-10.4** | refresh/graceful 설정 | duration 변환 | `refresh_duration()`/`graceful_duration()` 정확 | ○ | [U] config `test_config_refresh_duration`·`_graceful_duration` |

### 3-5. health / 타입 (F1·F2·F18) — `types_test`(23케이스)

| TC | 전제 | 입력 | 기대결과 | 우선 | 종류·모듈 |
|---|---|---|---|---|---|
| **TC-11.1** | CPU/MEM 값 | `from_cpu_mem` | ≥90→Critical·≥80→Warning·else Healthy | ◐ | [U] types `test_health_status_from_metrics`·`_high_cpu_is_warning`·`_very_high_cpu_is_critical` |
| **TC-11.2** | ★ status="Zombie" + 낮은 리소스 | `health()` | **Zombie/Dead가 리소스 무관 Critical 우선** | ● | [U] types `test_health_status_zombie`·`test_health_zombie_status_overrides_healthy_metrics` |
| **TC-11.3** | 정상 상태 + 낮은 리소스 | `health()` | Healthy | ◐ | [U] types `test_health_running_with_low_resources_is_healthy` |
| **TC-11.4** | runtime/framework 보유 | Display/직렬화 | Node/Python…·Next.js… 표기·serde 직렬화 정확 | ◐ | [U] types `test_runtime_display`·`_serialization`·`test_framework_kind_display`·`_serialization`·`test_new_framework_kind_display` |
| **TC-11.5** | 기본 ProcessInfo | default | 필드 기본값 정상 | ○ | [U] types `test_process_info_default` |
| **TC-11.6** | ★ health 메모리 인자(코드 실상) | `health()` 호출 | 현재 테스트는 **CPU 기준만** 검증 — 메모리 인자가 `%`가 아니라 **RSS의 MB 환산값**으로 넘어가는 버그(40 R6)는 **검출 못 함** | ● | [M] 갭/주의(메모리 분기 단위 미검증) |

---

## 4. 엣지 / 예외 케이스 (안전·견고성)

> playbook이 명시한 핵심 엣지: **EPERM 권한거부·lsof/netstat 부재→빈결과·좀비/고아·macOS comm 16자
> 절단·npx 오분류 회피.** 위 케이스와 교차 참조하고, 미테스트(갭)는 명확히 표기한다.

| TC | 엣지/예외 | 전제 | 입력 | 기대결과 | 우선 | 종류·모듈 |
|---|---|---|---|---|---|---|
| **TC-E1** | **EPERM 권한거부** | 다른 사용자/루트 소유 프로세스 | kill/signal 전송 | `PermissionDenied` 노출, 다른 PID 진행, 크래시 0 | ● | [M] 갭(↔TC-3.11) |
| **TC-E2** | **lsof/netstat 부재** | 시스템에 명령 없음 | 포트 조회 | 빈 `HashMap`→포트/Net 탭 빔, "No active network connections", 크래시 0 | ● | [M] 갭(↔TC-8.8) |
| **TC-E3** | **좀비/죽은 프로세스** | status=Zombie/Dead | health 판정 | 리소스 무관 Critical(상태 우선) | ● | [U] types(↔TC-11.2) |
| **TC-E4** | **고아 방지(역순 종료)** | 부모+자식 트리 | kill_tree | 자식부터 역순 → 고아 0 (실제 동작 미테스트) | ● | [M] 갭(↔TC-4.7) / collect_pids는 [U] tree |
| **TC-E5** | **macOS comm 16자 절단** | 이름이 16자로 잘림 | classify | normalize_name으로 정상 분류 | ● | [U] framework(↔TC-7.7) |
| **TC-E6** | **npx/전역 도구 오분류 회피** | npx MCP가 프레임워크 cwd에서 실행 | classify | fs 미독 → Generic 유지(오분류 0) | ● | [U] framework(↔TC-7.6) |
| **TC-E7** | **잘못된 주소/포트 입력** | 파싱 불가 주소 | parse_addr | `None`(항목 건너뜀, 크래시 0) | ● | [U] network(↔TC-8.4) |
| **TC-E8** | **부분/깨진 설정 파일** | 일부 키만/파싱 실패 | Config 로드 | 누락은 기본값(serde default), 크래시 0 | ● | [U] config(↔TC-10.3) |
| **TC-E9** | **로그 소스 없음** | cwd에 로그 파일 0 | log 탭/명령 | "No log source detected"(Linux는 `/proc/<pid>/fd/1` fallback 시도) | ◐ | [U] log `test_detect_and_open_no_log_file` + [M] /proc fallback 갭 |
| **TC-E10** | **로그 버퍼 폭주** | 1000줄 초과 출력 | poll 반복 | 앞에서 폐기(메모리 상한 1000) | ● | [U] log(↔TC-9.8) |
| **TC-E11** | **미지의 시그널명** | CLI `--signal FOO` | from_str | `None`→SIGTERM 폴백(크래시 0) | ◐ | [U] killer(↔TC-3.7) |
| **TC-E12** | **빈 입력/0건 스캔** | 프로세스 0 | 스캔·트리 빌드 | 빈 결과(패닉 0), V-EMPTY 수렴 | ◐ | [U] tree `test_build_tree_no_processes` + [M] TUI 갭 |
| **TC-E13** | **터미널 폭/높이 부족** | 좁은 터미널 | 렌더 | 컬럼 축소/탭 콘텐츠 생략(크래시 0) | ○ | [M] 갭(렌더 미테스트, STEP 7 수동) |
| **TC-E14** | **CLI info env 평문 노출(코드 실상)** | 민감 env 보유 | `ntop info <PID>` | 현재 **평문 출력**(TUI만 마스킹) — 프라이버시 갭(40 R8) | ◐ | [M] 갭/주의(미마스킹 미검증) |

---

## 5. ★ 실제 테스트 모듈 10종 매핑 (codebase-facts §9)

### 5-1. 모듈 → 기능/플로우 매핑 표

> `tests/*.rs` 실제 파일. **합계 115개 테스트**(아래 수치는 `#[test]`/`#[tokio::test]` 카운트).

| 모듈 | 테스트 수 | 종류 | 커버 기능(10) | 커버 플로우(13) | 무엇을 검증하나 | 대표 케이스 |
|---|---|---|---|---|---|---|
| **framework_test** | 24 | [U] | F1·F3·F4 | UF-1(식별) | 2단계 분류·매처 우선순위·fs 미독 오분류 회피·macOS comm 절단·config 게이트·8런타임/다수 FW | TC-7.* / TC-E5·E6 |
| **scanner_test** | 2 | [U] | F1·F10 | UF-1(스캔) | 스캔이 Vec 반환·분류 일관성(시스템 의존, 얇음) | TC-1.2·1.3 |
| **tree_test** | 8 | [U] | F8·F10 | UF-1·UF-4 | 트리 빌드(root/child 분할)·평탄화·레벨별 정렬·collect_pids | TC-4.1~4.5 |
| **killer_test** | 7 | [U] | F9(·F7 일부) | UF-3·UF-5 | **KillSignal 메타데이터**(name/desc/all 중복0/from_str 4종) — ※실제 종료 동작은 안 덮음 | TC-3.1~3.7 |
| **network_test** | 13 | [U] | F5·F6 | UF-1(Net)·UF-6 | parse_addr 3폼·parse_connection 상태/원격·잘못된 입력 None | TC-8.1~8.7 / TC-E7 |
| **log_test** | 13 | [U] | F16 | UF-1(Log) | LOG_PATTERNS glob·최신 파일·seek end·poll·버퍼 1000 상한 | TC-9.* / TC-E10 |
| **cli_test** | 11 | [E] | F19·F7·F8 | UF-1·UF-3·UF-4·UF-6 | 빌드 바이너리 디스패치(list/json/csv·kill/tree/signal/all·info·log·no-confirm·기본=TUI) | TC-1.1 / TC-3.9·3.10 / TC-6.* |
| **config_test** | 5 | [U] | F21 | (설정) | 기본값·TOML 파싱·부분 TOML 기본값 폴백·duration 변환 | TC-10.* / TC-E8 |
| **filter_test** | 9 | [U] | F11·F12·F13 | UF-2 | matches_filter 6필드 대소문자 무시·매칭 0·Node-only 토글 | TC-2.1~2.9 |
| **types_test** | 23 | [U] | F1·F2·F18 | UF-1 | FrameworkKind/Runtime Display·직렬화·HealthStatus(metrics/zombie override)·display_name 8케이스·memory/uptime 표시 | TC-1.5~1.8 / TC-11.* |

```
 모듈 테스트 수 분포 (총 115)
 framework_test ████████████████████████  24  ← 핵심 추상화에 가장 두텁게
 types_test     ███████████████████████   23  ← 표시·health·display_name
 network_test   █████████████             13
 log_test       █████████████             13
 cli_test       ███████████               11  ← 유일한 E2E(assert_cmd)
 filter_test    █████████                  9
 tree_test      ████████                   8
 killer_test    ███████                    7  ← 단, 시그널 메타데이터뿐(아래 갭)
 config_test    █████                      5
 scanner_test   ██                         2  ← 시스템 의존·얇음(아래 갭)
```

### 5-2. ★ 커버리지 갭 (자동 테스트가 닿지 않는 영역 — 정직 기록)

> "어디가 안 덮였나"를 숨기지 않는다. 아래는 **단위 테스트가 없어 수동/시각/통합 검수(STEP 7·릴리스
> 검증)에 맡겨진 영역**이다. 로드맵(40)의 일부 항목과 겹친다.

| 갭 영역 | 무엇이 안 덮였나 | 왜(원인) | 어떻게 보완 | 관련 |
|---|---|---|---|---|
| **TUI 렌더링** | `ui.rs`·`widgets/*`(process_list·detail_panel·각 탭·다이얼로그·status_bar·empty_state) 렌더·레이아웃 | ratatui 화면 출력은 단위화 어려움 | STEP 7 수동 시각 검수(20·21) | TC-1.4·2.10·2.11·5.3·E13 |
| **TUI 키 핸들링** | `handle_key`/`handle_*_key` 모드 분기(필터 Enter/Esc·정렬·다이얼로그 전이) | 상태 머신이 App+터미널에 결합 | 수동 시나리오 검수 | TC-2.10·2.11 |
| **실제 종료 동작** | `graceful_kill` 폴링·`force_kill` 에스컬레이션·`send_signal` 결과·`kill_tree` **역순**·EPERM | 실제 프로세스 생사·권한 필요(부수효과 큼) | 통합 테스트(가짜 프로세스)·수동 | TC-3.8·3.11·4.7·E1·E4 |
| **platform FFI** | `phys_footprint`·thread/fd count(F18 macOS 메모리 정확도) | OS별 FFI·Activity Monitor 비교 | macOS 릴리스 시 `info` vs Activity Monitor 육안 비교 | TC-1.6(표시만 덮임) |
| **env 마스킹 로직** | `env_tab.rs` SENSITIVE_PATTERNS 마스킹·과마스킹·CLI 미마스킹(F17) | 마스킹은 위젯 내부, 단위 테스트 없음 | 마스킹 단위 테스트 추가(40 R8·R10) | TC-E14 |
| **명령 부재 시나리오** | lsof/netstat 실제 부재 시 빈 결과(파싱만 덮임) | 환경 조작 필요 | 부재 환경 통합 테스트 | TC-8.8·E2 |
| **정렬 비교자** | `SortColumn` 9컬럼 비교자·트리 레벨별 정렬 | App `rebuild_view` 내부 | 비교자 단위 테스트 추가 | TC-2.11 |
| **health 메모리 분기** | `from_cpu_mem` 2번째 인자가 MB로 넘어가는 코드 실상(40 R6 버그) | 테스트가 CPU 기준만 확인 | 메모리 분기 단위 테스트(버그 회귀 방지) | TC-11.6 |

---

## 6. 시각화 (G-g)

### 6-1. 테스트 커버리지 맵 (마인드맵: 기능군 → 테스트 모듈)

> ✅ = 단위/E2E로 덮임 · ⚠ = 부분(메타데이터/얇음) · ✗갭 = 자동 테스트 없음(§5-2).

```
                              ┌──────────────────────────────┐
                              │   ntop 테스트 커버리지 맵     │
                              │   (10 모듈 / 115 테스트)      │
                              └───────────────┬──────────────┘
      ┌──────────────┬──────────────┬─────────┼─────────┬──────────────┬──────────────┐
      ▼              ▼              ▼          ▼         ▼              ▼              ▼
 ┌─────────┐  ┌─────────┐  ┌──────────┐ ┌─────────┐ ┌─────────┐ ┌────────────┐ ┌──────────┐
 │A 식별   │  │B 포트   │  │C 안전종료 │ │D 뷰탐색 │ │E 점검   │ │F 인터페이스│ │횡단/공통 │
 └────┬────┘  └────┬────┘  └────┬─────┘ └────┬────┘ └────┬────┘ └─────┬──────┘ └────┬─────┘
      │            │            │            │           │            │             │
 ✅framework(24)  ✅network(13) ⚠killer(7)  ✅filter(9) ✅log(13)   ✅cli(11,E2E)  ✅config(5)
   F1·F3·F4         F5·F6        F9 메타만    F11·12·13   F16          F19·7·8 디스패치 F21
 ✅scanner(2)                    ✅tree(8)               ✅types(23)                  ✅types(23)
   F1·F10 (얇음)                  F8·F10       (Node-only) F1·F2·F18(health/disp)     (Display/직렬화)
      │            │            │            │           │            │
 ✗갭: 없음        ✗갭:명령부재  ✗갭:실제종료 ✗갭:정렬    ✗갭:env마스킹 ✗갭:TUI키핸들  ✗갭:—
   (잘 덮임)         시나리오     ·역순·EPERM   비교자·    ·CLI미마스킹  (E2E 디스패치는
                    (TC-8.8)      (TC-3.8/4.7) Node토글UI  ·platform     덮임, 키 분기는 갭)
                                              (TUI)        FFI(F18)
 ────────────────────────────────────────────────────────────────────────────────────────────
 ✗ 전역 갭(모든 군 공통): TUI 렌더링(ui.rs·widgets/*) — 단위 테스트 0, STEP 7 수동 시각 검수 영역
```

### 6-2. 케이스 우선순위 분포 + 케이스↔모듈 요약

```
 우선순위 분포 (본 문서 정의 케이스 기준)
   ● 높음  ████████████████████████  안전·식별·견고성(분류·kill·EPERM·고아·comm절단·버퍼상한·견고파싱)
   ◐ 중간  ████████████████          코어 보조(필터·네트워크·로그·설정·health·CLI 보조)
   ○ 낮음  ██████                    표시·편의(uptime·PID부분일치·duration·default)

 종류 분포
   [U] 단위    ████████████████████████████████  대부분(framework/types/network/log/filter/tree/config/killer)
   [E] E2E     ███████                           cli_test 11(유일한 바이너리 구동)
   [M] 갭/수동 ████████                          TUI 렌더·키·실제 종료·FFI·마스킹·명령부재
```

### 6-3. 커버리지 갭 한눈 매트릭스 (자동 vs 수동)

```
                      │ 자동 테스트(있음)              │ 갭 — 수동/통합 필요(없음)
 ─────────────────────┼───────────────────────────────┼──────────────────────────────────
 분류(F1·F3·F4)       │ ✅ framework_test 24            │ —
 표시·health(F2·F18)  │ ✅ types_test(표시·health 분기) │ ⚠ platform FFI(메모리 정확도)·health MB 버그
 포트(F5·F6)          │ ✅ network_test(파싱)           │ ✗ lsof/netstat 부재 시나리오
 로그(F16)            │ ✅ log_test(glob·버퍼)          │ ⚠ /proc fallback
 필터/정렬(F11·12·13) │ ✅ filter_test(매칭·Node-only)  │ ✗ 정렬 비교자·필터 UI 키
 안전종료(F7·F8·F9)   │ ⚠ killer(시그널 메타)·tree(pids)│ ✗ 실제 graceful/force/역순/EPERM
 설정(F21)            │ ✅ config_test                  │ —
 CLI(F19)             │ ✅ cli_test(E2E 디스패치)       │ —
 TUI(F10·F15·F20…)    │ —                              │ ✗ 렌더링·키 핸들링 전부(STEP 7 수동)
 env 마스킹(F17)      │ —                              │ ✗ 마스킹·CLI 미마스킹(40 R8·R10)
```

---

## 자체 검증 (Self-verification)

| # | 체크 | 결과 | 어디서 충족 |
|---|---|---|---|
| **T-1** | 핵심 플로우별 케이스(UF-1~6) 전제/입력/기대/우선 | **PASS** — §2-1~§2-6 각 UF 케이스 표(TC-1.*~TC-6.*) | §2 |
| **T-2** | 핵심 기능별 케이스(분류·네트워크·로그·설정·health) | **PASS** — §3-1~§3-5(TC-7.*~TC-11.*) | §3 |
| **T-3** | 엣지/예외(EPERM·명령부재·좀비/고아·comm절단·npx오분류·버퍼·부분설정) | **PASS** — §4 TC-E1~E14, playbook 명시 엣지 전부 포함 | §4 |
| **T-4** | ★ 테스트 모듈 10종 매핑(함수명·커버 F#/UF#) | **PASS** — §5-1 표(framework/scanner/tree/killer/network/log/cli/config/filter/types, 실제 함수명) | §5-1 |
| **T-5** | 커버리지 갭 명시 | **PASS** — §5-2 갭 표(TUI 렌더·실제 종료·FFI·env 마스킹·명령부재·정렬비교자·health MB) + §6-3 | §5-2·§6-3 |
| **T-6** | 시각화(커버리지 맵 마인드맵 + 우선순위 분포, 외부 렌더러 0) | **PASS** — §6-1 마인드맵·§6-2 분포·§6-3 갭 매트릭스, 전부 ASCII | §6 |
| **T-7** | 단일 출처 준수(기능=10·플로우=13·키=21·값=31·정책=33·로드맵=40 참조) | **PASS** — 본문 참조 표기, 케이스만 소유 | 전체 |

**추가 원칙 점검:**
- **창작 금지(spec §7):** 결제·DB·REST·회원 케이스 0. 모든 테스트 모듈·함수명은 실제 `tests/*.rs`에서
  추출(framework_test 24·types_test 23·network_test 13·log_test 13·cli_test 11·filter_test 9·tree_test 8·
  killer_test 7·config_test 5·scanner_test 2 = 115). 없는 테스트를 지어내지 않음.
- **사실 정정 반영:** TUI 목록에 FRAMEWORK 컬럼 없음(TC-1.5는 Info 탭에서 확인)·세로 레이아웃·`Version=-`
  (40 R7)·health 메모리 인자 MB(40 R6)·CLI env 미마스킹(40 R8)을 정직 케이스/갭으로 기록.
- **갭 정직성:** killer_test가 **시그널 메타데이터만** 덮고 실제 종료 동작(graceful/force/역순/EPERM)은
  안 덮는다는 점, TUI 렌더·키 핸들링·platform FFI가 단위 테스트 0이라는 점을 §5-2·§6-3에 명시(숨기지 않음).
- **쉬운 설명(probe):** 첫 등장 용어(테스트 케이스·전제·단위/통합·커버리지 갭·EPERM·고아·comm 절단) 풀이.

**알려진 한계(후속 보완):**
- 본 문서는 **검증 케이스의 단일 출처**다. 기능·우선순위·키·값·정책·로드맵 항목 자체는 10/21/31/33/40을
  참조하며 재정의하지 않는다(같은 수치는 31 인용).
- `[M] 갭` 케이스(실제 종료·EPERM·TUI 렌더·env 마스킹·명령 부재)는 **수동/통합 검수 또는 추가 테스트
  여지**로 표기했을 뿐, 본 패키지는 문서화만 하며 실제 테스트 코드를 추가하지 않는다(spec §7 비목표).
- 테스트 수치(115·모듈별 카운트)는 현재 `tests/*.rs` 기준 — 코드 변경 시 갱신 대상이다.
