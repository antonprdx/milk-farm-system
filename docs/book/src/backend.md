# Backend

Backend реализован на **Rust** с использованием web-фреймворка **Axum** и runtime **Tokio**.

## Структура модулей

```
backend/src/
├── lib.rs           # Создание приложения, middleware-стек
├── main.rs          # Точка входа, запуск сервера
├── config.rs        # Конфигурация (env, файлы)
├── state.rs         # AppState — разделяемое состояние
├── errors.rs        # Типы ошибок (AppError)
├── validation.rs    # Валидация входных данных
├── openapi.rs       # Спецификация OpenAPI
├── seed.rs          # Начальные данные
├── db/              # Подключение к БД
├── handlers/        # HTTP-обработчики (routes)
├── middleware/      # Промежуточное ПО
├── models/          # Модели данных
├── services/        # Бизнес-логика
├── lely/            # Интеграция с Lely Horizon
└── handlers/
    └── events.rs    # EventBus (Server-Sent Events)
```

## Основные зависимости

| Crate | Назначение |
|-------|-----------|
| `axum` | HTTP-фреймворк |
| `tokio` | Асинхронный runtime |
| `sqlx` | Асинхронный PostgreSQL драйвер |
| `serde` / `serde_json` | Сериализация/десериализация |
| `jsonwebtoken` | Создание и проверка JWT |
| `bcrypt` | Хеширование паролей |
| `utoipa` | Генерация OpenAPI-документации |
| `tower` / `tower-http` | Middleware (CORS, compression, tracing) |
| `prometheus` | Метрики |
| `redis` | Redis-клиент |
| `tracing` | Структурированное логирование |

## AppState

Разделяемое состояние приложения (`AppState`) содержит:

- `config: Config` — конфигурация из переменных окружения
- `pool: PgPool` — пул подключений к PostgreSQL
- `redis: Option<ConnectionManager>` — подключение к Redis (опционально)
- `lely: LelyState` — состояние интеграции с Lely

## Обработка ошибок

Все ошибки приводятся к типу `AppError`, который реализует `IntoResponse`:

```mermaid
graph LR
    E[AppError] --> U[401 Unauthorized]
    E --> F[403 Forbidden]
    E --> N[404 Not Found]
    E --> B[400 BadRequest]
    E --> R[429 RateLimited]
    E --> I[500 Internal]
    E --> D[500 Database]
```

## EventBus

Система использует in-process шину событий на основе **tokio::broadcast** для доставки оповещений клиенту через **Server-Sent Events (SSE)**:

```mermaid
graph LR
    H1[Handler 1] --> EB[EventBus<br/>broadcast channel]
    H2[Handler 2] --> EB
    H3[Alert Engine] --> EB
    EB --> SSE[SSE endpoint<br/>/api/v1/events]
```

## Маршрутизация

Все API-эндпоинты расположены под префиксом `/api/v1`:

| Группа | Префикс | Назначение |
|--------|---------|------------|
| Auth | `/auth` | Логин, регистрация, refresh, logout |
| Animals | `/animals` | CRUD животных |
| Milk | `/milk` | Надои, качество, визиты |
| Reproduction | `/reproduction` | Осеменения, стельности, отёлы |
| Feed | `/feed` | Кормление, рационы |
| Reports | `/reports` | Генерация отчётов |
| Alerts | `/alerts` | Оповещения |
| Settings | `/settings` | Настройки системы |
| Analytics | `/analytics` | Аналитические данные |
| Lely | `/lely` | Управление синхронизацией Lely |
| Health | `/healthz`, `/readyz` | Проверки работоспособности |
