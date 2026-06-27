# Sprint 4 Report (g5) — service-planning-harness

## Strategic Decision
**REFINE** — 1라운드 신규 산출. g5 범위(`40-backlog` + `41-qa-testcases`)를 선행 산출물(10-prd F1~F21·
13-user-flows UF-1~6·21-screen-spec·30/31/32/33·03-personas PRB-1~8)과 실제 `tests/*.rs`(115개 테스트) 위에
작성. 로드맵은 codebase-facts §12(미구현 흔적 5건) + 선행 문서의 코드 실상 정직 발견(g4a/g4b 리포트 5건)
에서만 도출. 피벗 사유 없음(역설계 범위 안에서 통합).

## Deliverables produced (from sprint contract)
- `docs/40-backlog.md` — 완료 백로그(v0.2.0 = F1~F21 전부 ✅, Epic 6종) + 향후 로드맵(R1~R10, §12 5건 +
  정직 발견 5건) + 우선순위(High/Med/Low)·의존 그래프 + 완료/예정 간트(완료 강조)·우선순위 매트릭스.
- `docs/41-qa-testcases.md` — 핵심 플로우 케이스(UF-1~6, TC-1.*~6.*) + 기능 케이스(TC-7~11) + 엣지/예외
  (TC-E1~E14) + 테스트 모듈 10종 매핑(실제 함수명·115 테스트) + 커버리지 갭 + 커버리지 맵 마인드맵.

## Verification I performed (실측 결과)

**40-backlog (B-1~B-7 전부 PASS):**
- B-1 완료 백로그: §2 Epic A~F에 F1~F21 전부 ✅(완료), 우선순위는 10-prd §3 인용(재정의 0). §2-2 완료
  현황 막대(21/21).
- B-2 로드맵 = §12+발견만: §3-2 (가) R1~R5 = §12 5건(Express/Fastify/Koa/Hapi 규칙 미등록·include_bun
  deprecated·Windows 시그널·외부명령 의존·로그 stdout), (나) R6~R10 = 정직 발견 5건. 각 행 근거 출처 +
  `[제안]`/`[가설]` 라벨. **그 외 항목 0**(창작 0).
- B-3 우선순위+의존: §3-2 우선·의존 열 + §4 가치×노력 매트릭스 + §5 의존 그래프(대부분 독립, R7→R1 약한
  의존만).
- B-4 fs 미독 미반전: §3-1 영구 제외 박스(파일시스템 탐지·웹 GUI·프로세스 감독·전역 그래프) + §6-2 제외
  영역(좌표 없음).
- B-5 시각화: §6-1 완료/예정 간트(완료 구간 `████` 강조 + 로드맵 `░░░`)·§6-2 우선순위 매트릭스·§2-2 막대·
  §6-3 가치 루프, 전부 순수 ASCII(grep mermaid=0).
- B-6 단일 출처: 기능/우선=10·MVP=01·값=31·정책=33·계약=32·지속가능성=00 참조, 상태만 소유.
- B-7 사실 정정: R6 health() 메모리 인자 MB 버그(30 §2-12)·R7 version 항상 `-`(21 §3-6)·R8 CLI 미마스킹·
  R9 종료코드 Ok(())·R10 과마스킹(g4b) 로드맵화.

**41-qa-testcases (T-1~T-7 전부 PASS):**
- T-1 플로우 케이스: §2 UF-1~6 각 표(전제/입력/기대/우선), 13-user-flows 단계와 정합.
- T-2 기능 케이스: §3 분류(TC-7)·네트워크(TC-8)·로그(TC-9)·설정(TC-10)·health/types(TC-11).
- T-3 엣지/예외: §4 TC-E1~E14 — playbook 명시 엣지(EPERM·lsof/netstat 부재·좀비/고아·comm 16자 절단·npx
  오분류 회피) 전부 + 부분설정·버퍼상한·미지 시그널·CLI env 평문.
- T-4 ★ 모듈 10종 매핑: §5-1 표 — framework_test 24·scanner_test 2·tree_test 8·killer_test 7·network_test
  13·log_test 13·cli_test 11·config_test 5·filter_test 9·types_test 23(=115). 각 모듈 커버 F#/UF# + 실제
  함수명. `tests/*.rs` grep으로 함수명·카운트 실측.
- T-5 커버리지 갭: §5-2 갭 표 8영역(TUI 렌더·키 핸들링·실제 종료 동작·platform FFI·env 마스킹·명령부재·
  정렬 비교자·health MB 분기) + §6-3 자동 vs 수동 매트릭스. **killer_test가 시그널 메타데이터만 덮고 실제
  graceful/force/역순/EPERM은 안 덮는다는 점 명시.**
- T-6 시각화: §6-1 커버리지 맵(기능군→모듈 마인드맵, ✅/⚠/✗갭)·§6-2 우선순위·종류 분포·§6-3 갭 매트릭스,
  전부 ASCII.
- T-7 단일 출처: 기능=10·플로우=13·키=21·값=31·정책=33·로드맵=40 참조, 케이스만 소유.

**공통 게이트:** G-f(placeholder/TBD/TODO/FIXME grep = 0), G-g(40: 간트+매트릭스+막대+루프 / 41: 마인드맵+
분포+갭 매트릭스, 외부 렌더러 0 — mermaid grep=0), Sprint Contract(상단)+자체 검증(하단) 양 문서 grep 확인.
**도구 호출 태그(`</content>`·`</invoke>`·`antml:`) 누수 0**(tail+grep 확인).

## Known limitations
- 로드맵 노력 산정(High/Med/Low)·순서(단계 1/2/3)는 코드 결합도·근거 강도 기반 정성 추정 — 공수 시간·릴리스
  날짜·버전 번호는 사실이 아니므로 적지 않음(`[가설]` 라벨).
- 41의 `[M] 갭` 케이스(실제 종료·EPERM·TUI 렌더·env 마스킹·명령부재)는 자동 테스트가 없어 수동/통합 영역
  표기일 뿐, 본 패키지는 문서화만(실제 테스트 코드 미추가, spec §7 비목표).
- 40의 코드 수정형 로드맵(R6 버그·R8·R9 등)은 "고쳐야 함"의 사실·근거만 기록, 실제 코드 수정 안 함(역설계
  범위).
- 테스트 수치(115)는 현재 `tests/*.rs` 기준 — 코드 변경 시 갱신 대상.

## How to review
- **40:** §3-2 로드맵 R1~R10이 전부 codebase-facts §12 또는 g4a/g4b 정직 발견에 역추적되는지(근거 열),
  그 밖 항목 0인지(창작 0) 확인. §3-1이 fs 기반 탐지를 영구 제외로 두는지(원칙 미반전). §6-1 간트가 완료
  구간을 강조(`████`)하고 로드맵을 `░░░`로 구분하는지. MVP 경계·우선순위를 01/10 인용만 하고 재정의
  안 하는지(B-6).
- **41:** §5-1 모듈 매핑이 실제 `tests/*.rs` 함수명과 일치하는지(framework 24·types 23·… 115), §5-2 갭이
  killer_test의 메타데이터-only·TUI 미테스트·platform FFI를 정직 기록하는지. §4 엣지가 playbook 명시 5종을
  전부 포함하는지. §2 UF 케이스가 13-user-flows 단계와 정합하는지.
- **교차:** 40 R6(health MB)·R7(version `-`)·R8(CLI 미마스킹)이 30 §2-12·21 §3-6·33/g4b 발견과 일치하는지,
  41 TC-11.6·TC-1.5·TC-E14가 같은 발견을 케이스/갭으로 반영하는지(40↔41 정합).

READY_FOR_QA: 40-backlog.md, 41-qa-testcases.md
