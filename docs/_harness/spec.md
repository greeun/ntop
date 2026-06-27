# 서비스 기획서 Spec: ntop — 멀티 런타임 서버 프로세스 모니터/매니저 (역설계 풀 기획 패키지)

> 이 문서는 PLANNER가 작성한 **Generator용 명세**다. Generator는 이 대화를 보지 못하며,
> **오직 이 `spec.md` + `sprint-playbook.md` + `docs/_harness/codebase-facts.md`(사실의 원천)** 만 읽고 산출물을 만든다.
> 모든 사실은 codebase-facts.md(이하 "도시에/dossier")와 실제 소스코드에서만 가져온다. **추측·창작 금지.**
>
> **고정 파라미터 (재질문 금지):**
> - **출력 모드: Mode 5 (Full Planning Package)** — `docs/` 아래 번호가 매겨진 ~16종 문서 + INDEX + HTML.
> - **HTML 형태: Both** — 문서별 HTML(`NN-name.html`) + 허브 `index.html` + 다이어그램 종합 대시보드 `overview.html`. md 원본은 항상 보존.
> - **출력 언어: 한국어** — 산출물 본문과 모든 도메인 서술은 한국어. (템플릿의 섹션 라벨은 영어지만 **제목은 한국어로 렌더링**, 정규 파일명/식별자/코드 심볼은 원문 유지.)
>
> **핵심 전제:** ntop은 **이미 v0.2.0까지 구현된 MIT OSS 도구**다. 따라서 이 기획서는 신규 아이디어가 아니라
> **구현된 코드를 역설계(reverse-engineering)한 기획 문서 패키지**다. "앞으로 만들 것"이 아니라 "무엇이 왜 이렇게 만들어졌는가 + 향후 로드맵"을 기술한다.

---

## 1. 한 줄 요약 (One-line summary)

여러 런타임(Node·Python·Java·Deno·Bun·Ruby·PHP·.NET)의 **서버 프로세스를 런타임/프레임워크 인지(awareness) 수준으로 모니터링·관리**하는 Rust 製 단일 바이너리 TUI+CLI 도구 `ntop`의, 이미 구현된 동작을 역설계해 **개발/기여/채택/후원 의사결정에 쓰이는 16종 기획 문서 패키지**로 정리한 것이다. 독자는 ntop 메인테이너·기여자·잠재 채택자(개발자/운영자)·후원 검토자다.

## 2. 대상 사용자와 핵심 목적 (Target users & core purpose)

이 기획서에는 **두 층의 사용자**가 있고, 둘을 혼동하면 안 된다.

1. **기획 문서의 독자(이 패키지를 읽는 사람):** ntop 메인테이너, 신규 기여자(온보딩), 도구 채택을 검토하는 팀 리드, GitHub Sponsors/기업 후원 검토자, QA. → INDEX.md의 역할별 번들이 이들을 위한 동선이다.
2. **ntop이라는 제품의 사용자(문서가 묘사하는 대상):** **로컬에서 여러 런타임의 서버 프로세스를 동시에 띄우는 개발자/운영자.** 예: Next.js 프런트 + FastAPI 백엔드 + 여러 워커를 한 머신에서 돌리는 풀스택 개발자, 다수 서비스 프로세스를 점검하는 운영자, npx/MCP 서버를 다수 띄우는 도구 사용자.

핵심 목적: 범용 시스템 모니터(htop/top/Activity Monitor)가 보여주는 "익명의 `node` 프로세스 더미"를 **"포트 3000에서 도는 Next.js dev 서버"** 수준의 의미로 식별하고, 그 자리에서 **안전하게 종료(graceful→force, 트리 종료)** 까지 끝내는 것. 본 기획 패키지의 목적은 그 가치/동작/경계를 개발 착수·기여·채택·지속가능성 판단이 가능한 수준으로 문서화하는 것이다.

## 3. 기획 관점 · 분석 프레임 (Planning perspective · analytical framework)

Generator는 아래 3개 프레임을 **품질 기준(quality)** 으로 강제한다. 브랜드 네임드롭("토스처럼", "북극성 지표급" 등) 금지.

- **who-what-why 문제 정의 프레임:** "누가/무엇이/왜" 문제인지 날카롭게. 여기서의 핵심 문제는 **범용 프로세스 모니터와 "내 개발 서버" 사이의 의미 격차(semantic gap)** 다 — 범용 모니터는 PID·CPU·메모리는 보여주지만 "이 `node`가 어느 프레임워크의, 어느 포트의, 어느 프로젝트 서버인지"는 모른다. 개발자는 그 격차를 `lsof -i :3000`, `ps aux | grep node`, 활성 모니터 사이를 오가며 수동으로 메운다. 이 수작업/문맥전환이 문제다.
- **타깃 페르소나 구체성 요건:** 인구통계("20대 개발자")만 쓰면 **실패**. 반드시 **상황·맥락·페인포인트(빈도/결과 포함)** 까지 구체화한다. 예: "동일 머신에서 `node` 프로세스 5개가 떠 있는데 어느 게 죽일 안전한 dev 서버이고 어느 게 건드리면 안 되는 MCP/도구 프로세스인지 구분 못 해, 매번 포트로 역추적하다 엉뚱한 프로세스를 kill한 적이 있는 풀스택 개발자."
- **차별화 프레임(도메인 가치 훅):** 최소 1개의 **실명 경쟁자**와 **명시적 비교 축**으로 비교해야 한다(§6). "더 편리/더 빠름" 같은 일반론은 실패. ntop의 차별 축과 모방난이도(moat):
  - **런타임/프레임워크 인지 분류** — 데이터 주도 규칙 테이블(`FRAMEWORK_RULES`/`RUNTIME_RULES`)로 프로세스를 "Next.js/FastAPI/Spring Boot/Rails…"까지 식별. (범용 모니터엔 없음)
  - **프로세스-로컬 전용 탐지 원칙** — 이름+커맨드라인만 보고 분류, **파일시스템(`package.json`)을 절대 읽지 않음.** 전역 실행 프로세스를 상속 cwd로 오분류하는 함정을 의도적으로 회피. (= 설계 철학이자 차별점)
  - **포트→프로세스 매핑 내장**(LISTEN), **트리 종료 + graceful→force 에스컬레이션**, **TUI+CLI 동일 코어** 단일 바이너리, **8개 런타임 횡단**.

## 4. 구조 / 흐름 (Structure / flow — Mode 5: 16종 문서 패키지)

> Mode 5는 단일 문서가 아니라 `docs/` 아래 **5단계로 묶인 ~16종 번호 문서 + INDEX + HTML**이다. mode-templates.md의 Mode 5 골격을 따르되, **아래 ntop 도메인 재해석을 반드시 적용한다.** 각 문서는 구조(계층/흐름/관계/시간축/비교/상태전이)가 있으면 본문에 **개념 적합 시각화(G-g)** 를 임베드한다: md=ASCII 다이어그램/매트릭스, HTML=인라인 SVG/CSS, 가능하면 생성 이미지. **Mermaid 등 외부 렌더러 의존 금지(자체완결).**

### 4-0. 도메인 재해석 (codebase-facts §0-1 — 위반 시 즉시 실패)

ntop은 **백엔드 서버·DB·결제·HTTP API·회원 계정이 없는 로컬 개발자 도구**다. 16종 문서를 기계적으로 적용하지 말고 아래대로 재해석한다. **결제/정산/take-rate/회원 DB/REST 엔드포인트를 지어내면 창작(fabrication) = 실패.**

| 표준 문서 | ntop 재해석 (필수) |
|---|---|
| 00-business-model | **OSS 채택·지속가능성 모델.** 라이선스(MIT), 배포 채널(crates.io `cargo install`/Homebrew/릴리스 바이너리/`--git` 설치), GitHub Stars·다운로드 기반 채택 퍼널, 비용 구조(메인테이너 시간·CI), **수익화 옵션(GitHub Sponsors·기업 후원·향후 Pro/Team 기능)은 전부 "가설"로 명시.** 정산/take-rate 절대 없음. |
| 31-erd | **인메모리 도메인 모델.** DB 없음. `ProcessInfo`·`Config`·`NetworkConnection`·`LogStreamer`·`App`·enum(`Runtime`/`FrameworkKind`/`HealthStatus`/`KillSignal`/`DetailTab`/`DialogKind`/`SortColumn`) 등의 필드·관계를 ERD 스타일(엔티티 박스 + 1:N 관계선)로 시각화. |
| 32-api-spec | **외부 인터페이스 계약.** ① CLI 명령 계약(서브커맨드/플래그/종료코드/`--json`·`--format csv` 출력 스키마) ② 공개 라이브러리 API(`ntop::` 크레이트 공개 타입/함수) ③ 의존 시스템 명령(`lsof`/`netstat`/`taskkill`)과의 계약. **HTTP 엔드포인트 아님.** |
| 33-policy | **안전·프라이버시 정책.** kill 확인 게이트, 시그널 권한(EPERM/PermissionDenied), 환경변수 민감값 마스킹(`mask_env_values`), **"파일시스템 안 읽음" 프라이버시 원칙**, graceful→force 에스컬레이션 정책, 트리 종료(자식부터 역순) 규칙. |
| 03-personas | 결제 고객 아님 → **개발자/운영자 페르소나.** |
| 40-backlog | 이미 v0.2.0 구현 완료 → **"현재까지 구현된 것(완료 백로그) + 향후 로드맵"** 관점. 로드맵 근거는 도시에 §12(미구현 흔적)와 코드 한계뿐. 근거 없는 기능 창작 금지. |
| 20-wireframes / 21-screen-spec | ntop은 GUI 웹앱이 아니라 **터미널 TUI + CLI**다. "화면"=TUI 뷰/패널/탭/다이얼로그 + CLI 명령 출력. ASCII 와이어프레임이 TUI의 실제 충실도이므로 ASCII 레이아웃이 곧 화면 시안이다(HTML은 인라인-SVG 터미널 프레임). |

### 4-1. 16종 문서 목록과 골격 (각 문서의 ntop화 내용)

```
docs/
─ ① 발견/전략 (왜 만드는가)
  00-business-model.md   01-service-plan.md   02-market-competition.md   03-personas.md
─ ② 정의 (무엇을 만드는가)
  10-prd.md   11-user-stories.md   12-ia.md   13-user-flows.md
─ ③ 디자인 (어떻게 보이는가)
  20-wireframes.md   21-screen-spec.md
─ ④ 기술 (개발 명세)
  30-functional-spec.md   31-erd.md   32-api-spec.md   33-policy.md
─ ⑤ 실행/검증
  40-backlog.md   41-qa-testcases.md
─ 가이드 + HTML (Both)
  INDEX.md (최후 생성)   index.html(허브)   NN-name.html(문서별)   overview.html(종합 대시보드)
```

각 문서 골격(요지)과 ntop 전용 내용 + 필수 시각화:

- **00-business-model** — OSS 비용 구조 + 채택 퍼널 + 배포 채널별 비교 + **수익화 가설(가설 표기 필수)**. 시각화: 채택 퍼널(인지→설치→상시사용→기여/후원, 단계별 전환 지점)·배포 채널 비교 매트릭스·비용/지속가능성 구조도. **money-flow는 "기여/후원 흐름"으로 치환**, 정산 시퀀스 금지.
- **01-service-plan** — Mode 1의 7섹션(문제/타깃/핵심기능/플로우/MVP 범위/성공지표/차별화) + "OSS 지속가능성"(00 링크). MVP=v0.2.0 출시 범위, Out-of-scope=로드맵·명시적 제외(웹 GUI, fs 기반 탐지 등). 시각화: 구조 마인드맵·가치 루프(스캔→식별→점검→종료)·MVP↔향후 2축 쿼드런트.
- **02-market-competition** — **생태계 분석(전 참여자/역할/인센티브/가치교환/의존) + 생태계 맵** + 실명 경쟁자 비교 매트릭스(✓/△/✗) + 포지셔닝 쿼드런트 + 갭/기회. 경쟁자는 **실재하는 프로세스 모니터**(§6 목록). 시각화: 생태계 맵(가치 네트워크)·포지셔닝 2축 산점도·비교 매트릭스.
- **03-personas** — 개발자/운영자 핵심 페르소나(상황/페인포인트/빈도/원하는 결과). 시각화: 페르소나 여정 맵(서버 여러 개 띄움→어느 게 뭔지 모름→ntop으로 식별→안전 종료)·가치 루프·(가능 시) 설명 웹툰 패널.
- **10-prd** — 기능 ID 목록 + 우선순위(P0/P1/P2) + 기능별 요약 + 비기능 요구(성능: 스캔 tick·CPU delta 정확도, 정확성: macOS phys_footprint, 크로스플랫폼, 안전성) + MVP/Out-of-scope. 기능은 코드 실제 동작에서 도출(모니터링/분류/포트/트리/종료/시그널/필터/정렬/Node-only/로그/네트워크/Env/설정/CLI 5종). 시각화: 기능 마인드맵(우선순위 색)·범위 쿼드런트.
- **11-user-stories** — Epic→Story 계층, 각 스토리 수용 기준(Given-When-Then) + 연결 기능 ID. ntop Epic 예: "프로세스 식별", "안전한 종료", "리소스 점검", "CLI 자동화". 시각화: Epic-Story 트리.
- **12-ia** — **인터페이스 정보구조.** TUI 레이아웃 계층(상단바 / 좌 프로세스 목록(트리) / 우 상세 패널[Info·Log·Net·Env 탭] / 다이얼로그[KillConfirm·KillTreeConfirm·SignalPicker·ForceKillPrompt·Help] / 필터 모드) + **CLI 명령 트리**(`ntop`→TUI, `list`/`kill`/`info`/`log`/`config`). 시각화: IA 트리 다이어그램(역할별 색).
- **13-user-flows** — 주요 플로우(역할/모드별) 단계 + 분기 + 각 단계의 화면 ID/기능 ID 링크. 핵심 플로우: TUI 기동→스캔→선택→상세 점검→종료; 필터/정렬/Node-only; kill 트리; SignalPicker; CLI `list`→`kill --tree`. 시각화: 유저 플로우 플로우차트 + 핵심 세션(kill graceful→force) 시퀀스/상태 다이어그램.
- **20-wireframes** — **화면(=TUI 뷰/다이얼로그)당 1개 저충실도 시안.** 메인 뷰, 상세 4탭, 각 다이얼로그, 필터 모드, 빈 상태(empty_state). md=ASCII 터미널 레이아웃, HTML=인라인-SVG 터미널 프레임. 시각 품질은 LLM 한계 → **STEP 7 사용자 검수 대상(HUMAN_CHECKPOINT_REQUIRED)**.
- **21-screen-spec** — 화면당 시안/컴포넌트 배치 + **핵심 화면 4상태 세트(empty/loading/normal/error)** + 화면별 명세(목적/컴포넌트/상호작용/상태/예외/연결 기능/연결 인터페이스). **키 바인딩의 정규 출처(single source)** 가 여기. 프로세스 없음=empty, 스캔 중=loading(스피너 tick_count), 권한거부/명령부재=error. 산문만 금지(G-g).
- **30-functional-spec** — 기능별 입력/처리/출력/제약/검증/예외. **탐지 규칙 테이블(2단계 분류: FRAMEWORK→RUNTIME→dev runner)·스캔 2-pass·CPU delta·graceful_kill 폴링·네트워크 파싱**의 정규 출처. 시각화: 분류 의사결정 플로우차트(name_exact→command_binary→command_contains, None=미표시)·스캔 2-pass 플로우·graceful→force 처리 플로우.
- **31-erd** — 엔티티(구조체/enum) + 필드(타입/제약) + 관계(`ProcessInfo.children` 자기참조 1:N, `App`↔`ProcessInfo` 1:N, `App`↔`Config`, `ProcessInfo`↔`Runtime`/`FrameworkKind`, `NetworkConnection`↔pid). **Config 기본값과 enum 값의 정규 출처.** 시각화: ERD 다이어그램(엔티티 박스+관계선, 인라인 SVG/ASCII). DB 없음 명시.
- **32-api-spec** — ① CLI 계약: 각 서브커맨드 method/플래그/필수성/종료코드/출력 스키마(table 컬럼, JSON 전 필드+depth, CSV 9컬럼) ② 공개 라이브러리 API(`ntop::` 타입/함수) ③ 의존 시스템 명령 계약(`lsof`/`netstat`/`taskkill` 호출·파싱·부재 시 빈 결과). 시각화: CLI 호출 시퀀스(CLI→Scanner→NetworkInspector→시스템명령)·명령/플래그 표.
- **33-policy** — **안전·프라이버시 정책.** kill 확인 게이트(`confirm_before_kill`), 시그널 권한 매트릭스(런타임 OS×시그널 가용성: Unix 6종/Windows 3종, EPERM 처리), env 민감값 마스킹 규칙, **"파일시스템 안 읽음" 프라이버시 원칙**, graceful→force 에스컬레이션·트리 역순 규칙. **31의 Config 값을 참조(중복정의 금지).** 시각화: 권한 매트릭스 히트맵·kill 확인/에스컬레이션 플로우차트·마스킹 동의/적용 시퀀스.
- **40-backlog** — **완료 백로그(v0.2.0까지 구현된 것) + 향후 로드맵.** 로드맵 근거=도시에 §12(Express/Fastify/Koa/Hapi enum 존재·규칙 미등록, include_bun deprecated, Windows 시그널 한계, 외부명령 의존, 로그 stdout 캡처 한계)뿐. 시각화: 완료/예정 타임라인 또는 간트(완료 구간 강조)·로드맵 우선순위.
- **41-qa-testcases** — 핵심 플로우별 테스트 케이스(전제/입력/기대결과/우선순위) + 엣지/예외. 실제 테스트 모듈 10종(framework/scanner/tree/killer/network/log/cli/config/filter/types)에 매핑. 시각화: 테스트 커버리지 맵(마인드맵)·케이스 표.
- **INDEX.md** — 최후 생성. 중요도 3티어 + 의존 기반 작업순서 + 역할별 번들 + 최소 시작 세트. 신규 콘텐츠 생성 금지(분류·정렬만).
- **HTML(Both)** — 각 `.md`→`NN-name.html`(ASCII 보존 + 인라인-SVG 인포그래픽), 허브 `index.html`(INDEX.md 반영), 종합 `overview.html`(마인드맵/차트/생태계/플로우 종합). 신규 콘텐츠 금지(변환·시각화만).

### 4-2. 교차 추적성(cross-traceability) 척추 — 필수

모든 문서는 상호 추적 가능해야 한다. 추적 축: **기능(feature) ↔ 문제(problem) ↔ 화면/인터페이스(screen/interface) ↔ 데이터(data) ↔ 계약(contract).**

```
문제(01·03)  →  기능 ID(10-prd)  →  화면·키·CLI(12·13·20·21)  →  데이터 필드(31-erd)  →  계약(32-api-spec) / 정책(33)
   예) "어느 node가 뭔지 모름" → F-분류(P0) → 상단 목록 NAME 컬럼 + 상세 Info탭(★FRAMEWORK는 TUI 목록 컬럼이 아니라 Info 탭·CLI list 테이블에만 노출; 레이아웃은 세로 상/하) → ProcessInfo.framework/runtime/ports → list 출력 스키마·display_name()
   예) "엉뚱한 프로세스 kill 위험" → F-안전종료(P0) → KillConfirm 다이얼로그·x/K/S 키 → KillSignal/GracefulResult → kill 서브커맨드·graceful 정책(33)
```

- 10-prd의 모든 P0/P1 기능은 01/03의 특정 문제로 역추적되어야 한다(기능↔문제 추적표 = G-a).
- 21-screen-spec의 각 화면/상태는 13-user-flows의 단계와 10-prd의 기능 ID에 링크.
- 30/32는 31의 데이터·enum과 21의 키 바인딩을 참조하되 재정의하지 않는다.

### 4-3. 단일 출처 레지스트리(single source of truth) — 중복·모순 금지

| 정규 사실 | 정규 출처(이곳에서만 정의) | 참조하는 문서 |
|---|---|---|
| OSS 지속가능성/채택 모델 + 수익화 가설 + 비용 구조 | **00-business-model** | 01, 40 |
| 핵심 가치 / MVP 범위 / 성공지표 | **01-service-plan** | 10, 40 |
| 경쟁/생태계 분석 + 비교 매트릭스 | **02-market-competition** | 00, 01 |
| 페르소나 | **03-personas** | 01, 10, 11, 13, 41 |
| 기능 목록 / ID / 우선순위 | **10-prd** | 11, 12, 13, 21, 30, 40, 41 |
| 탐지 규칙 테이블·분류 알고리즘·스캔 2-pass | **30-functional-spec** | 10, 32, 41 |
| 구조체/enum 정의 + Config 기본값 | **31-erd** | 30, 32, 33, 21 |
| 외부 인터페이스 계약(CLI/lib/시스템명령) | **32-api-spec** | 21, 30, 41 |
| 안전·프라이버시 정책 값(graceful/마스킹/권한) | **33-policy** (31 참조) | 21, 30, 40 |
| 키 바인딩(전역/리스트/상세/필터/다이얼로그) | **21-screen-spec** | 13, 30, 41 |

같은 수치(예: `refresh_interval=3`, `graceful_timeout=10`, `MAX_BUFFER_LINES=1000`)는 정규 출처(31)에만 값으로 적고, 다른 문서는 "31 참조"로만 언급한다.

## 5. 산출물 (Deliverables)

최종 산출물 = `docs/` 아래 16종 md + INDEX.md + HTML(Both). 각 산출물의 품질 기준과 검증 방법:

| # | 파일 | 설명 | 품질 기준(quality bar) | 검증 방법 |
|---|---|---|---|---|
| 00 | 00-business-model.md | OSS 채택/지속가능성 모델 | 배포 채널 ≥3 비교 + 채택 퍼널 단계별 정의 + 수익화는 "가설" 명시 + 정산/take-rate 0 | 가설 라벨 존재·결제 용어 부재 grep |
| 01 | 01-service-plan.md | 핵심 기획(7섹션+지속가능성) | 7섹션 충족 + MVP=v0.2.0/Out-of-scope 분리 + 차별화 실명비교 + 마인드맵·가치루프 | 섹션 체크·시각화 존재 |
| 02 | 02-market-competition.md | 시장/경쟁/생태계 | 실명 경쟁자 ≥3 + 비교 축 + 생태계 맵 | 경쟁자 실명·맵 존재 |
| 03 | 03-personas.md | 개발자/운영자 페르소나 | 핵심 페르소나 ≥1 상황·페인포인트·빈도 구체화 + 여정 맵 | 추상 인구통계만이면 실패 |
| 10 | 10-prd.md | 기능/우선순위/비기능 | 기능 ID 부여·P0/P1/P2·각 기능↔문제 추적 + 비기능 정량 | 추적표·우선순위 마인드맵 |
| 11 | 11-user-stories.md | 유저 스토리 | Epic→Story + GWT 수용기준 + 기능 ID 링크 | GWT 존재·링크 |
| 12 | 12-ia.md | TUI/CLI 정보구조 | 인터페이스 계층 트리 + CLI 명령 트리 + 화면 ID | IA 트리 존재 |
| 13 | 13-user-flows.md | 유저 플로우 | 핵심 플로우 + 분기 + 화면/기능 ID 링크 + 플로우차트·시퀀스 | 다이어그램·링크 |
| 20 | 20-wireframes.md | TUI 와이어프레임 | 화면당 1시안(ASCII/SVG) | STEP 7 검수 |
| 21 | 21-screen-spec.md | 화면 명세(키바인딩 정규출처) | 화면당 시안 + 핵심화면 4상태 + 명세 + 키바인딩 | 산문만이면 실패(G-g) |
| 30 | 30-functional-spec.md | 기능 동작 명세 | 입력/처리/출력/예외 + 분류·스캔 플로우차트 | 알고리즘 도시에 일치 |
| 31 | 31-erd.md | 인메모리 도메인 모델 | 구조체/enum 필드·관계 + Config 기본값 + ERD 다이어그램 | 필드 도시에 일치·DB부재 명시 |
| 32 | 32-api-spec.md | 외부 인터페이스 계약 | CLI/lib/시스템명령 3계약 + 출력 스키마 + 시퀀스 | 서브커맨드/플래그 도시에 일치 |
| 33 | 33-policy.md | 안전·프라이버시 정책 | 권한 매트릭스 + 마스킹 + fs미독 원칙 + 에스컬레이션 + 31 참조 | 중복정의 0·플로우차트 |
| 40 | 40-backlog.md | 완료 백로그 + 로드맵 | 구현완료 정리 + 로드맵(도시에 §12 근거만) + 간트 | 근거없는 기능창작 0 |
| 41 | 41-qa-testcases.md | QA 케이스 | 케이스(전제/입력/기대/우선) + 테스트 모듈 10종 매핑 + 커버리지 맵 | 매핑 존재 |
| — | INDEX.md | 가이드(최후) | 3티어+작업순서+역할번들+최소세트 | 신규콘텐츠 0 |
| — | index.html / NN.html / overview.html | HTML(Both) | 각 페이지 주요 섹션마다 인라인 SVG ≥수개·ASCII→SVG 변환·외부렌더러 0 | 텍스트/ASCII-only 변환이면 실패 |

## 6. 시장 & 생태계 맥락 요구사항 (Market & ecosystem context)

Generator는 **실재하는 프로세스 모니터/매니저**를 최소 다수 실명으로 호명하고 비교 축으로 비교해야 한다(02 문서). 후보(필요 시 추가):

- **범용 시스템/프로세스 모니터:** `htop`, `btop`, `top`, `glances`, `gotop`, macOS **Activity Monitor**(활성 상태 보기).
- **컨테이너/프로세스 매니저:** `ctop`(컨테이너), `pm2`(Node 프로세스 매니저), `systemd`/`launchd`(서비스 관리·간접 비교).

비교 축(예): 런타임 인지 여부 / 프레임워크 식별 / 포트→프로세스 매핑 / 시그널·graceful 종료 관리 / 트리 뷰 / 크로스플랫폼(mac·Linux·Win) / TUI+CLI 동시 / 단일 바이너리 설치. **"ntop은 의미 인지(semantic awareness) 축에서 범용 모니터와 갈린다"** 가 명확히 드러나야 한다.

**생태계(가치 네트워크) 분석 + 생태계 맵 필수.** 참여자/역할/인센티브/가치교환/의존:
- **공급측:** 메인테이너(greeun), 기여자.
- **수요측:** 개발자/운영자(채택자).
- **플랫폼/중개:** GitHub(코드/이슈/Stars/Sponsors), crates.io(`cargo install`), Homebrew, 릴리스 바이너리 배포.
- **보완재(complements):** 대상 런타임(Node/Python/Java/Deno/Bun/Ruby/PHP/.NET)과 프레임워크(Next.js/FastAPI/Spring Boot 등), 터미널 에뮬레이터, 의존 크레이트(sysinfo/ratatui/crossterm/clap/tokio/nix/windows-sys), 의존 시스템 명령(lsof/netstat/taskkill).
- **대체재(substitutes):** htop/btop/pm2/Activity Monitor/glances 등(위 경쟁자).
- **제도/규범:** MIT/OSI 라이선스, Rust 생태계 관행, 의존 크레이트 메인테이너.
- **(결제/인프라/데이터 파트너 치환):** CI(GitHub Actions), crates.io 레지스트리, GitHub Sponsors(후원 인프라). **결제 파트너 아님 — 후원/배포 인프라로 해석.**

생태계 맵은 **이해관계자/가치 네트워크 다이어그램**으로, 참여자 간 가치(코드·피드백·후원·배포) 흐름을 시각화(G-g, 외부 렌더러 0).

## 7. 비목표 (Non-goals)

- **코드 구현/수정 없음.** ntop은 이미 구현됨 — 본 작업은 문서화/역설계만.
- **결제·정산·take-rate·회원 DB·REST 엔드포인트 창작 금지** (= 즉시 실패).
- **도시에/코드에 근거 없는 기능·수치 창작 금지.** 로드맵 항목은 §12 근거 + "가설/예정"으로 표기.
- **fs 기반 탐지 제안 금지** — "파일시스템 안 읽음"은 설계 원칙이므로 이를 뒤집는 제안은 정책 위반.
- 실제 디자인 에셋(픽셀 목업·브랜드 그래픽) 제작은 범위 밖(문서 내 ASCII/SVG 시각화까지만).
- 마케팅 카피 작성 아님.
- 기술 스택/라이브러리 선택 강제 아님(이미 결정됨, 사실로 기술만).

## 8. 완료 정의 (Definition of Done)

아래가 모두 참이어야 한다:

- **모든 핵심 기능이 문제 정의의 특정 문제로 매핑**된다(기능↔문제 추적표, G-a).
- **핵심 페르소나 ≥1개가 상황·페인포인트(빈도/결과)** 까지 구체적이다(추상 인구통계만이면 실패, G-b).
- **차별화가 실명 경쟁자 ≥1과 명시 비교**(경쟁자명 + 비교 축)이고 모방난이도/해자 논거 포함(G-c).
- **모든 성공지표에 측정 가능한 수치 + 측정 방법(도구/주기/판정기준)**. OSS 채택 지표(Stars/다운로드/기여자/이슈 응답시간 등)와 제품 품질 지표(스캔 tick 지연, CPU delta 정확도, 메모리 보고 정확도 등)를 구분하고, **채택 목표치는 "가설"로 명시**(G-d).
- **MVP(=v0.2.0 출시 범위) / Out-of-scope(로드맵·제외) 명시적 분리** + 우선순위 근거(G-e).
- **플레이스홀더/TBD 0개**(G-f).
- **생태계 참여자 + 생태계 맵 포함**.
- **구조가 있는 모든 문서/섹션에 개념 적합 시각화**(다이어그램/인포그래픽) 임베드(G-g) — 텍스트 벽 0, 빈 다이어그램 0, 외부 렌더러(Mermaid 등) 의존 0, 가능하면 생성 이미지.
- **문서별 HTML은 각 주요 섹션의 대표/핵심 콘텐츠를 시각화**(페이지당 인라인 `<svg>` 여러 개), 기존 ASCII 다이어그램은 인라인-SVG로 변환(텍스트/ASCII-only 변환이면 실패).
- **쉬운 설명 우선**: 불가피한 기술/도메인 용어는 첫 등장 시 한 구절로 풀이(경영/후원 검토자도 읽힘).
- **디자인 품질 상식(UX 기본기)**: 불필요한 단계/입력 최소화, 계정/맥락으로 결정되는 것은 수동 선택 강요 안 함(ntop 맥락에선 TUI/CLI의 군더더기 화면·중복 단계 지양).
- **(도메인 추가) 모든 사실이 도시에/코드로 역추적 가능**하고, 도메인 재해석(§4-0) 6항목이 모두 지켜진다(결제/DB/REST 창작 0). 단일 출처 레지스트리(§4-3) 위반(중복·모순) 0.
- **INDEX.md**가 3티어·작업순서·역할번들·최소세트를 담고 신규 콘텐츠를 만들지 않는다.
