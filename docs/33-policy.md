# 33 · 정책 (Policy) — 안전·프라이버시 단일 출처 (ntop)

> **이 문서는 단일 출처(single source)다.** ntop의 **안전·프라이버시 정책** — kill 확인 게이트, 정확한
> 종료 정책(TUI vs CLI 경로 구분), 시그널 권한 매트릭스, 환경변수 민감값 마스킹 규칙, **"파일시스템 안 읽음"
> 프라이버시 원칙** — 은 **여기서만 규칙으로 확정**한다. `21-screen-spec`·`30-functional-spec`·`40-backlog`
> 은 이 정책을 **참조**할 뿐 재정의하지 않는다(spec §4-3).
>
> **★ 이 문서는 ntop의 정체성 문서다.** ntop은 단순 "보기" 도구가 아니라 **프로세스를 죽이는(destructive)
> 도구**다. 따라서 "어떻게 안전하게 죽이는가 + 어떻게 사용자 데이터를 안 새게 하는가"가 제품의 신뢰
> 기반이다(sprint-playbook: 안전·프라이버시 = 도구 정체성 → T1 승격).
>
> **값은 여기서 정의하지 않는다.** Config 기본값(`confirm_before_kill=true`·`graceful_timeout=10`·
> `mask_env_values=true` 등)과 enum(`KillSignal`/`KillResult`/`GracefulResult`)의 정규 출처는 **`31-erd`**,
> 종료·마스킹의 처리 알고리즘(graceful_kill 폴링·kill_tree 역순·분류)은 **`30-functional-spec`**, 외부
> 인터페이스 계약(CLI 플래그·시스템 명령)은 **`32-api-spec`**, 비용·지속가능성은 **`00-business-model`**
> 이다. 본 문서는 그 값·동작을 **인용만** 하고(중복정의 0), 그 위에 **규칙·정책·권한**을 정의한다.
>
> **전제:** ntop은 **이미 v0.2.0까지 구현된 MIT 오픈소스 도구**다. 본 정책은 "앞으로 만들 규칙"이 아니라
> **이미 구현된 안전·프라이버시 동작을 역설계한 정책 기술**이며, 모든 규칙은 실제 소스코드(`src/process/
> killer.rs`·`src/tui/ui.rs`·`src/tui/widgets/env_tab.rs`·`src/process/framework.rs`·`src/main.rs`·
> `src/config.rs`)에서 추출했다(창작 0).
>
> **용어 풀이(첫 등장):** **시그널(signal)** = OS가 프로세스에 보내는 짧은 알림/명령(SIGTERM·SIGKILL 등) /
> **graceful 종료(정상 종료)** = "정리하고 꺼져라"라고 **요청**(SIGTERM, 프로세스가 무시·대비 가능) /
> **force 종료(강제 종료)** = SIGKILL, 프로세스가 막을 수 없음 / **에스컬레이션** = 정상 요청이 안 통하면
> 강제로 단계 올림 / **EPERM** = 권한 없음 오류 / **ESRCH** = 그런 프로세스 없음 오류 / **모달(modal)** =
> 확인을 받을 때까지 다른 조작을 막는 대화상자 / **마스킹(masking)** = 민감한 값을 `********`로 가림 /
> **fs(filesystem)** = 파일시스템(디스크의 파일들).

---

## Sprint Contract (self-proposed checks)

이 문서(g4의 `33-policy`)가 만족시킬 **관찰 가능한 체크**(playbook §S4-g4 관찰 바 + spec §5(33행)·§8).

| # | 체크 | 어떻게 충족하나 | 본문 위치 |
|---|---|---|---|
| **P-1** | **kill 확인 게이트** — `confirm_before_kill`(기본 true, 31값) + 3 모달(KillConfirm/KillTreeConfirm/ForceKillPrompt) + CLI `[y/N]` | §1 게이트 규칙 표 | §1 |
| **P-2** | **★ 정확한 종료 정책(g4a 발견)** — TUI KillConfirm=SIGTERM 직접 send_signal(폴링 아님), 자동 graceful→force는 **CLI 단건 전용**, ForceKillPrompt 수동 분리, kill_tree 역순 | §2 + §2-3 경로 구분 플로우차트 | §2 |
| **P-3** | **시그널 권한 매트릭스** — OS×시그널 가용성(Unix 6종/Windows 3종) + EPERM→PermissionDenied·ESRCH→AlreadyDead | §3 히트맵 + 오류 매핑 표 | §3 |
| **P-4** | **env 민감값 마스킹** — `mask_env_values`(기본 true, 31값), 8 패턴(PASSWORD/SECRET/TOKEN…), `********`, **TUI 전용**(CLI info 미마스킹 정직 표기) | §4 + §4-3 적용 시퀀스 | §4 |
| **P-5** | **★ fs 미독 프라이버시 원칙** — 분류는 이름+커맨드만, package.json 등 fs 절대 미독(영구 원칙) | §5 원칙 + 위반 시 결과 | §5 |
| **P-6** | **시각화(G-g)** — 권한 매트릭스 히트맵 + kill 확인/에스컬레이션 플로우차트(TUI vs CLI) + env 마스킹 시퀀스(외부 렌더러 0) | §2-3·§3-1·§4-3 ASCII | §2-3·§3·§4 |
| **P-7** | **단일 출처 준수** — 값=31·동작=30·계약=32·지속가능성=00 참조, 정책만 소유. 결제/DB/REST 0 | 본문 곳곳 참조 | 전체 |

---

## 0. 정책 철학 — "기본적으로 안전, 명시적으로 위험"

> ntop의 안전 정책은 두 원칙으로 요약된다. (1) **파괴적 동작은 기본적으로 한 번 더 묻는다**(확인 게이트가
> 기본 on). (2) **정상 종료(요청)를 먼저, 강제 종료(막을 수 없음)는 명시적으로.** 프라이버시는 한 원칙:
> (3) **사용자 디스크를 안 읽는다**(분류는 프로세스 정보만).

```
 ┌──────────────────────────────────────────────────────────────────────────────┐
 │  ntop 정책 3축                                                                 │
 │                                                                                │
 │  ① 확인 게이트(§1)      confirm_before_kill=true(31) → 죽이기 전 1단계 확인     │
 │       파괴적 동작은 기본적으로 한 번 더 묻는다                                  │
 │                                                                                │
 │  ② 단계적 종료(§2)      SIGTERM(정상 요청) ─먼저─▶  SIGKILL(강제)는 명시적으로  │
 │       자동 에스컬레이션은 CLI 단건에서만 / TUI 강제는 별도 수동 모달            │
 │                                                                                │
 │  ③ fs 미독 프라이버시(§5)  분류 = 이름+커맨드만, 디스크(package.json) 절대 미독 │
 │       프라이버시 + 오분류 회피 = 영구 설계 원칙(되돌리기 금지)                  │
 │                                                                                │
 │   (+ 부수: env 민감값 마스킹 §3·§4 / 시그널 권한·오류 처리 §3)                  │
 └──────────────────────────────────────────────────────────────────────────────┘
```

**이 문서가 소유(정규 정의)하는 것:** 확인 게이트 규칙 · 종료 경로별 정책(TUI/CLI 구분) · 시그널 권한
매트릭스와 오류 처리 · env 마스킹 규칙(대상 패턴·적용 범위) · fs 미독 프라이버시 원칙.

**참조(여기서 정의 안 함):** Config 값·enum → **31** · 처리 알고리즘 → **30** · CLI 플래그/시스템명령 →
**32** · 키 바인딩 → **21** · 비용/후원/지속가능성 → **00**.

---

## 1. kill 확인 게이트 (Confirmation gate)

> **규칙:** 파괴적 동작(프로세스 종료)은 **기본적으로 한 번 더 확인**을 받는다. 게이트는 설정으로 끌 수
> 있으나 **기본값은 켜짐**(`confirm_before_kill = true`, 정규값 31 §4). 게이트는 TUI(모달)와 CLI(`[y/N]`
> 프롬프트) 양쪽에 존재한다.

### 1-1. 게이트 규칙 표

| 경로 | 트리거 | 게이트 형태 | 게이트 우회 |
|---|---|---|---|
| **TUI 단건** | 키 `x` | `KillConfirm` 모달(Enter=실행, Esc=취소) | (모달은 항상 — `confirm_before_kill`은 CLI 프롬프트를 제어) |
| **TUI 트리** | 키 `K` | `KillTreeConfirm` 모달 | 〃 |
| **TUI 강제** | (별도 진입) | `ForceKillPrompt` 모달(강제 종료 전용) | 〃 |
| **TUI 시그널선택** | 키 `S` | `SignalPicker` 모달(시그널 고른 뒤 Enter=전송) | 〃 |
| **CLI 단건** | `ntop kill <PID>` | `"Kill process <PID> with <SIG>? [y/N]"` | `--no-confirm` **또는** `confirm_before_kill=false`(31) |
| **CLI 트리** | `ntop kill <PID> --tree` | 대상 PID 목록 미리보기 + `[y/N]` | 〃 |
| **CLI 전체** | `ntop kill --all` | 서버 목록 미리보기 + `[y/N]` | 〃 |

> **코드-진실:** CLI 프롬프트는 `!no_confirm && config.general.confirm_before_kill`일 때만 출력된다
> (`main.rs cmd_kill`). 응답이 `y`(대소문자 무시)가 아니면 `"Cancelled."` 출력 후 **종료코드 0**(아무 것도
> 죽이지 않음, 32 §2-7). TUI 확인 모달의 키 바인딩 정규 출처는 **21-screen-spec**(여기서는 정책만).

---

## 2. ★ 정확한 종료 정책 (Exact kill policy) — TUI vs CLI 경로 구분

> **이 절이 ntop 종료 동작의 정책 단일 출처다.** ★ **흔한 오해를 바로잡는다(g4a 발견, 코드 정밀 분석):**
> "ntop은 항상 SIGTERM을 보내고 안 죽으면 자동으로 SIGKILL로 올린다"는 **부분적으로만 맞다.** 자동
> 에스컬레이션은 **CLI 단건 종료 경로에서만** 일어난다. **TUI는 그렇게 동작하지 않는다.** 정확히 구분한다.

### 2-1. 경로별 종료 정책 (정규)

| 경로 | 보내는 것 | 폴링/대기? | 자동 force 에스컬레이션? | 코드 근거 |
|---|---|---|---|---|
| **CLI 단건** `ntop kill <PID>` | `graceful_kill`: SIGTERM | **있음** — 200ms 폴링(30-6), `graceful_timeout`(31=10s)까지 | **있음** — `TimedOut`이면 `force_kill`(SIGKILL) 자동 호출 | `main.rs cmd_kill` |
| **TUI 단건** `KillConfirm`(Enter) | **`send_signal(pid, SIGTERM)` 직접 1회** | **없음** — 폴링/대기 안 함, `needs_rescan=true`만 | **없음** | `ui.rs` L99·L102 |
| **TUI 강제** `ForceKillPrompt`(Enter) | **`force_kill(pid)` = SIGKILL** | 없음 | (이미 강제) — **수동 별도 모달** | `ui.rs` L187 |
| **TUI 트리** `KillTreeConfirm`(Enter) | `kill_tree(pids, SIGTERM)` — **역순(자식부터)** | 없음(각 PID 1회씩) | 없음 | `ui.rs` L116 |
| **TUI 시그널선택** `SignalPicker`(Enter) | `send_signal(pid, 선택시그널)` 1회 | 없음 | 없음 | `ui.rs` L145 |
| **CLI 트리** `--tree` | `kill_tree(pids, signal)` — 역순 | 없음 | 없음 | `main.rs cmd_kill` |
| **CLI 전체** `--all` | 각 서버에 `send_signal(signal)` 1회씩 | 없음 | 없음 | `main.rs cmd_kill` |

### 2-2. 정책 해설 (왜 이렇게 갈리나)

- **CLI 단건만 자동 에스컬레이션:** 비대화형/스크립트 맥락에서는 "안 죽으면 어쩌지"를 사람이 다시 판단할
  수 없으므로, **SIGTERM → (10초 안 죽으면) → SIGKILL**까지 한 호출에서 끝낸다(편의·결정성).
- **TUI 단건은 SIGTERM만, 강제는 수동 분리:** 대화형 화면에서는 **사용자가 직접 다음 단계를 결정**한다.
  `KillConfirm`은 정상 종료(SIGTERM)만 보내고, 안 죽으면 사용자가 **별도로 `ForceKillPrompt`(강제 종료
  전용 모달)** 를 띄워 SIGKILL을 보낸다. 즉 **TUI에서 강제 종료는 항상 명시적·수동**이다("기본적으로 안전,
  명시적으로 위험" 원칙).
- **트리 종료는 자식부터 역순:** `kill_tree`는 `pids.rev()`로 **자식을 먼저** 죽인다(30-7). 부모를 먼저
  죽이면 자식이 고아(orphan)가 되어 추적/회수가 어려워지므로, 역순이 고아를 막는다.
- **폴링 상수:** graceful 폴링 간격 `200ms`는 알고리즘 상수(30 소유), 대기 한계 `graceful_timeout`은 Config
  값(31). 본 문서는 인용만.

### 2-3. 시각화 — kill 확인/에스컬레이션 플로우차트 (TUI vs CLI 구분, G-g)

```
                         ┌──────────────────────────────────┐
                         │  종료 요청 발생                    │
                         └───────────────┬──────────────────┘
            ┌──────────────────────────┴───────────────────────────┐
            ▼ (대화형)                                              ▼ (비대화형/스크립트)
 ╔══════════ TUI 경로 ═════════════════════╗      ╔══════════ CLI 경로 ════════════════════════╗
 ║ KillConfirm 모달(키 x)                   ║      ║ ntop kill <PID>                             ║
 ║   confirm 게이트(31)                     ║      ║   confirm_before_kill(31) & !--no-confirm   ║
 ║   Enter ─▶ send_signal(SIGTERM) 1회      ║      ║   → "Kill <PID>? [y/N]"                      ║
 ║   (폴링 없음, needs_rescan=true)         ║      ║      └ y 아니면 → "Cancelled." (exit 0)     ║
 ║      │                                   ║      ║   y ─▶ graceful_kill(pid, 10s 31)           ║
 ║      ▼ 안 죽었나? (사용자가 화면에서 판단)║      ║      ├ SIGTERM 전송                          ║
 ║   ForceKillPrompt 모달(수동 진입)        ║      ║      ├ 200ms 폴링 is_alive… (30)             ║
 ║   Enter ─▶ force_kill(SIGKILL)           ║      ║      │   ├ 죽음 ───────────▶ Terminated      ║
 ║   (강제는 항상 명시적·수동)               ║      ║      │   └ 10s 초과 ───────▶ TimedOut        ║
 ╠═════════════════════════════════════════╣      ║      ▼ TimedOut 이면 (★자동 에스컬레이션)    ║
 ║ 트리: KillTreeConfirm(키 K)              ║      ║   "Force killing..." ─▶ force_kill(SIGKILL)  ║
 ║   Enter ─▶ kill_tree(pids, SIGTERM)      ║      ║   (이 자동 force는 CLI 단건에서만!)          ║
 ║   = pids.rev() 역순(자식부터), 폴링 없음 ║      ╠═════════════════════════════════════════════╣
 ║ 시그널: SignalPicker(키 S)               ║      ║ --tree: 확인 → kill_tree(pids,sig) 역순      ║
 ║   Enter ─▶ send_signal(선택시그널) 1회   ║      ║ --all : 확인 → 각 서버 send_signal(sig) 1회  ║
 ╚═════════════════════════════════════════╝      ╚═════════════════════════════════════════════╝
   ▲ TUI: SIGTERM은 1회 직접 전송, 강제는 수동       ▲ CLI 단건만 SIGTERM→(timeout)→SIGKILL 자동
   (키 바인딩 정규 출처 = 21 / 알고리즘 = 30 / 값 graceful_timeout=10·SIGTERM 기본 = 31)
```

---

## 3. 시그널 권한 매트릭스 (Signal & permission matrix)

> **규칙:** 보낼 수 있는 시그널은 **OS에 따라 다르다.** Unix는 6종, Windows는 3종만 가능하다(`KillSignal::
> all()`, 31 §3-4). 시그널 전송 결과는 **권한·존재 여부에 따라 enum으로 분류**되며, 권한 부족(EPERM)이나
> 이미 죽음(ESRCH)을 **예외(크래시)가 아니라 결과 값**으로 다룬다.

### 3-1. ★ OS × 시그널 가용성 히트맵 (G-g)

```
                    가용성 매트릭스 (✓ = 보낼 수 있음 · ✗ = 그 OS엔 없음)
 ┌────────────┬──────────┬─────────────────────────────────────────────┬──────────┐
 │ KillSignal │ name()   │ 의미(description)                            │ Unix│Win │
 ├────────────┼──────────┼─────────────────────────────────────────────┼─────┼────┤
 │ Term       │ SIGTERM  │ Graceful termination request (정상 종료 요청)│  ✓  │ ✓  │  ← 기본 시그널
 │ Kill       │ SIGKILL  │ Force kill (cannot be caught) (막을 수 없음) │  ✓  │ ✓  │  ← 강제
 │ Int        │ SIGINT   │ Interrupt (like Ctrl+C)                      │  ✓  │ ✓  │
 │ Hup        │ SIGHUP   │ Hangup / reload configuration (설정 리로드)  │  ✓  │ ✗  │  ← Unix 전용
 │ Usr1       │ SIGUSR1  │ User-defined (Node.js: activate debugger)    │  ✓  │ ✗  │  ← Unix 전용
 │ Usr2       │ SIGUSR2  │ User-defined signal                          │  ✓  │ ✗  │  ← Unix 전용
 ├────────────┴──────────┴─────────────────────────────────────────────┼─────┼────┤
 │  all() 반환 개수/순서                                                 │  6  │ 3  │
 │  Unix: [Term, Kill, Hup, Int, Usr1, Usr2]   Windows: [Term, Kill, Int]│     │    │
 └──────────────────────────────────────────────────────────────────────┴─────┴────┘
   ※ Windows에 HUP/USR1/USR2가 없는 것은 OS 한계 — SignalPicker는 그 OS의 all()만 노출(40-backlog).
   ※ from_str("SIGTERM"|"TERM" 등) 대소문자·SIG 접두 무시 허용 (31 §3-4). enum 정의 = 31.
```

### 3-2. 시그널 전송 결과·오류 처리 정책

> **규칙:** 권한·존재 오류는 **사용자에게 명확한 결과로 보고**하고 도구를 중단시키지 않는다(graceful). 코드
> 근거: `killer.rs` unix_impl(`signal::kill` 결과 매핑) / windows_impl.

| 상황(Unix errno) | 단건 결과 `KillResult` | graceful 결과 `GracefulResult` | 사용자에게 의미 |
|---|---|---|---|
| 성공 | `Success` | (폴링 진행 → `Terminated`/`TimedOut`) | 시그널 전달됨 |
| **EPERM**(권한 없음) | `PermissionDenied` | `PermissionDenied` | 내 권한으로 못 죽임(예: 다른 사용자/루트 소유) |
| **ESRCH**(없는 PID) | `AlreadyDead` | `AlreadyDead` | 이미 죽었거나 사라진 PID |
| 기타 errno | `Error(문자열)` | `Error(문자열)` | 원인 메시지 표시 |

> **Windows 매핑(동등 정책):** `Kill`은 `OpenProcess`+`TerminateProcess`(에러 5→`PermissionDenied`,
> 87→`AlreadyDead`), `Term`/`Int`은 `taskkill /PID`(stderr `"Access"`→`PermissionDenied`, `"not found"`
> →`AlreadyDead`). 결과 enum은 31 §3-4, 처리 절차는 30-8, 시스템 명령 계약은 32 §5.
>
> **정책 함의:** 권한 부족·이미 죽음은 **정상 흐름의 결과**다 — ntop은 이를 메시지로 알리고 계속 동작한다
> (CLI는 종료코드도 0, 32 §2-7). "권한 없으면 패닉/비정상 종료"하지 않는다.

---

## 4. 환경변수 민감값 마스킹 (Env masking)

> **규칙:** 상세 패널의 환경변수 표시에서 **민감해 보이는 키의 값은 기본적으로 가린다**(`mask_env_values =
> true`, 정규값 31 §4). 패스워드·토큰 등이 어깨너머·화면 공유로 새는 것을 막는다(PRB-6).

### 4-1. 마스킹 규칙 (정규)

| 항목 | 규칙 | 코드 근거 |
|---|---|---|
| **토글** | `display.mask_env_values`(기본 `true`, 31값). false면 원문 표시 | `env_tab.rs` L42 |
| **대상 판정** | 키를 **대문자로 변환** 후, 아래 8개 패턴 중 하나라도 **부분 문자열로 포함**하면 민감 | `is_sensitive` L25-30 |
| **민감 패턴(8)** | `PASSWORD` · `SECRET` · `TOKEN` · `KEY` · `API_KEY` · `PRIVATE` · `CREDENTIALS` · `AUTH` | `SENSITIVE_PATTERNS` L13-22 |
| **가린 값** | 값을 `********`(별표 8개)로 치환. 키 이름은 그대로 노출 | `env_tab.rs` L48-49 |

> **판정 예:** `DB_PASSWORD`→가림(PASSWORD 포함), `GITHUB_TOKEN`→가림, `API_KEY`→가림, `MY_SECRET_X`→가림,
> `AUTH_HEADER`→가림, `PUBLIC_URL`→노출(패턴 없음). `KEY`가 부분일치라 `KEYBOARD_LAYOUT` 같은 무해한 키도
> 가려질 수 있다(과(過)마스킹은 안전 측 — 정직 표기, 정밀화는 40-backlog).

### 4-2. ★ 적용 범위(코드-진실) — TUI 전용

> **정직 표기(중요):** 마스킹은 **TUI 상세 Env 탭에서만** 적용된다. **CLI `ntop info`의 환경변수 출력은
> 마스킹하지 않고 `KEY=VALUE` 원문을 그대로 찍는다**(`main.rs cmd_info` L503-505, 32 §2-3). 즉 화면 공유
> 중 `ntop info`로 env를 덤프하면 민감값이 노출된다. 이는 실제 v0.2.0 동작이며 프라이버시 갭으로 식별됨 —
> **개선(예: info에도 마스킹 적용)은 40-backlog 후보**(창작 아님, 현재 한계의 정직한 기록).

```
 적용 여부 한눈:
   TUI Env 탭        : mask_env_values=true → 민감 키 값 ********  ✓ 마스킹
   CLI `ntop info`   : env_vars 원문 KEY=VALUE                    ✗ 마스킹 안 됨 (40-backlog)
   `list --json/csv` : env 미포함(env 컬럼 없음)                  — 해당 없음
```

### 4-3. 시각화 — env 마스킹 적용 시퀀스 (G-g)

```
 사용자          TUI Env 탭(render)        is_sensitive(key)         화면
   │                  │                        │                      │
   │ 상세 → Env 탭     │                        │                      │
   ├─────────────────▶│ for (key, value) in env_vars:                 │
   │                  │ mask_env_values?(31 기본 true)                │
   │                  │   ├ false → value 원문 ───────────────────────▶ "DB_HOST=localhost"
   │                  │   └ true  → is_sensitive(key)?                 │
   │                  ├───────────────────────▶│ key.to_uppercase()   │
   │                  │                        │ 8패턴 중 contains?    │
   │                  │◀── true(민감) ─────────┤                      │
   │                  │   value = "********" ─────────────────────────▶ "DB_PASSWORD=********"
   │                  │◀── false(무해) ────────┤                      │
   │                  │   value 원문 ────────────────────────────────▶ "PUBLIC_URL=https://…"
   (※ CLI `ntop info`는 이 분기를 거치지 않고 원문 출력 — §4-2 정직 표기)
```

---

## 5. ★ "파일시스템 안 읽음" 프라이버시 원칙 (No-filesystem-read)

> **★ 영구 설계 원칙(되돌리기 금지).** ntop의 프로세스 분류는 **프로세스 이름 + 커맨드라인만** 본다.
> **`package.json`·`.env`·프로젝트 설정 등 디스크의 어떤 파일도 읽지 않는다**(코드 근거: `framework.rs
> classify`는 `name`/`command`/`&Config`만 입력으로 받고 cwd/fs 미접근, 30-1·32 §4). 이 원칙은 **반전을
> 제안하는 것 자체가 정책 위반**이다(spec §7).

### 5-1. 왜 fs를 안 읽는가 — 두 이유

| 이유 | 설명 |
|---|---|
| **① 프라이버시** | 사용자 프로젝트 디렉터리의 파일을 열지 않으므로, **모니터링 도구가 소스/설정/비밀파일을 읽을 일이 없다.** 도구가 보는 것은 OS가 이미 노출하는 프로세스 메타데이터(이름·커맨드·cwd 문자열)뿐. |
| **② 오분류 회피(정확성)** | cwd 기반 fs 읽기는 **전역 실행 프로세스를 오분류**한다. 예: `npx`로 띄운 MCP 서버나 CLI 도구는 **상속된(엉뚱한) cwd**를 가질 수 있어, 그 cwd의 `package.json`을 읽으면 "그 디렉터리의 프레임워크"로 잘못 찍힌다. 이름+커맨드만 보면 이 함정을 피한다. |

### 5-2. 안전 정책과의 연결 (왜 "안전"에 중요한가)

```
 fs 미독 ──┬─▶ [프라이버시]  도구가 사용자 디스크 파일을 안 읽음 → 신뢰
           │
           └─▶ [오분류 회피] ──▶ 분류 정확 ──▶ "서버"로 식별된 것만 kill 후보 풀에 들어감
                                                  (비서버=None=미표시, 30-1)
                                                       │
                                                       ▼
                                       엉뚱한 프로세스를 죽일 위험 ↓ (PRB-3, F3)
```

- fs 미독으로 분류가 정확해지면, **"서버로 식별된 것"만 종료 후보**가 된다(미분류=`None`=목록 비노출, 30-1).
  이는 §1·§2의 확인 게이트·단계적 종료와 합쳐져 **"엉뚱한 프로세스 kill"** 위험을 낮춘다(F3, 10-prd §5
  프라이버시·안전성 NFR).
- **금지(spec §7):** "fs를 읽어 분류를 강화하자"는 제안은 이 원칙(영구)과 충돌 → 정책 위반. 향후 로드맵도
  이 원칙을 반전하지 않는 범위에서만 다룬다(40-backlog).

---

## 6. 비용·지속가능성 정책 (참조 — 00 소유)

> ntop은 **MIT 라이선스 OSS**이며 결제·정산·회원 과금이 없다. 안전·프라이버시 정책을 유지·발전시키는 **비용
> 구조(메인테이너 시간·CI), 채택 퍼널, 수익화 가설(GitHub Sponsors·기업 후원 등 — 전부 "가설")의 단일
> 출처는 `00-business-model`**이다. 본 문서는 정책만 소유하며 비용/수익 수치를 재정의하지 않는다(중복 0).
> **결제 파트너·take-rate·정산 규칙은 존재하지 않는다**(spec §7 — 창작 금지).

---

## 자체 검증 (Self-verification)

| # | 체크 | 결과 | 어디서 충족 |
|---|---|---|---|
| **P-1** | kill 확인 게이트(기본 true 31 + 3모달 + CLI [y/N] + 우회) | **PASS** — §1-1 표(경로별 게이트·우회 조건), 취소 시 exit 0 | §1 |
| **P-2** | ★ 정확한 종료 정책(TUI SIGTERM 직접/CLI 단건만 자동 force/ForceKillPrompt 수동/트리 역순) | **PASS** — §2-1 경로별 표 + §2-2 해설 + §2-3 TUI vs CLI 플로우차트 | §2 |
| **P-3** | 시그널 권한 매트릭스(Unix6/Win3) + EPERM·ESRCH 처리 | **PASS** — §3-1 히트맵 + §3-2 오류 매핑 표(KillResult/GracefulResult) | §3 |
| **P-4** | env 마스킹(기본 true 31, 8패턴, ********, TUI 전용/CLI 미마스킹 정직) | **PASS** — §4-1 규칙 + §4-2 적용범위 정직표기 + §4-3 시퀀스 | §4 |
| **P-5** | ★ fs 미독 프라이버시 원칙(영구, 반전 금지) | **PASS** — §5 두 이유(프라이버시·오분류 회피) + 안전 연결 + 금지 명시 | §5 |
| **P-6** | 시각화(히트맵·플로우차트·시퀀스, 외부 렌더러 0) | **PASS** — §3-1 히트맵·§2-3 플로우차트·§4-3 시퀀스(mermaid 0) | §2-3·§3·§4 |
| **P-7** | 단일 출처 + 창작 0 | **PASS** — 값=31·동작=30·계약=32·지속=00 참조, 결제/DB/REST 0(§6) | 전체 |

**추가 원칙 점검:**
- **창작 금지(spec §7):** 결제·정산·take-rate·DB·REST·회원 정책 **0**(§6 명시). 모든 규칙은 실제 소스
  (`killer.rs`·`ui.rs`·`env_tab.rs`·`framework.rs`·`main.rs`·`config.rs`)에서 추출.
- **코드-진실 정직 표기:** ① TUI KillConfirm은 graceful 폴링이 아니라 **SIGTERM 직접 전송**, 자동 force는
  **CLI 단건 전용**, TUI 강제는 **수동 ForceKillPrompt**(§2-1, g4a 발견); ② env 마스킹은 **TUI 전용**, CLI
  `info`는 미마스킹(§4-2); ③ `KEY` 부분일치로 인한 과마스킹 가능성(§4-1) — 모두 실제 v0.2.0 동작(개선 =
  40-backlog).
- **중복정의 0:** `confirm_before_kill=true`·`graceful_timeout=10`·`mask_env_values=true`·`SIGTERM` 기본은
  값으로 재정의하지 않고 "31값" 인용, 폴링 200ms·kill_tree 역순 등 알고리즘은 "30 참조", CLI 플래그·시스템
  명령은 "32 참조", 비용/수익은 "00 참조".
- **fs 미독 원칙 미반전:** §5에서 영구 원칙으로 명시하고 반전 제안을 정책 위반으로 규정(spec §7 준수).
- **쉬운 설명:** 시그널·graceful·force·에스컬레이션·EPERM·ESRCH·모달·마스킹·fs 첫 등장 풀이.

**알려진 한계(후속 보완):**
- CLI `ntop info`의 env 미마스킹은 프라이버시 갭 — 개선 후보(단일 출처 40-backlog).
- 마스킹 패턴이 부분일치라 과마스킹 가능(`KEY` 등) — 정밀화 여지(40-backlog).
- Windows는 Unix 전용 시그널(HUP/USR1/USR2) 부재 — OS 한계(40-backlog).
- 정책 값(게이트·timeout·마스킹 기본)이 코드/31-erd에서 바뀌면 본 문서가 그 정규값을 따른다.
