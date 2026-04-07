# Makefile Task Runner 설계

## 목표

Makefile 각 타겟의 시작/종료/소요시간 추적, 성공/실패 분리, 로그 기록을 재사용 가능하게 구현.

## 요구사항

1. 각 task 시작시간, 종료시간, 동작시간 출력
2. 성공(`✓`) / 실패(`✗`) 분리 + EXIT code 표시
3. 재사용 가능 (다른 프로젝트 복사 가능)
4. `MAKE.log` 에 로그 기록
5. 로그 로테이션: 10MB 초과 시 압축 보관

## 출력 예시

```
[✓] terraform-apply      | START: 2026-04-07 08:15:23 UTC | END: 2026-04-07 08:17:41 UTC | ELAPSED: 02m 18s
[✗] spring-package       | START: 2026-04-07 08:20:10 UTC | END: 2026-04-07 08:22:05 UTC | ELAPSED: 01m 55s | EXIT: 1
```

## 아키텍처

### 파일 구성

| 파일 | 역할 |
|------|------|
| `scripts/task-runner.sh` | 재사용 가능한 task wrapper (독립 실행 가능) |
| `Makefile` | `$(RUN)` 매크로로 task-runner.sh 호출 |
| `MAKE.log` | 실행 이력 로그 (gitignore) |

### task-runner.sh

**인터페이스:**

```bash
bash scripts/task-runner.sh <task-name> <command...>
```

**동작 흐름:**

1. 로그 로테이션 체크 (MAKE.log > 10MB → `MAKE.log.1.gz` 압축)
2. START 시간 기록 (`date -u`)
3. 명령 실행 (`"$@"`), stdout/stderr 그대로 출력 + MAKE.log 동시 기록
4. 종료 코드 캡처
5. END 시간 기록, ELAPSED 계산
6. 성공/실패 상태를 포맷에 맞게 출력 + MAKE.log 기록
7. 종료 코드 전파 (실패 시 non-zero)

**로그 로테이션 규칙:**

- MAKE.log 크기 > 10MB: `MAKE.log` → `MAKE.log.1.gz` 압축
- 기존 `MAKE.log.1.gz` 존재 시 덮어쓰기 (1세대만 유지)

### Makefile 변경

```makefile
# 추가
RUN = @bash scripts/task-runner.sh

# Before
build-worker:
	@echo "Building Worker (WASM)..."
	@cd crates/worker && worker-build --release

# After
build-worker:
	$(RUN) build-worker cd crates/worker && worker-build --release
```

**규칙:**
- 모든 타겟에 `$(RUN) <task-name>` 적용
- `loc-*` (장기 실행 서버), `clean`, `destroy-*` 타겟도 동일 적용
- `setup`, `deploy`, `test` 등 복합 타겟(의존성 체인)은 개별 서브 타겟에만 적용

## 영향 범위

- `scripts/task-runner.sh` — 신규 파일
- `Makefile` — 모든 타겟 수정
- `.gitignore` — `MAKE.log` 추가
- 기존 동작 변경 없음 (추가 출력만 발생)
