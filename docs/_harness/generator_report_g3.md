# Sprint 4 / g3 Report — service-planning-harness (ntop)

## Strategic Decision
**REFINE** — 첫 라운드(신규 산출물). 기존 방향 유지, S3 리서치(`research-s3` §5~§8)를 충실히
승계하되 **실제 소스코드로 코드-진실을 정정·강화**했다. 이번 라운드 핵심 작업 5가지:
1. `20-wireframes.md`(12 TUI 화면 ASCII 시안) + `21-screen-spec.md`(키 카탈로그 정규 출처 + 화면 명세
   16종 + 핵심 화면 4상태) 신규 작성.
2. **연구 시안 대비 코드-진실 정정**: ① TUI 목록 NAME 컬럼은 별도 FRAMEWORK 컬럼이 아니라 탐지 시
   `(Next.js)`/`[Python]` **인라인 배지**(process_list.rs L121–132) ② 체크박스는 `□`가 아니라
   `[ ]`/`[x]` + 헬스 `●` ③ TAB-INFO는 2컬럼이 아니라 **세로 18줄 key:value**(info_tab.rs) ④
   다이얼로그는 한글 의역이 아니라 **영문 실문자열**(kill_dialog.rs·signal_picker.rs·help_dialog.rs).
3. 키 바인딩 카탈로그를 `ui.rs`+`status_bar.rs`+`help_dialog.rs`로 **3중 코드-검증**해 `21` §2를 정규
   출처로 확정(`+/-` 1~60s 범위, BackTab, Help 스크롤키 포함).
4. 핵심 화면(V-LIST/TAB-LOG/TAB-NET) **4상태 개별 미니 시안** + 에러 트리거(EPERM·lsof/netstat 부재)
   매트릭스.
5. HUMAN_CHECKPOINT 배너 + 검수 포인트 6항(폭·정렬·문자열 충실도) 명시.

## Deliverables produced (from sprint contract)
- `docs/20-wireframes.md` — 저충실도 TUI 시안 12종(W-1~W-12: V-MAIN·TAB-INFO/LOG/NET/ENV·DLG-KILL/
  KILLTREE/SIGNAL/FORCE/HELP·V-FILTER·V-EMPTY) + CLI 참고 1컷. HUMAN_CHECKPOINT 배너·검수 포인트.
- `docs/21-screen-spec.md` — ★키 바인딩 카탈로그(정규 출처, 5그룹+컨텍스트 하단바) + 화면 명세 16종
  (목적/구성요소/상호작용/상태/예외/연결 기능10·화면12·데이터31·인터페이스→32) + 핵심 화면 4상태
  (V-LIST·TAB-LOG·TAB-NET) + 에러 트리거 매트릭스.
- `docs/_harness/generator_report_g3.md` — 본 보고서.

## Verification I performed (관찰 바 + 게이트, 실제 관찰 결과)
**20-wireframes 관찰 바("화면당 1 저충실도 시안"):**
- 12 TUI 화면 전부 ASCII 프레임 존재(§3 W-1~W-12) — **PASS**.
- 세로(상/하) 레이아웃·목록 컬럼 코드-진실·실문자열 충실도 — **PASS**(§0 정정 + 소스 대조).
- HUMAN_CHECKPOINT_REQUIRED 배너 상단 + 검수 포인트 6항 — **PASS**.

**21-screen-spec 관찰 바("화면당 시안+4상태+명세+키 정규 출처"):**
- 키 카탈로그 5그룹+하단바, ui.rs/status_bar.rs/help_dialog.rs 일치 — **PASS**(§2).
- 화면 16종 명세 9요소 충족 — **PASS**(§3-1~§3-16).
- 핵심 화면 4상태 개별 미니 시안(V-LIST/TAB-LOG/TAB-NET) — **PASS**(§3-4·§3-7·§3-8 + §4 매트릭스).
- 에러 트리거(EPERM·lsof/netstat 부재·로그 접근 실패) 명시 — **PASS**(§4).

**게이트:**
- **G-g**(인포그래픽 우선, 외부 렌더러 0): 두 문서 전부 순수 ASCII, Mermaid 0, 산문-only 화면 0 — **PASS**.
- **P-1**(와이어프레임 휴먼 체크포인트): 20 상단 배너 + 21 §0 위 P-1 배너, 검수 포인트 제시 — **PASS**.
- 단일 출처(spec §4-3): 키=21 소유, 기능=10·화면=12·플로우=13·데이터=31·알고리즘=30·계약=32·정책=33
  참조(재정의 0) — **PASS**.
- 창작 금지(spec §7): 결제/DB/REST/로그인/설정 마법사 0, 실제 위젯 렌더 화면만 — **PASS**.

## Code facts I corrected vs research-s3 (정직 보고)
연구 산출물 `research-s3` §8 와이어프레임은 일부를 의역/근사했으나, 실제 소스를 읽어 다음을 정정했다
(모두 파일:라인 근거, 창작 아님):
- 목록 NAME 인라인 프레임워크 배지(process_list.rs L121–132) — 연구 시안은 NAME만 표기.
- 체크박스 `[ ]`/`[x]`(L92) — 연구 시안 `□`.
- TAB-INFO 세로 18줄 key:value(info_tab.rs L30–52) — 연구 시안 2컬럼.
- 다이얼로그 영문 실문자열(kill_dialog.rs / signal_picker.rs / help_dialog.rs) — 연구 시안 한글 의역.
- 스피너 프레임 `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`(empty_state.rs L9), empty=loading 단일 위젯.
- net_tab 헤더 `LOCAL/REMOTE/STATE`(net_tab.rs L23), env 마스킹 `********` 8자(env_tab.rs L48).

## Known limitations
- **시각 폭/정렬은 약 84칸 예시** — 실제는 ratatui가 터미널 폭에 신축(넓이>100/≤100 분기). P-1 검수 대상.
- **CLI 화면 명세/출력 스키마는 본 그룹 범위 밖** — `32-api-spec`(g4)가 소유. 20은 참고 1컷만, 21은 TUI
  16종만 명세.
- **메인 W-1의 Info 탭은 공간상 압축 표기** — 정확본은 W-2(세로 18줄).
- 레이아웃 좌/우 vs 상/하 불일치는 코드(상/하)를 채택, 도시에 §6 정정 권고는 research-s3 §10에서 이미 회부됨.

## How to review (Evaluator 가이드)
- **G-g**: 20·21에서 Mermaid/외부 렌더러 0, 모든 화면이 ASCII 시각 동반인지 grep("```", "mermaid").
- **P-1**: 20 상단 `HUMAN_CHECKPOINT_REQUIRED` 배너 + §2 검수 포인트, 21 P-1 배너 존재 확인.
- **키 정규 출처**: 21 §2가 12-ia/13-user-flows의 "키 정규 출처=21" 참조와 일치하는지, ui.rs 키와 대조.
- **4상태**: 21 §3-4/§3-7/§3-8에 empty/loading/normal/error 개별 미니 시안 4개씩 + §4 매트릭스 ★트리거.
- **코드-진실**: 세로 레이아웃·FRAMEWORK 컬럼 없음(NAME 인라인)·체크박스 `[ ]`/`[x]`가 20·21 §0에 일관.
- **단일 출처**: 21이 데이터값(1000줄/10s/3s)을 31 참조로만 인용하는지(중복정의 0).

READY_FOR_QA: 20-wireframes.md, 21-screen-spec.md
