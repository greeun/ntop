# Nuxt.js 완전 지원 설계

## 배경

`FrameworkKind::Nuxt`는 이미 존재하지만, 프로세스 이름 직접 매칭과 JS 런타임 인식이 빠져 있어 `nuxt` 바이너리로 직접 실행된 프로세스는 감지되지 않음.

## 변경 범위

### 1. `src/process/framework.rs`

**NAME_MAP 추가:**
- `("nuxt", FrameworkKind::Nuxt)` — `nuxt dev`, `nuxt start` 직접 실행 시
- `("nuxi", FrameworkKind::Nuxt)` — Nuxt 3 CLI

**`is_js_runtime` 확장:**
- `"nuxt"`, `"nuxi"` 추가 → package.json 스캔 트리거됨

### 2. `src/process/mod.rs`

- `FrameworkKind::Nuxt` 표시명: `"Nuxt"` → `"Nuxt.js"` (Next.js와 일관성)

### 3. `tests/framework_test.rs`

추가 테스트:
- `detect_by_name("nuxt")` → `Some(FrameworkKind::Nuxt)`
- `detect_by_name("nuxi")` → `Some(FrameworkKind::Nuxt)`
- `detect_by_package_json`에서 `"nuxt": "3.0.0"` → `(Some(FrameworkKind::Nuxt), Some("3.0.0"))`
- `detect` 통합: `nuxt` 프로세스 이름이 명령어보다 우선

## 비변경 범위

- `COMMAND_KEYWORDS`의 `"nuxt"` 항목: 이미 존재
- `PACKAGE_DEPS`의 `"nuxt"` 항목: 이미 존재
- TUI 렌더링 코드: 표시명은 `Display` trait에서 자동 반영
