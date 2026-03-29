# AI Global Rules

PushOver Wrapper 를 만든다 [PushOver API](https://pushover.net/api) 와 [CloudFlare](https://cloudflare.com/) 이용해서.

## 🗣️ Language & Communication

- **주 언어**: 모든 응답, 설명, 주석은 **한국어**를 기본으로 합니다.
- **전문 용어**: 명확성을 위해 IT 전문 용어는 영어 원문을 그대로 사용하거나 병기합니다.
  - 예: "의존성 주입(Dependency Injection)", "Race Condition 발생 가능성"
- **어조**: 공감이나 미사여구는 생략합니다. 간결하고(Concise), 전문적이며(Professional), 드라이(Dry)한 어조를 유지합니다.
- **요약**: 긴 설명이 필요한 경우, 핵심 내용을 먼저 요약(TL;DR)하여 상단에 배치합니다.

## 💻Coding Standards

- **test**: make 로 통합 관리 하고 각 모듈별로 test 구현한다.
- **Reference**: **Always use Context7 MCP when I need library/API documentation, code generation, setup or configuration steps without me having to explicitly ask.**
- **일관성(Consistency)**: 기존 프로젝트의 코딩 스타일(들여쓰기, 네이밍 컨벤션, 패턴)을 최우선으로 준수합니다.
- **주석**: *왜(Why)* 그렇게 작성되었는지에 집중합니다. 과한 주석보다는 깔끔한 code 가 좋다.
- **안전성**: 에러 핸들링(Error Handling)과 엣지 케이스(Edge Cases)를 항상 고려합니다.
- **라이브러리**:
  - API Token, API Key 값을 이야기 할때는 항상 Full Name 으로 이야기 한다.
  - cloudflare-docs MCP 를 이용해서 Cloudflare 기능을 참조한다.
  - **알림**: [PushOver](https://pushover.net/api) 를 이용한 iOS, Android, Desktop(Web browser) 대상으로 한다.

## 🛡️ Operations & Safety

- **파괴적 명령어**: 파일 삭제(`rm`), 강제 종료(`kill`) 등 시스템 변경이 큰 작업은 실행 전 반드시 사용자에게 확인을 받거나 경고 문구를 출력합니다.
- **파일 경로**: 절대 경로보다는 프로젝트 루트 기준의 상대 경로를 사용하여 가독성을 높입니다.

## 📝Git

- **보안 점검**: git commit 하기 전 commit 대상의 파일들에 보안에 취약한 내용을 확인한다.
- **커밋 메시지**: [Conventional Commits](https://www.conventionalcommits.org/) 규격을 따릅니다.
  - 커밋 말머리는 **영어**로 작성하는 것을 기본으로 하되, 메세지는 **한국어**로 작성합니다.

## 🧰Tool

- Software SDK 는 mise 를 사용 한다.
- Node Package 관리시 pnpm 을 사용 한다.
- Python Package 관리시 uv 를 사용 한다.
- SDK, Package Mananger 를 사용시 Project 에서 만 사용하는것 사용 한다.

## 🚀Problem Solving

1. **상황 파악**: 파일 구조와 관련 코드를 먼저 읽고 분석합니다.
2. **원인 추론**: 문제의 근본 원인을 논리적으로 추론합니다.
3. **계획 수립**: 단계별 해결책을 제시합니다.
4. **BackUp**:  `git tag` 로 `YYMMDD/hh:mm` 사용해 `checkpoint` 한다.
5. **TDD**: 계획 수립에 맞게 테스트 코드 작성 하고 관리는 make 로 관리한다.
   - make 로 실패 하는 코드 까지 작성을 해야 합니다.
6. **실행**: 개발을 시작한다.
7. **검증**: make 를 이용해 테스트 코드를 검증 한다.
8. **Restore**: 심각한 오류가 있을 때만 사용자의 동의를 `checkpoint` 로 돌아간다.

## 📂 Project Structure

```
PushOver/                      # 프로젝트 루트 — 공통 문서(Makefile, *.md)만 위치, 어떤 SDK도 루트를 독점하지 않음
├── Makefile                  # 전체 빌드/테스트 orchestration
├── CLAUDE.md                 # AI 지침
├── crates/                   # Rust workspace
│   ├── Cargo.toml            # Rust workspace 루트 (members: sdk, cli, worker)
│   ├── Cargo.lock            # Rust 의존성 lock
│   ├── cli/                  # CLI 도구 (Rust binary)
│   ├── sdk/                  # SDK 라이브러리 (Rust library)
│   └── worker/               # Cloudflare Worker (Rust → WASM)
│       ├── wrangler.toml     # Worker 배포 설정
│       └── package.json      # worker-build JS 바인딩용
├── dashboard/                # Next.js 프론트엔드 (유일한 Node.js 프로젝트)
│   ├── package.json          # Node.js 의존성 관리
│   └── .vercel/output/       # @cloudflare/next-on-pages 빌드 산출물
├── tests/                    # 통합 테스트
└── infrastructure/           # OpenTofu (Terraform) IaC
```

### 구조 규칙

- **프로젝트 루트는 중립 지대** — 공통 문서(Makefile, *.md, .gitignore)만 위치, 어떤 SDK도 루트를 독점하지 않음
- **`package.json`은 `dashboard/`, `crates/worker/` 에만 허용** — 루트에 Node.js package.json 생성 금지 (Next.js module resolver 오염 방지)
- **`Cargo.toml`은 `crates/` 에만 위치** — Rust workspace 루트는 `crates/Cargo.toml`
- **배포 플랫폼은 Cloudflare Pages/Workers만 사용** — Vercel 플랫폼 미사용
- **`.vercel/` 디렉토리는 `dashboard/` 하위만 허용** — Cloudflare Pages 빌드 포맷(Vercel Build Output API)의 산출물
- **Node.js 명령어는 항상 `dashboard/`에서 실행** — `pnpm install`, `pnpm build`, `next dev` 등
- **Rust 명령어는 항상 `crates/`에서 실행** — `cargo build`, `cargo test`, `cargo check` 등
