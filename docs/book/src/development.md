# Разработка

## Требования к окружению

| Инструмент | Версия | Назначение |
|------------|--------|------------|
| Rust | stable | Компиляция backend |
| Node.js | 20+ | Frontend (SvelteKit) |
| PostgreSQL | 16+ | Основная БД |
| Redis | 7+ | Кэш, rate limiting |
| ClickHouse | 24+ | Аналитика |
| Python | 3.11+ | ML-сервис |

## Запуск для разработки

### Backend

```bash
cd backend
cp .env.example .env
# Настроить переменные окружения в .env
cargo run
```

Backend запускается на порту 8080 (настраивается через `PORT`).

### Frontend

```bash
cd frontend
npm install
npm run dev
```

Frontend запускается на порту 5173 с проксированием API-запросов на backend.

### ML-сервис

```bash
cd ml
pip install -r requirements.txt
uvicorn app.main:app --port 8000
```

### Docker Compose

Для запуска всех зависимостей (PostgreSQL, Redis, ClickHouse):

```bash
docker-compose up -d
```

## Переменные окружения (Backend)

| Переменная | Описание | По умолчанию |
|------------|----------|-------------|
| `DATABASE_URL` | Строка подключения PostgreSQL | — |
| `REDIS_URL` | Строка подключения Redis | — |
| `JWT_SECRET` | Секрет для подписи JWT | — |
| `PORT` | Порт HTTP-сервера | `8080` |
| `CORS_ORIGINS` | Разрешённые origins (через запятую) | `http://localhost:5173` |
| `LELY_API_KEY` | API-ключ Lely Horizon | — |
| `LELY_BASE_URL` | Базовый URL Lely API | — |
| `DEMO_MODE` | Режим демонстрации | `false` |
| `RATE_LIMIT_MAX` | Макс. запросов в окне | `100` |
| `RATE_LIMIT_WINDOW_SECS` | Окно rate limit (сек) | `60` |
| `TRUST_PROXY` | Доверять X-Forwarded-For | `false` |
| `SWAGGER_ENABLED` | Включить Swagger UI | `true` |

## Миграции БД

SQL-миграции расположены в `docs/sql/`. Применяются при запуске backend или вручную через `sqlx`.

## Структура тестов

### Backend

Тесты пишутся с использованием `#[cfg(test)]` модулей и интеграционных тестов в `tests/`:

```bash
cargo test
```

### Frontend

```bash
cd frontend
npm run test
```

## Стиль кода

### Rust

- Форматирование: `cargo fmt`
- Линтер: `cargo clippy`
- Асинхронный код: Tokio runtime
- Обработка ошибок: `anyhow` для приложения, `AppError` для HTTP

### TypeScript

- Форматирование: Prettier
- Линтер: ESLint
- Строгая типизация: `strict: true`

## Рекомендации по разработке

1. **Новые API-эндпоинты** — добавить handler в `handlers/`, модель в `models/`, сервис в `services/`
2. **Новые таблицы** — создать SQL-миграцию, добавить модель в `models/`
3. **Новые страницы** — создать директорию в `frontend/src/routes/`
4. **Новые компоненты** — разместить в `frontend/src/lib/components/`
