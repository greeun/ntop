# Sprint 4 (g2b) Report — service-planning-harness (ntop)

> 산출물: `docs/12-ia.md`, `docs/13-user-flows.md` (Mode 5, 그룹 g2 정의 단계의 후반 2종).
> 사실 원천: `codebase-facts.md`(도시에) + 실제 소스코드 + 선행 산출물(`research-s3`·`10-prd`·`11-user-stories`).

## Strategic Decision
**REFINE** — 첫 생성 라운드(이전 critique 없음). spec §4-0/§4-2/§4-3, playbook §S4-g2 관찰 바, mode-templates(12-ia=IA 트리·13=플로우차트+시퀀스), research-s3(IA 트리·F1~F6 플로우·상태/시퀀스), 10-prd(F1~F21·PRB)를 충실히 승계. 이번 라운드에서 한 핵심 판단 3가지:
1. **플로우 ID 충돌 회피:** research-s3는 플로우를 "F1~F6"로 불렀으나 10-prd가 "F1~F21"을 **기능 ID**로 소유 → 같은 문서에서 충돌. 플로우를 **`UF-1~UF-6`**로 재명명하고 §1-1에서 research-s3 F1~F6에 1:1 대응 명시(추적성 유지).
2. **코드-진실 일관 반영:** 세로(상/하) 레이아웃 + TUI 목록 FRAMEWORK 컬럼 없음을 12 §0·§3·§4·§6과 13 §0·UF-1에 일관 반영.
3. **단일 출처 준수:** 키 바인딩은 21-screen-spec(현재 research-s3 §7)을, 기능은 10-prd를, 문제는 research-s2를, Config/enum은 31을 **참조만** 하고 재정의하지 않음(spec §4-3).

## Deliverables produced (from sprint contract)
- `/Users/uni4love/project/workspace/211-withwiz/ntop/docs/12-ia.md` — 인터페이스 정보구조(IA). TUI 레이아웃 계층(세로) + 통합 IA 트리(TUI+CLI) + CLI 명령 트리(플래그) + 화면 ID 21종 배정·노출 경로 표 + 색·역할 범례 + IA 특성 분석. (389줄)
- `/Users/uni4love/project/workspace/211-withwiz/ntop/docs/13-user-flows.md` — 유저 플로우. UF-1~UF-6 플로우차트 + kill graceful→force 상태 다이어그램 + 시퀀스 다이어그램 + 플로우×화면×기능×PRB 추적표. (429줄)

## Verification I performed
12-ia.md (I-1~I-8 자체검증, §자체검증 표):
- **I-1 TUI 레이아웃 계층** PASS — §3 세로 다이어그램(BAR-TOP/V-LIST 55%/V-DETAIL 45%·탭4/BAR-BOTTOM + V-FILTER·DLG-* 오버레이), 역할 태그·진입키·기능 ID 부기.
- **I-2 IA 트리(TUI+CLI, 역할/색)** PASS — §4 통합 트리 + §2 색 범례(코드 Color 근거).
- **I-3 CLI 명령 트리** PASS — §5 `ntop`→TUI + 5명령 + 플래그(`--json`/`--format`/`--tree`/`--signal`/`--all`/`--no-confirm`)·필수성·출력 요지.
- **I-4 화면 ID + 노출 경로** PASS — §6-1 21행 표(TUI 16 + CLI 5: ID·종류·역할·부모·진입 키/모드/명령·위젯·기능·PRB) + §6-2 도달 다이어그램.
- **I-5 색·역할 범례** PASS — §2(크롬/패널/탭/모드/모달/빈로딩 + 색 의미 임계값, status_bar.rs/empty_state.rs 근거).
- **I-6 사실 정정** PASS — §0 박스 + §3·§4·§6 세로 레이아웃·FRAMEWORK 컬럼 없음 일관.
- **I-7 G-g(외부 렌더러 0)** PASS — grep `mermaid` 0건, 전부 ASCII.
- **I-8 단일 출처** PASS — 화면 ID만 소유, 키/기능/문제 재정의 0(참조 표기).

13-user-flows.md (L-1~L-8 자체검증):
- **L-1 플로우 6종 플로우차트** PASS — §2~§7 UF-1~UF-6 ASCII, 분기 `◇`.
- **L-2 단계→화면/기능/PRB 링크** PASS — 단계 라벨 + §9-1 추적표(17행) + §9-2 PRB-1~8 전부 커버 확인.
- **L-3 상태 다이어그램** PASS — §8-1 상태기계(Idle→Confirming→Sending SIGTERM→Polling 200ms→Terminated/TimedOut/PermissionDenied/AlreadyDead→[DLG-FORCE/자동]→SIGKILL→Idle), GracefulResult 5분기.
- **L-4 시퀀스 다이어그램** PASS — §8-2 User↔App↔ProcessKiller↔OS↔Process, 경우 A/B/C + Windows taskkill 분기 + TUI/CLI 에스컬레이션 차이.
- **L-5 분기 명시** PASS — 필터 Enter/Esc·graceful 5분기·DLG-FORCE Enter/Esc·CLI 확인.
- **L-6 사실 정정** PASS — §0 표기 규약 + UF-1(프레임워크는 TAB-INFO에서).
- **L-7 G-g** PASS — grep `mermaid` 0건, 전부 ASCII.
- **L-8 단일 출처** PASS — 플로우만 소유, 나머지 참조.

기계 점검(bash):
- 스트레이 도구 태그(`antml`/`invoke`/`content`/`parameter`) — 최초 Write 시 말미에 `</content></invoke>` 누출 발견 → **head 절단으로 제거 완료**, 재grep 0건(CLEAN).
- 플레이스홀더(`TBD`/`TODO`/`FIXME`/`Mermaid`) 0건.
- `## Sprint Contract` 상단(12:L29, 13:L35) + `## 자체 검증` 하단(12:L367, 13:L406) 존재.
- 화면 ID 21종 모두 12-ia에 등장(BAR-TOP/BOTTOM·V-MAIN/LIST/DETAIL/FILTER/EMPTY·TAB-INFO/LOG/NET/ENV·DLG-KILL/KILLTREE/SIGNAL/FORCE/HELP·CLI-LIST/KILL/INFO/LOG/CONFIG).

## Known limitations
- **키 바인딩 정규 출처는 21-screen-spec**(아직 미작성, 현재는 research-s3 §7). 12/13은 진입 키를 도달성/단계 표기용으로 인용만 함 — 정규 키표 미소유(spec §4-3 준수). 21 작성 시 키 표기가 21을 가리키도록 정렬되어 있음.
- **레이아웃 좌/우 vs 상/하:** 도시에 §6 일부 표기와 코드 불일치 → 코드(상/하) 채택. 메인테이너 정정 권고는 research-s3 §10에서 회부됨(본 g2b는 코드-진실 채택).
- **플로우 ID 재명명(UF-n):** research-s3는 F1~F6로 불렀음. 충돌 회피를 위해 UF-1~UF-6 사용 + 1:1 매핑 명시. Evaluator가 research-s3 라벨 강제를 원하면 §1-1 매핑으로 재정렬 가능.
- CLI 출력 스키마 상세(JSON 필드·CSV 9컬럼·table 컬럼)는 32-api-spec 단일 출처 — 본 문서는 요지만.

## How to review
- **G-g(시각화):** 12 §3 레이아웃·§4 IA 트리·§5 CLI 트리·§6-2 도달 / 13 §2~§7 플로우차트·§8-1 상태기계·§8-2 시퀀스 — 전부 ASCII, 외부 렌더러 0. (`grep -i mermaid` 0건)
- **교차 추적성(spec §4-2):** 12 §6-1(화면↔기능 ID↔PRB) + 13 §9-1/§9-2(플로우 단계↔화면 ID↔기능 ID↔PRB, PRB-1~8 전부 커버). 화면 ID는 12가 소유, 기능 ID는 10이 소유, 키는 21이 소유 — 재정의 0 확인.
- **코드-진실:** 12 §0/§3, 13 §0 — 세로 레이아웃·목록 FRAMEWORK 컬럼 없음. process_list.rs 컬럼([✓]PID·NAME·PORT·THR·CPU·MEM·USER·STS·UPTIME)과 일치.
- **창작 금지:** 결제/정산/DB/REST/회원 화면·플로우 0 — 실제 TUI 뷰·CLI 명령·killer 로직만.

READY_FOR_QA: 12-ia.md, 13-user-flows.md
