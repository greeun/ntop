# Critique — 리서치 스프린트 게이트 (S1·S2·S3, MODE A2)

> 평가자(Evaluator) = Generator의 적대자(사용자 편). 기준: `spec.md`·`sprint-playbook.md`·`rubric.md`·
> `evaluator-calibration.md`, 사실 원천 = `codebase-facts.md`(정정본). 각 점수는 calibration 1/3/5
> 앵커에 정렬, 증거는 정확 인용 + 위치로 표기. C2/C3는 2×, C1/C4는 1×.
> S1→C2(2×) / S2→C1(1×) / S3→C1 일부 + 와이어프레임 P-1 채점.

---

# Sprint 1 — 시장·경쟁·생태계 (`research-s1.md`)

## Verdict: **PASS**

C2(2×, S1의 핵심 통과 조건) = 5/5 ≥4. 게이트 G-c·G-g 통과. 차별화 명시비교 probe·생태계/시각화 probe 통과.

## Rubric Scores
| Criterion | Score | Weight | Justification | Evidence |
|---|---|---|---|---|
| C2 차별화·독창성 | **5/5** | 2× | 실명 경쟁자 10개 × 비교 축 8개 매트릭스 + 포지셔닝 쿼드런트 + "RT·FW·PORT 동시 ✓는 ntop뿐" 명시비교 + 3중 모방난이도(해자) 논거 → calibration C2 5/5 앵커(다수 경쟁자+비교 축+해자) 초과 | §3 매트릭스 "RT·FW·PORT 세 열이 동시에 ✓인 도구는 ntop뿐이다"(L198), §9 해자 3논거 "① 경험이 응축된 데이터 사전 × ② 비자명한 탐지 설계 원칙 × ③ 복리로 커지는 확장 구조"(L482-484) |

> 보강 렌즈(경쟁자 시점): 해자 논거가 "더 편리/빠름" 일반론이 아니라, `package.json` 미독 트랩·`normalize_name`(comm 16자 절단)·`Rule` 1개 추가 확장구조 등 **코드 근거로 특정**됨(L464-480). table-stakes 아님 확인. REDIRECT 불필요.

## §8 Gate Results (S1 관련)
- **G-c (실명 경쟁자 명시 비교): PASS** — htop·btop·top·glances·gotop·Activity Monitor·ctop·pm2·systemd·launchd(10종) 각각 "한다/안 한다" 프로파일(§2-1) + ✓/△/✗ 매트릭스(§3) + △ 판정 근거 명시(L179-196). 일반론 미사용.
- **G-g (인포그래픽 우선, 외부 렌더러 0): PASS** — 비교 매트릭스(§3)·포지셔닝 쿼드런트(§4)·세그먼트 깔때기(§5-2)·**생태계 가치 네트워크 맵(§7, 노드-엣지)** 모두 순수 ASCII. Mermaid 미사용(L367 명시), 빈 다이어그램 0. S1 추가요건인 **생태계 맵(참여자/가치 네트워크) 존재** 충족(공급/수요/플랫폼·중개/보완재/대체재/제도/후원·배포 7범주, §6).

## Wireframe Checkpoint
none — automatic (S1에 와이어프레임 없음).

## Blocking Issues
없음.

## Non-Blocking Notes
1. 경쟁자 세부기능(glances 포트 플러그인, btop Windows 성숙도 등)을 "보수적으로(없는/제한적인 쪽)" 판정했다고 자체 명시(L512-514). 외부 도구 사실이라 도시에로 역추적 불가 → 판정은 합리적이고 정직하나, S4 `02-market-competition` 전환 시 버전 명시(예: htop 3.x, btop++ 1.x)를 부기하면 검증성이 올라간다.
2. launchd를 경쟁자 행에 둔 것은 다소 약하나 spec §6이 "간접 비교"로 명시했으므로 허용 범위.

## Recommended Next Focus
S4 `02-market-competition`로 옮길 때 매트릭스/쿼드런트/생태계 맵을 그대로 승계하되, C-7 생태계 맵을 HTML 인라인 SVG로 변환(g6) 시 노드-엣지 가독성 확보. C2 근거(§9)는 `01-service-plan` 차별화 섹션의 단일 출처로 삼을 것.

---

# Sprint 2 — 문제·타깃·페르소나 (`research-s2.md`)

## Verdict: **PASS**

C1(1×, ≥3 필요) = 5/5. 게이트 G-a·G-b·G-g 통과. 타깃 구체성 probe 통과.

## Rubric Scores
| Criterion | Score | Weight | Justification | Evidence |
|---|---|---|---|---|
| C1 문제정의 명료성 | **5/5** | 1× | 핵심 문제를 "의미 격차"로 누가/무엇이/왜 정의(§1) → 6개 하위문제를 각각 누가·무엇이·왜·**빈도·결과**로 분해(§2) → PRB-ID 앵커화(§3); 페르소나 P1·P3가 이름·스택·일과 한 장면·빈도/결과·우회법·원하는 결과까지 구체 → calibration C1 5/5 앵커(상황·빈도·결과까지) 충족·초과 | §4-1 P1 박서연 "node 6~8개 + MCP 2~3개… 지난주엔 MCP 서버를 dev 서버로 착각해 죽였다가 Claude Code 세션이 끊겼다"(L201-202), 페인 빈도/결과 "월 1~2회 실제 사고… 재시작·재인덱싱 10~15분 손실"(L203) |

> 보강 렌즈(데맨딩 PM 시점): 빈도 수치(주 10회·월 1~2회)는 도시에 부재 → "사용 맥락 기반 개연 추정"으로 정직하게 라벨링(서두·L373-375). ntop **동작** 인용은 전부 도시에 §번호 부기 → 추정과 사실을 분리해 둠. C1 감점 사유 아님.

## §8 Gate Results (S2 관련)
- **G-a (문제 목록 = 추적 시작점 존재): PASS** — §3 PRB-1~8 표(문제·통증·주 페르소나·메우는 ntop 동작·도시에 근거·잠정 F-핸들). S4 `10-prd`가 기능을 역추적할 명확한 출발점 제공.
- **G-b (타깃 구체성): PASS** — P1·P3 **완전 구체**(추상 인구통계 아님). 핵심 1개 요건을 2개로 초과 충족. P2(운영자 축)도 상황/일과/우회법/원하는 결과 보유. calibration C1 5/5 앵커 수준의 페르소나.
- **G-g (인포그래픽 우선, 외부 렌더러 0): PASS** — §1 격차 도식·§5 능력 매핑·§6-1 여정 맵·§6-2 문제×페르소나 매트릭스·§6-3 통증-빈도 차트 모두 ASCII. Mermaid 미사용(L365), 빈 다이어그램 0.

## Wireframe Checkpoint
none — automatic.

## Blocking Issues
없음.

## Non-Blocking Notes
1. **[사실 정정 — S4에서 수정 필수]** L92 "→ 좌측 목록 `NAME`/`FRAMEWORK` 컬럼·Info 탭" 및 §6-1 여정맵 L287 "`ntop` 실행 → **좌측 목록에서 NAME/FRAMEWORK**/PORT 즉시 확인"은 **정정본 도시에 §6(L250)과 모순**이다: "TUI 좌(상단) 목록에는 **FRAMEWORK 컬럼이 없다** … framework/runtime은 상세 Info 탭 + CLI list 테이블에서만 노출", 레이아웃도 좌/우가 아니라 **세로(상/하)**(도시에 L245). 이 슬립은 `spec.md` §4-2 추적성 예시(L104 "좌측 목록 NAME/FRAMEWORK 컬럼")에서 그대로 상속된 것으로, S2 작성 시점에는 spec/도시에가 미정정 상태였던 데 기인. **C1 채점에는 영향 없음**(문제정의·페르소나 본문이 아니라 "ntop이 메움" 주석의 UI 위치 표기일 뿐)이나, S4 `03-personas`·`10-prd`로 옮길 때 반드시 "Info 탭/CLI 테이블의 FRAMEWORK"로 수정하고 "좌측"→"상단 목록"으로 바로잡아야 함. (S3는 이미 코드-진실로 정정 적용됨 — 아래 S3 참조.)
2. P2(정태호)는 P1·P3 대비 "일과 한 장면"의 구체 수치가 얕다(빈도 "주 수회 인시던트"). 통과엔 영향 없으나 S4 `03-personas`에서 P2도 빈도·결과를 P1 수준으로 보강하면 좋다.

## Recommended Next Focus
S4 g1로 옮기기 전 비고1의 FRAMEWORK/좌측 표기를 정정해 오류 전파 차단. PRB-1~8을 `10-prd`의 기능↔문제 추적표(G-a)와 `11-user-stories` GWT의 앵커로 1:1 연결.

---

# Sprint 3 — UX/UI 플로우·화면 (`research-s3.md`)

## Verdict: **PASS**

C1 일부(플로우↔문제 일관) 강함. 게이트 G-g 통과. 와이어프레임 존재 → **P-1 HUMAN_CHECKPOINT_REQUIRED**(시각 품질은 채점 제외, 텍스트부만 채점).

## Rubric Scores (S3 관련 = C1 일부)
| Criterion | Score | Weight | Justification | Evidence |
|---|---|---|---|---|
| C1 일부(플로우↔문제 일관) | **5/5** | 1× | F1~F6 모든 플로우 단계가 research-s2의 PRB-ID로 라벨링되어 문제-플로우 일관; IA 트리·화면 인벤토리·키 바인딩이 도시에 §6/§7과 **정확 일치**; 코드 근거(파일:라인) 부기 | §3 각 플로우 PRB 라벨(F1 "PRB-1·2·6·8" L158 등), §7 키 바인딩 5그룹이 도시에 §6과 일치(전역 q//s/r/+/−/x/K/H/S/e/n) |

> 사실 일치 정밀 점검: graceful→force 상태기계(§4-1: SIGTERM→200ms 폴링→graceful_timeout 10s→Terminated/TimedOut/AlreadyDead/PermissionDenied, TUI=DLG-FORCE 확인·CLI=자동)가 도시에 §5와 일치. Unix 6시그널/Windows 3시그널·EPERM·taskkill 모두 정확(§3-5·§4-2).

## §8 Gate Results (S3 관련)
- **G-g (플로우·화면 다이어그램 적합, 외부 렌더러 0): PASS** — IA 트리(§2)·플로우차트 6종(§3)·상태기계+시퀀스(§4)·화면 인벤토리(§5)·4상태 매트릭스+전이도(§6)·와이어프레임 12종(§8) 전부 순수 ASCII. Mermaid 미사용(L804), 빈 다이어그램 0. **산문-only 화면 0**(모든 화면에 시각 동반) — 표준 시각 점검 통과.

## Wireframe Checkpoint
**HUMAN_CHECKPOINT_REQUIRED** — 위치: **§8 저충실도 와이어프레임 (L543–729)**, 12종 시안(8-1 V-MAIN · 8-2~8-5 TAB-INFO/LOG/NET/ENV · 8-6~8-10 DLG-KILL/KILLTREE/SIGNAL/FORCE/HELP · 8-11 V-FILTER · 8-12 V-EMPTY). 문서 §8 상단(L545)에 `HUMAN_CHECKPOINT_REQUIRED` 명시 + 검수 포인트 5개(L731-733) 제시. 시각 품질(폭·정렬·문자열 충실도)은 채점 제외 → 오케스트레이터가 사용자 승인에 노출.

## 사실 정확성 — 정정본 도시에 대조 (특별 점검)
- **레이아웃 세로(상/하) 55/45 채택: 정확.** §0 정정 노트(L41-48)·§2-2(L109)·§8-1 와이어프레임 모두 "위 55% process_list / 아래 45% detail_panel" = 코드-진실. 정정본 도시에 §6(L245)과 일치. ✅
- **TUI 목록에 FRAMEWORK 컬럼 없음: 정확.** §2-2(L112) 컬럼 "[✓/health] PID·NAME·PORT·THR·CPU·MEM·USER·STATUS·UPTIME"(FRAMEWORK 없음), §8-1 와이어프레임도 동일. FRAMEWORK는 TAB-INFO(§2-2 L118)·CLI-LIST table(§2-2 L138)에서만 노출 = 정정본 도시에 §6(L250)과 일치. ✅
- 결론: **S3는 코드-진실(세로 레이아웃, FRAMEWORK 미포함 목록)을 정확히 사용.** 정정본 도시에와 모순 없음.

## Blocking Issues
없음.

## Non-Blocking Notes
1. **[메타-노트 잔존, 무해]** §0 정정 노트(L41-48)·§10 플래그(L779-783)는 "도시에 §6이 좌/우로 적었으니 메인테이너가 정정하라"고 회부하는데, 도시에는 **이미 세로(상/하)로 정정 완료**다(L245). 즉 본문이 채택한 사실(세로·FRAMEWORK 미포함)은 정확하고, 이 플래그 문구만 시점상 낡은(stale) 상태다. 실제 사실 주장에는 오류가 없으므로 비블로킹. S4로 옮길 때 이 회부 문구는 "정정 반영 완료"로 정리하면 됨.
2. 와이어프레임은 화면당 1종(12 화면=12 시안)으로 충분하나, 핵심 화면(V-MAIN/V-LIST)의 4상태 세트는 §6 매트릭스로만 다룸. S4 g3 `21-screen-spec`에서 핵심 화면은 empty/loading/normal/error를 **개별 시안**으로 분리할 것(spec §5 21번 품질바).
3. §9-(1) 발견성(하단바가 K/e/n/r 등 일부 전역키를 상시 노출 안 함)을 정직하게 결함 후보로 플래그함 — 결함 아님 판정 합리적. S4 `21`에서 "Help 외 노출 경로" 명시 권장.

## Recommended Next Focus
§7 키 바인딩을 `21-screen-spec` 키 단일 출처로 그대로 승계. §8 와이어프레임은 사용자 STEP 7 승인 후 g3로. 핵심 화면 4상태를 개별 시안화. §10 정정 회부 문구를 "반영 완료"로 정리.

---

# 종합 게이트 판정

| 스프린트 | 핵심 축 | 점수 | 게이트 | Verdict |
|---|---|---|---|---|
| S1 시장·경쟁·생태계 | C2 (2×) | 5/5 | G-c ✓ · G-g ✓(생태계 맵) | **PASS** |
| S2 문제·타깃·페르소나 | C1 (1×) | 5/5 | G-a ✓ · G-b ✓ · G-g ✓ | **PASS** |
| S3 UX/UI 플로우·화면 | C1 일부 | 5/5 | G-g ✓ · P-1 HUMAN_CHECKPOINT_REQUIRED(§8) | **PASS** |

- 창작(결제/정산/take-rate/DB/REST) 0 — 전 언급이 부정/면책 맥락(grep 검증).
- 외부 렌더러(Mermaid) 의존 0, 플레이스홀더/TBD 0(grep 검증).
- 쉬운 설명(probe 9): 3문서 모두 첫 등장 용어 풀이 블록 보유(TUI/CLI/runtime/framework/MCP/LSP/graceful/IA 등) — 통과.
- S4 이월 필수 수정 1건: **S2의 "좌측 목록 NAME/FRAMEWORK 컬럼" 표기**(L92·L287)를 "상단 목록(FRAMEWORK 없음) + Info 탭/CLI 테이블"로 정정(S3는 이미 정정 적용). 동일 오류가 `spec.md` L104 추적성 예시에도 남아 있으니 S4 작성 전 함께 바로잡을 것.

RESEARCH GATE: PASS
