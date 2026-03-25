# Contributing

기여를 환영합니다!

## 개발 환경 설정

```bash
# 의존성 설치
pnpm install
cd crates/worker && pnpm install

# 포맷팅 도구
cargo install rustfmt
cargo install clippy
```

## 코드 스타일

```bash
# 포맷팅
cargo fmt

# Lint 검사
cargo clippy -- -D warnings

# 테스트
cargo test
```

## 커밋 컨벤션

[Conventional Commits](https://www.conventionalcommits.org/) 따릅니다:

- `feat`: 새로운 기능
- `fix`: 버그 수정
- `docs`: 문서
- `refactor`: 리팩토링
- `test`: 테스트
- `chore`: 빌드/설정

## PR 프로세스

1. Fork 후 브랜치 생성
2. 변경 사항 커밋
3. PR 생성 (템플릿 사용)
4. Code Review 통과
5. Merge
