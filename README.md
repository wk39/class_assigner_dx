# class-assigner-dx

학교 반 배정용 웹 애플리케이션입니다. Rust + [Dioxus 0.7](https://dioxuslabs.com/learn/0.7) 로 작성되어 WebAssembly 로 브라우저에서 실행됩니다.

## 목적

한 학년의 학생들을 여러 학급으로 배정할 때, 학급 간 **평균 점수**와 **성비**가 균형을 이루도록 도와주는 도구입니다. 담임 교사나 교무 담당자가 별도의 스프레드시트 수식 없이 브라우저에서 즉시 학생 목록을 관리하고 배정 결과를 얻을 수 있도록 만드는 것을 목표로 합니다.

## 유의사항

- 모든 연산은 접속한 **장치(PC / 모바일) 내에서만** 수행됩니다. 서버로 학생 정보가 전송되지 않습니다.
- 따라서 **새로고침하거나 탭을 닫으면 입력한 데이터가 사라집니다**. 필요하면 작업 전·후로 별도 저장 수단을 사용하세요.
- 현재 배정 알고리즘은 **스네이크 드래프트(snake draft)** 방식 한 가지입니다. 점수 균형 옵션이 켜져 있으면 점수 내림차순으로 정렬 후 분배하고, 성비 옵션이 켜져 있으면 성별로 먼저 나눠 각각을 분배합니다. 두 옵션 모두 꺼져 있으면 무작위 배정입니다.
- 초기 진입 시 테스트용 학생 101명이 랜덤으로 생성됩니다 (`src/main.rs` 의 `App` 함수). 운영 시에는 이 부분을 제거하거나 빈 목록으로 초기화해야 합니다.

## 요구사항 및 구현 상태

현재 저장소 (`src/main.rs`) 기준.

| 분류 | 요구사항 | 상태 | 비고 |
| --- | --- | --- | --- |
| 라우팅 | 사이드바 기반 다중 페이지 | ✅ 구현 | 메인 / 학생 목록 / 반 배정 / 정보 |
| 학생 목록 | 학생 추가 / 삭제 | ✅ 구현 | `+ 학생 추가` 버튼, 행 단위 `X` 버튼 |
| 학생 목록 | 이름 / 성별 / 점수 / 비고 편집 | ✅ 구현 | 표 내 인라인 입력 |
| 학생 목록 | 초기 샘플 데이터 | ⚠️ 데모 전용 | 앱 시작 시 랜덤 학생 101명 자동 생성 |
| 학생 목록 | CSV / 엑셀 가져오기·내보내기 | ❌ 미구현 | — |
| 반 배정 | 학급 수 설정 (3~30) | ✅ 구현 | 슬라이더 |
| 반 배정 | 평균점수 균형 옵션 | ✅ 구현 | 점수 내림차순 snake-draft |
| 반 배정 | 성비 균형 옵션 | ✅ 구현 | 성별 분리 후 그룹별 snake-draft |
| 반 배정 | 배정 알고리즘 실행 | ✅ 구현 | `assign_classes` (snake-draft 기반) |
| 반 배정 | 결과 표시 | ✅ 구현 | 반별 카드: 인원·평균·성비·학생 목록 |
| 반 배정 | 배정 결과 내보내기 | ❌ 미구현 | CSV / 인쇄 등 |
| 데이터 | 브라우저 내(클라이언트) 저장 (localStorage 등) | ❌ 미구현 | 새로고침 시 소실 |
| 정보 페이지 | 프로그램 정보 문구 | 🟡 스텁 | 본문 자리표시자 수준 |
| Egui 연동 | iframe 기반 별도 WASM 앱 임베드 | 🔒 비활성 | `EguiPage` 코드는 있으나 라우트 주석 처리 |

범례: ✅ 완료 · 🟡 부분 구현 · 🚧 진행 중 · ❌ 미구현 · 🔒 비활성

### 실행 환경 (사용자)

- 최신 크로미움 / 파이어폭스 / 사파리 계열 브라우저 (WebAssembly 지원)
- 네트워크는 초기 페이지 로딩에만 필요. 이후 동작은 오프라인에서도 가능

### 개발 환경

- Rust (stable, edition 2021)
- [Dioxus CLI (`dx`)](https://dioxuslabs.com/learn/0.7) — `curl -sSL https://dioxus.dev/install.sh | sh`
- WebAssembly 타겟: `rustup target add wasm32-unknown-unknown`
- Node.js 는 **불필요** (Dioxus 0.7 의 자동 Tailwind 사용)

## 프로젝트 구조

```
.
├─ src/main.rs       # 모든 컴포넌트, 라우트, 상태가 정의된 단일 엔트리
├─ assets/           # favicon, main.css, tailwind.css 등 정적 자원
├─ public/egui-app/  # (선택) iframe 으로 임베드되는 별도 egui WASM 앱
├─ Cargo.toml
├─ Dioxus.toml       # 타이틀, base_path, watcher 설정
└─ tailwind.css      # Tailwind 입력 파일 (dx serve 가 자동 처리)
```

## 실행

개발 서버:

```bash
dx serve
```

다른 플랫폼:

```bash
dx serve --platform desktop
```

릴리즈 빌드 (`Cargo.toml` 의 `opt-level = "z"`, `lto`, `strip` 으로 크기 최적화됨):

```bash
dx build --release
```

## 의존성

- `dioxus` 0.7 (`web`, `router`)
- `dioxus-logger`, `log`
- `gloo-timers` — 비동기 지연
- `rand`, `rand_distr` — 샘플 학생 데이터 생성
- `getrandom` (wasm32 타겟, `wasm_js` 피처)
