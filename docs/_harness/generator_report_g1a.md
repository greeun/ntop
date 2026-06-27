# S4 g1a Report — service-planning-harness (ntop)

## Strategic Decision
**REFINE** — 첫 라운드 g1a 생성. critique 부재(신규), research 게이트는 전부 PASS(critique-research.md).
연구 단계 평가의 **유일한 S4 이월 필수 수정**(critique-research §종합: "좌측 목록 NAME/FRAMEWORK 컬럼"
표기를 "상단 목록(FRAMEWORK 없음) + Info 탭/CLI 테이블"로 정정, 레이아웃 좌/우 → 세로 상/하)을
선반영했다. 이번 라운드에 반영한 구체 편집 5건:
1. 01 §3에 **사실 정정 박스** 추가 — TUI 상단 목록 컬럼에 FRAMEWORK 없음(Info 탭·CLI `list` 테이블만),
   레이아웃 세로(위 목록 55%/아래 상세 45%).
2. 01 §4 가치 루프·플로우 설명을 **세로 레이아웃** 기준으로 기술.
3. 00의 필수 면책 문구를 spec 지정 문장 그대로(**"정산/take-rate/결제 없음 — 후원·배포 흐름으로 해석."**)
   삽입.
4. 단일 출처 경계 확정 — 00=지속가능성/수익화/배포/퍼널, 01=가치/MVP/지표/차별화. 상호 참조만(중복정의 0),
   경쟁/생태계는 02, 페르소나는 03으로 위임.
5. 성공지표를 **제품 품질(사실·측정 가능) vs OSS 채택(전부 가설)** 2부로 분리하고 채택 목표치를 본 01 §6
   단일 출처로 두되 사실 숫자 0.

## Deliverables produced (from sprint contract)
- `/Users/uni4love/project/workspace/211-withwiz/ntop/docs/00-business-model.md` — OSS 채택·지속가능성
  단일 출처(비용 구조·배포 채널 비교·채택 퍼널·수익화 가설·지속가능성 구조·기여/후원 가치 흐름도).
- `/Users/uni4love/project/workspace/211-withwiz/ntop/docs/01-service-plan.md` — 가치/MVP/지표/차별화
  단일 출처(7섹션 + 지속가능성 링크).
- `/Users/uni4love/project/workspace/211-withwiz/ntop/docs/_harness/generator_report_g1a.md` — 본 보고서.

## Verification I performed (관찰 바 + §8 게이트, 실측 결과)

**00-business-model (playbook g1 관찰 바):**
- 배포 채널 ≥3 비교 — **PASS**: §3에 4채널(cargo `--git`/crates.io/Homebrew/GitHub Releases) ×
  설치난이도·업데이트·대상사용자·선결조건·상태(확정/가설) 매트릭스.
- 채택 퍼널 — **PASS**: §4 인지→설치→상시사용→기여/후원 4단계 다이어그램 + 단계별 이탈/레버 표.
- 수익화 "가설" 표기 — **PASS**: §5 표 전 행 "가설", Pro/Team "가설(미구현)" 명기. grep "가설"=30+ 회.
- 정산/take-rate 0 — **PASS**: 면책 문구(spec 지정 문장 포함) + §7 흐름도에 결제 화살표 없음.
  grep로 결제/정산 용어는 전부 부정·면책 맥락임 확인.

**01-service-plan (playbook g1 관찰 바 + §8 게이트):**
- 7섹션 + MVP(=v0.2.0)·Out-of-scope 분리 — **PASS(G-e)**: §1~§7 + §8 링크. grep로 `^## [1-7]\.` 7개
  + §8 확인. §5-1 포함(P0/P1/P2 근거)·§5-2 제외(근거 4항)·§5-3 쿼드런트(제외 별도 영역).
- G-a 시드(기능↔문제 추적) — **PASS**: §3 표 14개 `F-*` 핸들 전부 PRB 매핑, PRB-1~8 전부 커버(추적
  커버리지 문단 명시).
- G-d(지표 수치+측정방법, 품질 vs 채택 가설) — **PASS**: §6-1 품질 7지표(수치·도구·주기·판정 모두 기입)
  / §6-2 채택 4지표(전부 "가설").
- G-c(실명 경쟁자 명시 비교 + moat) — **PASS**: §7 "RT·FW·PORT 동시 ✓는 ntop뿐" + 8축 매트릭스 요약 +
  포지셔닝 + 해자 3논거(research-s1 §9 / 02 인용).
- 차별화 마인드맵·가치루프 — **PASS(G-g)**: §3-1 구조 마인드맵·§4-1 가치 루프·§5-3 MVP 쿼드런트·§6 지표
  차트 모두 ASCII.

**공통 §8 게이트:**
- G-f(플레이스홀더 0) — **PASS**: grep TBD/TODO/placeholder/미정 = 0건.
- G-g(인포그래픽 우선·외부 렌더러 0) — **PASS**: 모든 다이어그램 순수 ASCII. grep "mermaid" 유일 히트는
  "Mermaid 미사용" 부정 문구.
- 단일 출처 레지스트리(spec §4-3) — **PASS**: 00↔01 상호 참조, 경쟁/생태계→02, 페르소나→03,
  config 정규값→31-erd로 위임. 중복정의 0.
- 도메인 재해석(spec §4-0) — **PASS**: 00을 OSS 지속가능성으로 재해석(결제/정산/DB/REST 창작 0),
  fs 미독 원칙을 01 §5-2에서 영구 제외로 유지.
- 사실 정정 반영 — **PASS**: TUI 목록 FRAMEWORK 컬럼 없음 + 세로 레이아웃을 01 §3·§4에 반영.
- 쉬운 설명(probe 9) — **PASS**: 두 문서 모두 첫 등장 용어 풀이 블록.
- Sprint Contract(상단)/자체 검증(하단) — **PASS**: 두 문서 각각 1쌍.

## Known limitations
- crates.io 게시·Homebrew 포뮬러의 실제 존재는 dossier §11에서 미확정 → "가설/예정"으로 정직 표기(00 §3).
  메인테이너 확인 시 "확정" 승격 필요.
- OSS 채택 지표의 목표치는 사실이 아니라 가설로만 제시(숫자 산정 0). 정량 산정은 spec/playbook이 금지.
- 잠정 기능 핸들(`F-*`)은 g2 `10-prd`에서 정규 기능 ID로 확정될 앵커(여기서 기능 창작 0).
- 제품 품질 지표 임계값 일부(scan < 3s, graceful 10s)는 config 기본값에 종속 → 정규 출처는 31-erd(g4),
  본 문서는 측정 임계값으로만 인용(전방 참조).

## How to review
- **G-a 시드:** 01 §3 표의 모든 `F-*`가 PRB로 역추적되는지, PRB-1~8이 전부 커버되는지 확인(추적
  커버리지 문단). g2 `10-prd`의 G-a 추적표가 이를 정규 ID로 승계할 수 있어야 함.
- **G-e:** 01 §5-1(포함)·§5-2(제외, 근거)·§5-3(쿼드런트)에서 MVP(v0.2.0)와 Out-of-scope(웹GUI·fs탐지·
  감독/재시작·전역그래프)가 명시 분리되고 우선순위 근거가 붙는지.
- **G-d:** 01 §6-1 각 지표에 수치·측정도구·주기·판정이 4요소 모두 있는지, §6-2 채택 지표가 전부 "가설"인지.
- **G-c:** 01 §7 + 00이 실명 경쟁자(htop·btop·pm2·systemd 등)와 축으로 비교하고 일반론("더 편리")을
  쓰지 않는지. 02 인용 경계 확인.
- **단일 출처:** 00=지속가능성/수익화/배포, 01=가치/MVP/지표 경계가 지켜지고 중복정의/모순이 없는지.
  config 수치는 31 참조로만 언급되는지.
- **창작 0:** 결제/정산/take-rate/DB/REST 언급이 전부 부정·면책 맥락인지(grep), fs 미독 원칙이 뒤집히지
  않는지(01 §5-2 영구 제외).

READY_FOR_QA: generator_report_g1a.md
</content>
