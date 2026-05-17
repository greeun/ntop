# ntop

Node.js / Next.js / Nuxt.js 서버 프로세스를 실시간으로 모니터링하고 관리하는 터미널 TUI 도구입니다.

Rust로 개발되어 빠른 실행과 최소한의 리소스 사용을 보장합니다.

![License](https://img.shields.io/badge/license-MIT-blue.svg)

## 주요 기능

- **실시간 프로세스 모니터링** - 설정 가능한 갱신 주기
- **프레임워크 자동 감지** - Next.js, Nuxt.js, Express, Fastify, NestJS, Koa, Hapi
- **프로세스 트리 뷰** - 부모-자식 관계를 접기/펼치기로 탐색
- **분할 패널 TUI** - 프로세스 목록 + 탭형 상세 패널 (Info / Log / Net / Env)
- **로그 스트리밍** - 감지된 로그 파일의 실시간 tail
- **네트워크 검사** - 프로세스별 리스닝 포트 및 활성 TCP 연결
- **환경변수 조회** - 민감 정보(PASSWORD, TOKEN 등) 자동 마스킹
- **다양한 시그널 지원** - SIGTERM, SIGKILL, SIGHUP, SIGINT, SIGUSR1, SIGUSR2
- **그레이스풀 셧다운** - SIGTERM 전송 후 타임아웃 시 SIGKILL 에스컬레이션
- **트리 Kill** - 부모 프로세스 + 모든 자식 프로세스 일괄 종료
- **다중 선택** - 여러 프로세스를 선택하여 일괄 종료
- **CLI 서브커맨드** - `list`, `kill`, `info`, `log` (JSON/CSV 출력 지원)
- **설정 파일** - `~/.config/ntop/config.toml` (TOML 형식)

## 설치

### 소스 빌드 (cargo)

```bash
cargo install --git https://github.com/greeun/ntop
```

### 릴리스 바이너리

[Releases](https://github.com/greeun/ntop/releases) 페이지에서 플랫폼별 바이너리를 다운로드하세요.

## 사용법

### TUI 모드 (기본)

```bash
ntop
```

인터랙티브 대시보드가 실행됩니다:

```
┌─────────────────────────────────────────────────────────────────┐
│ ntop v0.1.1  |  CPU: 12.3%  MEM: 4.2GB  |  Nodes: 7  | [H]elp│
├──────────────────────────┬──────────────────────────────────────┤
│  프로세스 목록            │  [Info] [Log] [Net] [Env]           │
│                          │                                      │
│  ▸ ● Next.js dev :3000  │  PID:       12345                    │
│    ├ next-server         │  Framework: Next.js                  │
│    └ next-router-worker  │  Port:      3000                     │
│  ▸ ● Express    :4000   │  CPU:       3.2%                     │
│    ● Node.js    :8080   │  Memory:    128.0 MB                 │
│                          │  Uptime:    2h 13m 5s                │
├──────────────────────────┴──────────────────────────────────────┤
│ [q] 종료 | [↑↓] 이동 | [Tab] 탭 전환 | [x] Kill | ...        │
└─────────────────────────────────────────────────────────────────┘
```

### 키 바인딩

| 키 | 동작 |
|----|------|
| `↑/↓` 또는 `j/k` | 프로세스 목록 이동 |
| `Enter` | 프로세스 트리 접기/펼치기 |
| `Tab` | 상세 탭 전환 (Info → Log → Net → Env) |
| `Space` | 다중 선택 토글 |
| `/` | 검색/필터 |
| `s` | 정렬 컬럼 변경 |
| `x` | 선택 프로세스 종료 |
| `K` | 프로세스 트리 전체 종료 |
| `S` | 시그널 선택 다이얼로그 |
| `q` / `Esc` | 종료 |

### CLI 모드

```bash
# Node.js 프로세스 목록
ntop list
ntop list --json
ntop list --format csv

# 프로세스 종료
ntop kill <PID>
ntop kill --tree <PID>            # 트리 전체 종료
ntop kill --signal SIGKILL <PID>  # 시그널 지정
ntop kill --all                   # 모든 Node 프로세스 종료
ntop kill --no-confirm <PID>      # 확인 없이 종료

# 상세 정보 조회
ntop info <PID>

# 로그 스트리밍
ntop log <PID>

# 설정 파일 경로 확인
ntop config
```

### 출력 예시: `ntop list`

```
PID      NAME                 FRAMEWORK    PORT       CPU      MEM        UPTIME
--------------------------------------------------------------------------------
12345    node                 Next.js      3000       3.2%     128.0 MB   2h 13m
12390    node                 Express      4000       1.1%     64.0 MB    5h 2m
12401    node                 Node.js      8080       0.3%     32.0 MB    12m
```

## 설정

설정 파일 경로: `~/.config/ntop/config.toml`

```toml
[general]
refresh_interval = 1          # 갱신 주기 (초)
default_signal = "SIGTERM"    # 기본 시그널
graceful_timeout = 10         # 그레이스풀 셧다운 타임아웃 (초)
confirm_before_kill = true    # Kill 전 확인 여부

[display]
show_tree = true              # 프로세스 트리 표시
color_theme = "auto"          # auto | dark | light
mask_env_values = true        # 민감 환경변수 값 마스킹

[filter]
include_bun = false           # bun 런타임 포함
include_tsx = false            # tsx 런타임 포함
include_ts_node = false        # ts-node 런타임 포함
```

## 지원 프레임워크

| 프레임워크 | 감지 방법 |
|------------|-----------|
| Next.js | `next-server` 프로세스명, 커맨드의 `next`, package.json의 `next` |
| Express | package.json의 `express` |
| Fastify | package.json의 `fastify` |
| NestJS | package.json의 `@nestjs/core` |
| Nuxt.js | `nuxt`, `nuxi` 프로세스명, 커맨드의 `nuxt`, package.json의 `nuxt` |
| Koa | package.json의 `koa` |
| Hapi | package.json의 `@hapi/hapi` |

## 시스템 요구사항

- macOS 또는 Linux
- Rust 1.70+ (소스 빌드 시)

## 라이선스

MIT
