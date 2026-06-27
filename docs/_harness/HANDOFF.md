# HANDOFF — service-planning-harness (ntop 역설계 기획 패키지)

> 새 세션이 이 작업을 이어받기 위한 인수인계 문서. 이 파일 + `spec.md` + `sprint-playbook.md` + `docs/*.md` 16종을 읽으면 중단 지점부터 재개 가능.

## 0. 무엇을 하는 중인가
- 스킬: `/service-planning-harness` (Planner→Generator→Evaluator GAN 하니스, Full tier).
- 작업: **ntop 코드(Rust TUI+CLI 프로세스 모니터, MIT OSS, v0.2.0)를 역설계해 기획 문서 패키지 생성.**
- 고정 파라미터: **Mode 5(16문서 풀 패키지) · HTML=Both · 출력 언어=한국어.**
- 핵심 원칙: ntop은 백엔드/DB/결제/HTTP API/회원 없는 로컬 OSS 도구 → 16문서를 도메인 재해석(spec §4-0). **결제/정산/DB/REST 창작 = 실패.** 모든 사실은 `docs/_harness/codebase-facts.md`(원천) + 실제 코드에서만.

## 1. 완료된 것 (✅)
- **코드 분석 → `docs/_harness/codebase-facts.md`** (원천 dossier, 정정 2건 반영: TUI 레이아웃 세로 상/하, 목록에 FRAMEWORK 컬럼 없음=NAME 인라인).
- **Planner → `docs/_harness/spec.md` + `sprint-playbook.md`** (Mode5 16문서 골격·단일출처 레지스트리 §4-3·교차추적성 §4-2).
- **리서치 S1/S2/S3 → `docs/_harness/research-s1.md`(시장·경쟁·생태계) / research-s2.md(문제·PRB-1~8·페르소나 P1/P2/P3) / research-s3.md(IA·플로우·화면·와이어프레임).**
- **연구 게이트 PASS → `docs/_harness/critique-research.md`** (S1 C2=5/5, S2 C1=5/5, S3 C1-part=5/5; G-a/b/c/g 통과; S3 와이어프레임 P-1 HUMAN_CHECKPOINT_REQUIRED).
- **S4 16문서 전부 생성 완료 → `docs/00~41.md`** (g1a:00,01 / g1b:02,03 / g2a:10,11 / g2b:12,13 / g4a:30,31 / g4b:32,33 / g3:20,21 / g5:40,41). 누출 태그(`</content>`/`</invoke>`) 전량 정리됨.

### 16문서 목록 (docs/)
```
00-business-model 01-service-plan 02-market-competition 03-personas
10-prd 11-user-stories 12-ia 13-user-flows
20-wireframes 21-screen-spec
30-functional-spec 31-erd 32-api-spec 33-policy
40-backlog 41-qa-testcases
```

## 2. 완료 (✅ STEP 8 전달)
- **STEP 5/6**: 라운드1 FAIL(Blocking#1 kill 모순) → REFINE → 라운드2 **PASS**(`critique-final-r2.md`).
- **STEP 7**: 사용자 시각 검수 통과. `13` §8-1 상태기계 ASCII를 ASCII-only 박스(한글 박스밖 주석)로 재작도해 전각폭 정렬 흠 제거.
- **STEP 7.6 INDEX.md**: 3티어(T1 9·T2 2·T3 5)·작업순서·역할번들·최소세트(6) 생성.
- **STEP 7.5 HTML(Both)**: 16 `NN-name.html`(ASCII→인라인 SVG 변환, 페이지당 SVG 다수, per-screen 20=14·21=23 프레임) + 허브 `index.html`(INDEX 반영) + 대시보드 `overview.html`(SVG 12, 시각 카탈로그). 공통 디자인 시스템 `docs/_harness/_html-base.css` 전 HTML 인라인. 외부 렌더러/CDN 0, 누출 태그 0, 깨진 내부 링크 0. (03-personas는 1차 부실 → 재렌더로 svg=9·913줄 정상화.)
- **최종 산출**: `docs/` 16 md + INDEX.md + 16 NN-name.html + index.html + overview.html. 원본 md 전량 보존.

### STEP 5/6 결과 (✅ PASS)
- 라운드 1 `critique-final.md` = **FAIL**(Blocking #1: TUI kill→force 에스컬레이션 모순, `11/13/20/21` ↔ `30/32/33/40/41`+코드. C3 2×축 3/5).
- 수정: Generator REFINE 1라운드 → `13` §8을 TUI①/CLI② 2-lane 재작도, `11/20/21` 코드 정렬, `40` M1(죽은 코드)·M2(NAME 배지) 로드맵화, `12`§0 배지 정정. 코드 미수정(문서만). 보고서 `generator_report_fix1.md`.
- 라운드 2 재평가 `critique-final-r2.md` = **PASS**(C3 3→5, 전 §8 게이트 PASS, 누출/플레이스홀더 0, 신규 창작 0, 교차 ID 무손상). FINAL VERDICT: PASS.

## 3. 남은 단계 (⏳) — 순서대로
1. ~~STEP 6 판정 분기~~ ✅ PASS 확정.
2. **STEP 7 와이어프레임 휴먼 체크포인트 (진행 중)**: `20-wireframes.md`·`21-screen-spec.md` 화면 시안 + `13` §8-1 상태기계 ASCII 정렬 → **사용자 검수·승인 대기**. 승인 전 HTML 렌더 금지. 시각부만 `HUMAN_CHECKPOINT_REQUIRED`, 텍스트/사실부는 PASS.
3. **STEP 7.5 HTML 렌더 (Both)** — `references/html-doc-template.md` + `html-visual-template.md` 따라:
   - 각 `NN.md` → `docs/NN-name.html` (읽기용; **본문 ASCII 다이어그램을 인라인 SVG로 변환** + 섹션별 인포그래픽; 페이지당 인라인 `<svg>` 여러 개; ASCII-only 변환이면 실패 G-g).
   - `docs/index.html` (허브; INDEX.md의 중요도·순서·역할 반영).
   - `docs/overview.html` (종합 다이어그램 대시보드; 마인드맵·생태계맵·경쟁매트릭스·플로우·ERD·간트 등 카탈로그).
   - **외부 렌더러(Mermaid)·CDN 0, 자체완결 인라인 SVG/CSS. 신규 콘텐츠 0(기존 md 변환·시각화만).** 와이어프레임은 STEP 7 승인 후 렌더.
4. **STEP 7.6 INDEX.md (최후 생성)** — `references/mode-templates.md` "INDEX.md 생성 규칙": ① 3티어 중요도(T1 필수/T2 조건부/T3 보조) ② 의존 기반 작업순서 ③ 역할별 번들(개발/QA/디자인/PO/메인테이너·후원검토) ④ 최소 시작 세트. 신규 콘텐츠 0. (sprint-playbook S4-g7에 ntop 티어 가이드 있음: T1=01·10·13·21·30·31·32·40, 33은 안전 정체성→T1 승격 권장.)
5. **STEP 8 전달** — docs/ 16 md + INDEX.md + HTML(NN.html + index.html + overview.html). 원본 md 항상 보존.

## 4. 새 세션 재개 절차
```
1) ls docs/*.md          # 16개 확인
2) cat docs/_harness/critique-final.md  # 있으면 FINAL VERDICT 확인 / 없으면 최종 Evaluator 재디스패치
3) PASS면 → 사용자에게 20-wireframes.md·21-screen-spec.md 검수 요청(STEP 7)
4) 승인 후 → HTML 렌더(g6) → INDEX.md(g7) → 전달
```
- 평가/생성 에이전트 재디스패치 시 프롬프트 패턴은 이전 generator/evaluator 디스패치와 동일(각 `docs/_harness/generator_report_*.md` 참고). reset≠compaction 원칙.

## 5. 주의·정정 사항 (반드시 반영)
- **코드-진실(정정 완료)**: TUI 레이아웃 **세로 상/하**(목록 위 55%/상세 아래 45%), TUI 목록에 **별도 FRAMEWORK 컬럼 없음**(framework는 NAME 셀 인라인 `(Next.js)`/`[Python]` + Info 탭 + CLI list 테이블). CLI `list` 테이블엔 FRAMEWORK 컬럼 있음.
- **종료 정책(정직 기술)**: TUI `KillConfirm`은 SIGTERM **직접 전송**, 자동 graceful→force 에스컬레이션은 **CLI 단건 경로 전용**, TUI 강제는 `ForceKillPrompt` 별도 모달.
- **40-backlog 정직 발견**(로드맵으로 기술, 작동 기능 아님): `health()`가 RSS를 MB로 환산해 from_cpu_mem에 넘김(버그), `framework_version` 항상 None, CLI info의 env 미마스킹 등.
- **단일출처 레지스트리**(중복정의 금지): 돈/지속가능성=00, 가치/MVP/지표=01, 경쟁/생태계=02, 페르소나=03, 기능ID/우선순위=10, 구조체+Config기본값=31, 인터페이스 계약=32, 안전/프라이버시=33, 키바인딩=21. 같은 값(refresh_interval=3·graceful_timeout=10·MAX_BUFFER_LINES=1000)은 정규출처에만, 나머지는 참조.
- **누출 태그 주의**: Write 도구가 파일 말미에 `</content>`/`</invoke>` 누출 빈번 → 생성 후 `grep -rlE '</content>|</invoke>' docs/*.md`로 스캔, perl로 제거.
- **도구 호출 토큰**: 호출 블록은 반드시 `antml:invoke`/`antml:parameter`로 시작(앞에 다른 단어 금지 — 디스패치 누락 원인).

## 6. 부수 작업(완료)
- claude-hud `agents-line.ts` 패치: `MAX_AGENTS_SHOWN` 3→8, `MAX_RECENT_COMPLETED` 2→3 (`~/.claude/plugins/cache/claude-hud/claude-hud/0.1.0/src/render/`). 플러그인 업데이트 시 덮어쓰임 주의.
</content>
