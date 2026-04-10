# Молочная ферма

Система управления молочной фермой — учёт животных, надоев, воспроизводства, кормления, фитнеса и пастьбы.

## Стек

| Слой | Технологии |
|------|-----------|
| Backend | Rust, Axum, SQLx, PostgreSQL |
| Frontend | SvelteKit 2, Svelte 5, Tailwind CSS 4, Chart.js |
| Инфра | Docker, docker-compose, GitHub Actions CI |

## Быстрый старт

### Локальная разработка (dev.sh)

Скрипт автоматически запускает PostgreSQL, сеет тестовые данные и поднимает оба сервера:

```bash
chmod +x dev.sh
./dev.sh
```

После запуска:
- Frontend: http://localhost:5173
- Backend: http://localhost:3000
- Логин: `admin` / `admin` (смена пароля при первом входе)

### Docker

```bash
docker-compose up -d
```

### Ручной запуск

1. Убедитесь, что PostgreSQL запущен и доступен
2. Скопируйте и настройте переменные окружения:

```bash
cp backend/.env.example backend/.env
```

3. Запустите backend:

```bash
cd backend
cargo run
```

4. Запустите frontend:

```bash
cd frontend
npm install
npm run dev
```

## Переменные окружения

См. `backend/.env.example`:

| Переменная | Описание | По умолчанию |
|-----------|----------|-------------|
| `DATABASE_URL` | Строка подключения PostgreSQL | — |
| `JWT_SECRET` | Секрет для JWT (минимум 32 символа) | — |
| `CORS_ORIGINS` | Разрешённые CORS-источники (через запятую) | `http://localhost:5173` |
| `HOST` | Адрес привязки backend | `0.0.0.0` |
| `PORT` | Порт backend | `3000` |
| `SECURE_COOKIES` | Использовать secure-куки (true для HTTPS) | `false` |
| `RUST_LOG` | Уровень логирования | `milk_farm_backend=debug` |

## Структура проекта

```
├── backend/           # Rust API сервер
│   ├── src/
│   │   ├── handlers/  # HTTP-обработчики
│   │   ├── services/  # Бизнес-логика
│   │   ├── models/    # Модели данных
│   │   ├── middleware/ # Аутентификация, авторизация
│   │   └── migrations/ # SQL-миграции
│   └── tests/         # Интеграционные тесты
├── frontend/          # SvelteKit SPA
│   ├── src/
│   │   ├── routes/    # Страницы
│   │   ├── lib/api/   # API-клиент
│   │   ├── lib/components/ # UI-компоненты
│   │   └── lib/utils/ # Утилиты
│   └── tests/         # Vitest тесты
├── docker-compose.yml
├── .github/workflows/ # CI/CD
└── dev.sh             # Скрипт локальной разработки
```

## API

API доступно по префиксу `/api/v1/`. Аутентификация — HttpOnly cookie с JWT.

## Тесты

```bash
# Backend
cd backend && cargo test

# Frontend
cd frontend && npm test
```
