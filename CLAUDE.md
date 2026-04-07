# AI Local Rules

당신은 프로젝트 'PushOver Wrapper'를 완수하기 위한 전문 AI 파트너이자 아키텍트입니다.

## 개요

- **프로젝트명**: PushOver
- **핵심 가치**: [PushOver API](https://pushover.net/api) 와 [CloudFlare](https://cloudflare.com/) 사용 해서 학습 읨미를 둔다.

## Project Structure

```bash
PushOver/                     # 프로젝트 루트 — 공통 문서(Makefile, *.md)만 위치, 어떤 SDK도 루트를 독점하지 않음
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

- **프로젝트 루트는 중립 지대** — 공통 문서(Makefile, *.md, .gitignore)만 위치, 어떤 SDK도 루트를 사용 하지 않음
- **`package.json`은 `dashboard/`, `crates/worker/` 에만 허용** — 루트에 Node.js package.json 생성 금지 (Next.js module resolver 오염 방지)
- **`Cargo.toml`은 `crates/` 에만 위치** — Rust workspace 루트는 `crates/Cargo.toml`
- **배포 플랫폼은 Cloudflare Pages/Workers만 사용** — Vercel 플랫폼 미사용
- **`.vercel/` 디렉토리는 `dashboard/` 하위만 허용** — Cloudflare Pages 빌드 포맷(Vercel Build Output API)의 산출물
- **Node.js 명령어는 항상 `dashboard/`에서 실행** — `pnpm install`, `pnpm build`, `next dev` 등
- **Rust 명령어는 항상 `crates/`에서 실행** — `cargo build`, `cargo test`, `cargo check` 등
