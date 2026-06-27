# Sprint 4 (g1b) Report — service-planning-harness

## Strategic Decision
**REFINE** — S1/S2 리서치 게이트는 이미 PASS(critique-research.md). 본 라운드는 두 리서치를 최종
문서(02·03)로 승계하면서 평가자의 **비블로킹 비고 3건을 구체적 수정으로 반영**하는 정제 작업이다.
구체 편집:
1. (S1-비고1) `02` §3-3에 경쟁자 **버전 주석표** 신설 — htop 3.x·btop++ 1.x·glances 3~4.x·top
   procps-ng 4.x/BSD·gotop 4.x·ctop 0.7.x·pm2 5.x·systemd 250+·OS 번들(Activity Monitor/launchd) +
   ntop 0.2.0. 검증성 보강 + 보수적 판정 사유 명시.
2. (S2-비고2) `03` §4에서 **P2(정태호) 빈도·결과를 P1 수준으로 보강** — 새벽 2시 배포 인시던트 한
   장면 + 4개 페인포인트에 구체 빈도(주 5~10회 SSH·매 정리·주 5회+)·결과(월 2~3회 2차 인시던트 재호출,
   야간 30분+ 추가) 부여.
3. (S2-비고1 = ★ 필수 정정) `03` 전반에서 **"좌측 목록 NAME/FRAMEWORK 컬럼" 옛 표기를 제거**하고
   코드-진실로 교체: TUI는 세로(목록 위/상세 아래), 목록 컬럼에 FRAMEWORK 없음, 프레임워크는 Info 탭 +
   CLI `list` 테이블 FRAMEWORK 컬럼에만 노출(도시에 §6). §1 화면 표기 원칙 블록 신설 + §3·§5·§6·§7-1
   여정맵까지 일관 적용.

## Deliverables produced (from sprint contract)
- `/Users/uni4love/project/workspace/211-withwiz/ntop/docs/02-market-competition.md` (494행) —
  경쟁/생태계 단일 출처.
- `/Users/uni4love/project/workspace/211-withwiz/ntop/docs/03-personas.md` (336행) — 페르소나 단일 출처.

## Verification I performed
모든 검증은 grep + 구조 점검으로 실제 실행해 관찰한 결과다.

**02-market-competition.md (체크 M-1~M-7):**
- M-1 생태계 7범주(공급/수요/플랫폼·중개/보완재/대체재/제도/후원·배포): PASS — §1-1~§1-7 표 전부 존재.
- M-2 생태계 맵(노드-엣지, 외부 렌더러 0): PASS — §2 ASCII, Mermaid는 "미사용" 부정문맥에만 등장(grep).
- M-3 비교 매트릭스(10 실명 × 8축 + 탐지 범위, ▶ ntop 강조): PASS — §3-2. RT·FW·PORT 동시 ✓는 ntop뿐
  명시(§3-2·§7).
- M-4 버전 주석: PASS — §3-3 11행.
- M-5 포지셔닝 쿼드런트(2축 ASCII): PASS — §4.
- M-6 시장/세그먼트(매출 산정 0): PASS — §5, 금액 없음·"가설"은 00으로 회부.
- M-7 갭/기회 + 해자 3논거 + 다이어그램: PASS — §6·§7.

**03-personas.md (체크 P-1~P-7):**
- P-1 3 페르소나 7항목: PASS — §3·§4·§5.
- P-2 P1·P3 구체 + P2 보강: PASS — §4 P2에 인시던트 장면 + 빈도/결과.
- P-3 원하는 결과→능력 매핑 + PRB 링크: PASS — §6.
- P-4 PRB-1~8 앵커표: PASS — §2.
- P-5 여정 맵: PASS — §7-1(지정 흐름 ①~⑦ 포함).
- P-6 가치 루프 + 문제→페르소나 매트릭스(+빈도): PASS — §7-2·§7-3·§7-4.
- P-7 FRAMEWORK 정정: PASS — §1 원칙 + 전체 일관, "좌측 목록 FRAMEWORK" 잔존 0(자체검증 인용 1건은
  의도적 정정 기록).

**공통 게이트/금지 grep(실행 결과):**
- 플레이스홀더/TBD/TODO: 0건.
- Mermaid·결제/정산/take-rate/DB/REST: 본문 등장은 전부 **부정/면책/치환** 문맥(미사용·아님·없다·치환·
  창작금지 준수)만 — 창작 0.
- 도구 호출 태그 누출(antml:/invoke/parameter): 0건.
- 구조: 두 파일 모두 상단 `## Sprint Contract (self-proposed checks)` + 하단 `## 자체 검증` 확인.

## Known limitations
- 경쟁자 버전(htop 3.x 등)은 외부 도구 사실이라 도시에로 역추적 불가 → "대략/계열" 표기 + 보수적 판정
  사유를 §3-3에 명시했다.
- 페르소나 빈도 수치(P1 주 10회·월 1~2회, P2 주 5~10회·월 2~3회 2차 인시던트 등)는 도시에에 없는 사용
  맥락 기반 개연 추정으로 라벨링했다(§1·§4). ntop 동작 인용은 전부 도시에 §번호 근거.
- 본 g1b는 02·03만 담당. 01-service-plan은 §7 해자(02) + 페르소나(03)를 단일 출처로 참조해야 하며,
  00-business-model이 수익화 "가설" 수치의 단일 출처다(본 문서들은 02 §1-7·§5에서 00으로 회부).

## How to review
- **G-c(02):** §3-2 매트릭스에 실명 경쟁자 10개 + 비교 축이 있고, §7이 "RT·FW·PORT 동시 ✓는 ntop뿐"으로
  명시 비교(일반론 아님)인지 확인.
- **G-b(03):** §3·§4·§5가 인구통계만이 아니라 상황·일과 한 장면·빈도/결과·우회법·원하는 결과까지
  구체인지, 특히 §4 P2가 P1 수준으로 보강됐는지 확인.
- **★ 코드-진실(03):** §1 화면 표기 원칙 + §7-1 ⑥단계가 "목록 위/상세 아래, 목록 컬럼에 FRAMEWORK 없음,
  FRAMEWORK는 Info 탭/CLI 테이블"로 되어 있는지(도시에 §6 대조).
- **G-g:** 두 문서의 모든 다이어그램이 ASCII이고 빈 다이어그램·외부 렌더러가 없는지.
- **단일 출처:** 02가 페르소나를 중복하지 않고, 03이 경쟁 매트릭스를 중복하지 않는지(상호 §-참조만).

READY_FOR_QA: 02-market-competition.md, 03-personas.md
