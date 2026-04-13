# Система управления молочным животноводством на базе доильных роботов, совместимых с Lely Astronaut

> Ориентир для рассказа по слайдам презентации.  
> Каждый раздел соответствует слайду (или группе слайдов).

---

## 1. Актуальность

Молочное животноводство — одна из ключевых отраслей сельского хозяйства. При росте поголовья (300+ коров) ручной учёт становится невозможен:

- **Коммерческие решения** (Lely Horizon, DeLaval DelPro) — дорогие, закрытые, привязаны к оборудованию конкретного вендора.
- **Электронные таблицы** — ручной ввод, нет аналитики, нет интеграции с доильными роботами, высокий риск ошибок.

Необходимо **доступное, расширяемое, self-hosted решение**, которое:

- Ведёт учёт по всем направлениям (животные, удои, кормление, воспроизводство).
- Интегрируется с роботизированными системами доения Lely Astronaut.
- Предоставляет аналитику и генерацию отчётов.
- Не требует лицензионных отчислений.

---

## 2. Цель и задачи

### Цель

Разработка веб-приложения для комплексного управления молочной фермой с интеграцией роботизированных систем доения Lely Astronaut.

### Задачи

1. **Спроектировать архитектуру** — REST API + SPA, контейнеризация, reverse proxy.
2. **Спроектировать БД** — 25+ таблиц, covering все предметные области.
3. **Реализовать backend** — Rust/Axum, 15 модулей handlers, 20 сервисов, 4 middleware.
4. **Реализовать frontend** — SvelteKit 5, 15 страниц, 16 UI компонентов.
5. **Обеспечить интеграцию с Lely Cloud API** — OAuth2, синхронизация каждые 5 мин, AES-256-GCM шифрование credentials.
6. **Реализовать аналитику и отчёты** — 17 отчётов с экспортом в CSV/PDF, KPI, тренды, прогнозы.
7. **Настроить развёртывание** — Docker Compose (3 сервиса), Nginx, SSL, CI/CD через GHCR.

---

## 3. Постановка задачи

### Учёт данных

| Область | Данные |
|---|---|
| **Животные** | CRUD 300+ голов, timeline, batch deactivate, CSV import |
| **Удои** | дневные надои, визиты к роботу, качество молока (жир, белок, SCC) |
| **Кормление** | рационы, визиты к кормовому роботу, типы кормов, группы |
| **Воспроизводство** | осеменения, беременности, отёлы, охоты, сухостой (5 этапов) |
| **Локации** | CRUD помещений и пастбищ |
| **Контакты** | ветеринары, зоотехники, поставщики |
| **Bulk Tank** | записи о сборном молоке |
| **Фитнес** | активность, жвачка, ходьба/стояние |
| **Выпас** | данные о пастьбе |

### Аналитика и интеграция

- **17 отчётов** с экспортом CSV/PDF: молоко, воспроизводство, кормление, здоровье вымени, лактация, эффективность робота и др.
- **Аналитика**: KPI dashboard, тренды надоев, прогноз воспроизводства, прогноз кормления, алерты.
- **Lely**: автоматическая синхронизация данных с доильными роботами.
- **Роли**: user (просмотр + CRUD данных) / admin (+ управление пользователями, система, безопасность, Lely).
- **Развёртывание**: Docker Compose — один `docker compose up`.

---

## 4. Стек технологий

### Backend (19 679 строк)

| Компонент | Технология | Версия | Назначение |
|---|---|---|---|
| Язык | **Rust** | Edition 2024 | Memory safety, zero-cost abstractions, производительность |
| Фреймворк | **Axum** | 0.8 | Async HTTP framework (Tower ecosystem) |
| Runtime | **Tokio** | 1.x | Async runtime (multi-thread) |
| БД драйвер | **SQLx** | 0.8 | Compile-time проверка SQL, connection pooling |
| БД | **PostgreSQL** | 16 | Реляционная БД, 25+ таблиц |
| Сериализация | **Serde** + **Serde JSON** | 1.x | JSON serialization/deserialization |
| Аутентификация | **jsonwebtoken** + **bcrypt** | 9.x / 0.17 | JWT tokens + хеширование паролей |
| HTTP middleware | **Tower HTTP** | 0.6 | CORS, tracing, gzip compression |
| Документация API | **utoipa** + **utoipa-swagger-ui** | 5.4 / 9.0 | OpenAPI 3.0 spec + Swagger UI |
| Метрики | **Prometheus** | 0.13 | Экспорт метрик в Prometheus format |
| PDF генерация | **genpdf** | 0.2 | Генерация PDF отчётов |
| HTTP клиент | **reqwest** | 0.12 | Lely API integration (rustls-tls) |
| Шифрование | **aes-gcm** | 0.10 | AES-256-GCM для Lely credentials |
| Логирование | **tracing** + **tracing-subscriber** | 0.1 / 0.3 | Structured logging (JSON формат) |
| CLI | **clap** | 4.x | CLI аргументы (seed binary) |
| UUID | **uuid** | 1.x | Генерация UUID v4 |
| Время | **chrono** | 0.4 | Дата/время с timezone |
| Ошибки | **thiserror** + **anyhow** | 2.x / 1.x | Типизированные ошибки |

**27 production зависимостей** — минимум внешних зависимостей, стабильные crates.

### Frontend (12 707 строк)

| Компонент | Технология | Версия | Назначение |
|---|---|---|---|
| Фреймворк | **SvelteKit** | 2.50+ | Fullstack framework (SSR + SPA) |
| UI библиотека | **Svelte** | 5.54+ | Runes: `$state`, `$derived`, `$effect`, `$props` |
| Стили | **Tailwind CSS** | 4.2 | Utility-first CSS |
| Графики | **Chart.js** | 4.5 | Milk yield trends, KPI charts |
| Иконки | **Lucide Svelte** | 1.0+ | 1600+ SVG иконок |
| Сборка | **Vite** | 7.3 | Fast HMR, tree-shaking |
| Типы | **TypeScript** | 5.9 | Статическая типизация |
| Адаптер | **@sveltejs/adapter-node** | 5.5 | Node.js SSR production server |
| Тесты | **Vitest** + **Testing Library** | 4.1 / 5.3 | Unit/component testing |
| Линтинг | **ESLint** + **Prettier** | 10.x / 3.x | Code quality |

### Infrastructure

| Компонент | Технология | Назначение |
|---|---|---|
| Контейнеризация | **Docker Compose** | 3 сервиса: postgres, backend, frontend |
| Reverse Proxy | **Nginx** | SSL termination, маршрутизация, rate limiting |
| Registry | **GHCR** | Container registry, CI/CD |

**Итого: ~32 400 строк кода** (backend + frontend).

---

## 5. Архитектура: контекст системы (System Context)

На диаграмме показаны 3 внешних актёра и система:

1. **Фермер** (роль `user`) — просмотр отчётов, данных о животных, надоев, кормления. Не имеет доступа к управлению пользователями и настройкам системы.
2. **Администратор** (роль `admin`) — полный CRUD всех данных, управление пользователями, настройки системы (JWT TTL, alert thresholds, backup), конфигурация Lely, просмотр Prometheus метрик.
3. **Lely Cloud API** (`farmauth.lely.com`) — внешний сервис, данные от роботизированных систем доения Lely Astronaut. Система подключается по HTTPS, синхронизация каждые 5 минут.

Внутренняя система — **Milk Farm System** — единое веб-приложение.

---

## 6. Архитектура: контейнеры (C4 Container)

4 контейнера в Docker Compose:

| Контейнер | Образ | Порт | Память | Назначение |
|---|---|---|---|---|
| **Nginx** | `nginx:alpine` | 80 → 443 | — | Reverse proxy, SSL termination, rate limiting. Маршрутизация: `/*` → frontend, `/api/*` → backend |
| **Frontend** | Node.js 20 | 3000 | 256 MB | SvelteKit SSR server. Отдаёт HTML с server-side rendered данными, затем hydrate на клиенте |
| **Backend** | Rust binary | 3000 | 256 MB | REST API `/api/v1`. JWT auth, бизнес-логика, PDF генерация, Lely sync |
| **PostgreSQL** | postgres:16 | 5432 | 512 MB | 25+ таблиц, connection pool 2–20 connections. Persistent volume `pgdata` |

**Потоки данных:**
- Browser → Nginx (HTTPS) → Frontend (SSR) или Backend (API)
- Frontend → Backend (SSR fetch `/api/v1/*` при server-side rendering)
- Backend → PostgreSQL (SQLx PgPool, prepared statements, compile-time checked)
- Backend → Lely Cloud (reqwest HTTPS, OAuth2, периодический sync)

---

## 7. Backend: компоненты

4 слоя архитектуры backend:

### 1. Middleware Stack (7 middleware)

Порядок выполнения (снаружи внутрь):

1. **Tracing** — structured logging в JSON формате (request method, path, status, duration)
2. **Gzip compression** — автоматическое сжатие ответов
3. **CORS** — настраиваемый через `CORS_ORIGINS` env var
4. **Rate Limiting** — 100 запросов / 60 сек (in-memory, сконфигурировано через env)
5. **Request ID** — уникальный UUID для каждого запроса, пробрасывается в логи
6. **Prometheus Metrics** — счётчики запросов по endpoint, latency histogram (доступен по `/metrics`, только для admin)
7. **JWT Auth** — проверка access token из HttpOnly cookie или Bearer header. Извлекает Claims, проверяет expiry и revocation list

### 2. REST API — 15 handler модулей, 132 async fn

| Модуль | Кол-во функций | Ключевые операции |
|---|---|---|
| `auth.rs` | 8 | login, register, logout, refresh, health, healthz, readyz, stats |
| `animals.rs` | 9 | CRUD, timeline, stats, batch deactivate, CSV import |
| `milk.rs` | 7 | day-productions CRUD, visits, quality |
| `feed.rs` | 11 | day-amounts, visits, types CRUD, groups CRUD |
| `reproduction.rs` | 26 | 5 сущностей CRUD (insem, pregnancy, calving, heat, dry) + status |
| `reports.rs` | 29 | 17 отчётов + CSV/PDF экспорт |
| `analytics.rs` | 6 | KPI, alerts, milk-trend, repro-forecast, feed-forecast, latest-milk |
| `settings.rs` | 15 | users CRUD, role, password reset, preferences, system-info, JWT TTL, alerts, backup |
| `lely.rs` | 5 | status, sync, config CRUD, test-connection |
| `locations.rs` | 4 | CRUD |
| `contacts.rs` | 4 | CRUD |
| `bulk_tank.rs` | 5 | CRUD |
| `fitness.rs` | 2 | activities, ruminations |
| `grazing.rs` | 1 | grazing data |

**Всего ~97 route registrations** на `/api/v1/*`.

### 3. Service Layer — 20 сервисов

| Сервис | Назначение |
|---|---|
| `animal_service` | CRUD животных, batch, CSV parse |
| `animal_stats_service` | Статистика по животному |
| `milk_service` | Удои, визиты, качество |
| `feed_service` | Кормление, рационы |
| `reproduction_service` | Воспроизводство (5 сущностей) |
| `reports_service` | 17 отчётов, агрегация SQL |
| `analytics_service` | KPI, тренды, прогнозы |
| `user_service` | CRUD пользователей |
| `auth` (встроен в middleware) | JWT generation/validation, bcrypt, token revocation |
| `token_revocation_service` | Blacklist отозванных refresh tokens |
| `preferences_service` | Пользовательские настройки (theme, page size) |
| `system_settings_service` | Системные настройки (JWT TTL, alert thresholds) |
| `timeline_service` | Timeline событий для животного |
| `lely_service` (client, mapper, sync, crypto) | Lely API: OAuth2, data mapping, periodic sync, AES-256-GCM шифрование |
| `pdf_service` | Генерация PDF через genpdf |
| `bulk_tank_service` | CRUD сборного молока |
| `contact_service` | CRUD контактов |
| `location_service` | CRUD локаций |
| `fitness_service` | Активность, жвачка |
| `grazing_service` | Данные выпаса |
| `stats_service` | Общая статистика системы |
| `retry` | Retry logic для внешних API вызовов |

### 4. Data Layer

- **SQLx PgPool** — connection pool (2–20 connections)
- **Compile-time SQL verification** — макросы `sqlx::query!` проверяют SQL при `cargo build`
- **Prepared statements** — автоматическое кэширование
- **Миграции** — `sqlx::migrate!` из кода, up + down

---

## 8. Backend: API endpoints (примеры)

### Auth

```
POST /api/v1/auth/login      → { access_token, user }
POST /api/v1/auth/register   → { access_token, user }
POST /api/v1/auth/logout     → 204
POST /api/v1/auth/refresh    → { access_token }
```

JWT access token: HttpOnly + Secure cookie,短期 (~15 мин).  
Refresh token: HttpOnly cookie, долгосрочный, с revocation list.

### Animals

```
GET    /api/v1/animals              → { data: [...], total, page, per_page }
POST   /api/v1/animals              → { data: animal }
GET    /api/v1/animals/{id}         → { data: animal }
PUT    /api/v1/animals/{id}         → { data: animal }
DELETE /api/v1/animals/{id}         → 204
POST   /api/v1/animals/batch/deactivate  → { deactivated_count }
POST   /api/v1/animals/import/csv         → { imported, errors }
GET    /api/v1/animals/{id}/timeline → { data: [...] }
GET    /api/v1/animals/{id}/stats   → { data: {...} }
```

### Reports (17 отчётов)

```
GET /api/v1/reports/milk-summary
GET /api/v1/reports/reproduction-summary
GET /api/v1/reports/feed-summary
GET /api/v1/reports/herd-overview
GET /api/v1/reports/rest-feed
GET /api/v1/reports/robot-performance
GET /api/v1/reports/failed-milkings
GET /api/v1/reports/udder-health-worklist
GET /api/v1/reports/udder-health-analyze
GET /api/v1/reports/milk-day-production-time
GET /api/v1/reports/visit-behavior
GET /api/v1/reports/calendar
GET /api/v1/reports/health-activity-rumination
GET /api/v1/reports/cow-robot-efficiency
GET /api/v1/reports/lactation-analysis
GET /api/v1/reports/feed-per-type-day
GET /api/v1/reports/feed-per-cow-day
GET /api/v1/reports/health-task
GET /api/v1/reports/pregnancy-rate
GET /api/v1/reports/transition

GET /api/v1/reports/export/{report_type}/csv   → CSV file
GET /api/v1/reports/export/{report_type}/pdf    → PDF file
```

### Analytics

```
GET /api/v1/analytics/kpi                   → KPI dashboard
GET /api/v1/analytics/alerts                 → Alert list
GET /api/v1/analytics/milk-trend             → Milk yield trend
GET /api/v1/analytics/reproduction-forecast  → Repro forecast
GET /api/v1/analytics/feed-forecast          → Feed forecast
GET /api/v1/analytics/latest-milk            → Latest milk data
```

Все API endpoints документированы через **OpenAPI 3.0** (utoipa), Swagger UI доступен по `/api/v1/docs`.

---

## 9. Frontend: компоненты

### hooks.server.ts

- **JWT decode** — проверяет access token в cookie при каждом SSR запросе
- **Auth guard** — если нет валидного token и маршрут не `/login`, редиректит
- **Передаёт user info** в `event.locals` для использования в load functions

### 15 страниц (routes/)

| Маршрут | Страница | SSR load | Ключевые возможности |
|---|---|---|---|
| `/` | Dashboard | `+page.ts` | KPI карточки, графики Chart.js, alerts |
| `/animals` | Animals | `+page.ts` | DataTable, CRUD, batch deactivate, CSV import |
| `/animals/{id}` | Animal detail | `+page.ts` | Timeline, stats, edit form |
| `/milk` | Milk | `+page.ts` | 3 вкладки: yield, visits, quality |
| `/feed` | Feed | `+page.ts` | 4 вкладки: rations, visits, types, groups |
| `/reproduction` | Reproduction | `+page.ts` | 5 вкладок: insem, pregnancy, calving, heat, dry |
| `/reports` | Reports | `+page.ts` | 17 вкладок, CSV/PDF export |
| `/settings` | Settings | `+page.ts` | 7 вкладок: users, JWT, alerts, backup, password |
| `/locations` | Locations | `+page.ts` | CRUD |
| `/contacts` | Contacts | `+page.ts` | CRUD |
| `/bulk-tank` | Bulk Tank | `+page.ts` | CRUD |
| `/fitness` | Fitness | `+page.ts` | Activity, rumination |
| `/grazing` | Grazing | `+page.ts` | Grazing data |

Каждая страница имеет **SSR load function** (`+page.ts`) — данные загружаются на сервере, пользователь видит готовый HTML без waiting spinner.

### 16 UI компонентов (lib/components/ui/)

| Компонент | Назначение |
|---|---|
| **DataTable** | Универсальная таблица: sorting, pagination, hidden columns на мобильных, row click |
| **Modal** | Модальное окно с формой |
| **ConfirmDialog** | Подтверждение удаления |
| **FormField** | Label + input с валидацией |
| **FilterBar** | Фильтры для DataTable |
| **Pagination** | Постраничная навигация |
| **TabBar** | Вкладки (ARIA accessible: role=tablist/tab/tabpanel) |
| **EmptyState** | Заглушка при пустых данных |
| **ErrorAlert** | Отображение ошибок |
| **Toaster** | Уведомления: success, error, warning, info + expandable details |
| **AnimalSelect** | Поиск и выбор животного |

### Stores (lib/stores/)

| Store | Назначение |
|---|---|
| `auth.ts` | login/logout/refresh, authenticated state |
| `preferences.ts` | theme (light/dark), page_size |
| `toast.ts` | 4 типа уведомлений + details |
| `theme.ts` | CSS class переключение |

### API клиент (lib/api/) — 14 модулей

Каждый модуль — набор типизированных функций для работы с конкретным API endpoint:

`client.ts` (базовый fetch wrapper) → `animals.ts`, `milk.ts`, `feed.ts`, `reproduction.ts`, `reports.ts`, `analytics.ts`, `auth.ts`, `settings.ts`, `locations.ts`, `contacts.ts`, `lely.ts`, `bulk-tank.ts`, `fitness.ts`

### Utils (lib/utils/)

- `validators.ts` — `useFormValidation` (required, min, max, pattern)
- `useCrudModal` — универсальный CRUD: open/create/edit/delete с формой
- `usePaginatedList` — загрузка с пагинацией + auto-refresh
- `format.ts` — форматирование дат, чисел

---

## 10. Frontend: SvelteKit 5 Runes

Svelte 5 ввёл **runes** — новый реактивный примитив:

| Rune | Аналог | Использование |
|---|---|---|
| `$state()` | `let` (reactive) | Реактивное состояние: `let name = $state('')` |
| `$derived()` | `$:` | Вычисляемые значения: `let filtered = $derived(items.filter(...))` |
| `$effect()` | Reactive statement | Побочные эффекты: fetch при изменении параметров |
| `$props()` | `export let` | Типобезопасные входные параметры компонента |

**Примеры использования:**

- **DataTable**: `$derived` для сортировки/фильтрации, `$state` для currentPage/sortColumn
- **CRUD модалки**: `$state` для form values, `$effect` для сброса при открытии
- **Reports**: 17 tab компонентов, каждый с `$state` для параметров отчёта
- **Sidebar**: `$derived(auth)` для показа/скрытия admin пунктов

**SSR паттерн:**
```
+page.ts (load function) → fetch /api/v1/... → return { initialData }
+page.svelte → let { data } = $props() → render с SSR
```

**Responsive design:**
- DataTable: `hideOnMobile` prop скрывает столбцы на экранах < 768px
- Sidebar: collapsible на мобильных
- TabBar: horizontal scroll на узких экранах

---

## 11. База данных

### 25+ таблиц, 11 миграций

| Миграция | Содержание |
|---|---|
| `001_init` | Базовые таблицы: animals, milk_yield, milk_visits, milk_quality, feed_*, reproduction_*, users, refresh_tokens, bulk_tank_records |
| `002_seed_admin` | Начальный admin пользователь |
| `003_must_change_password` | Флаг принудительной смены пароля |
| `004_performance_indexes` | Индексы для часто используемых запросов |
| `005_settings` | system_settings, user_preferences таблицы |
| `006_revoked_tokens` | token_revocation_list для logout |
| `007_pg_trgm` | pg_trgm extension для поиска |
| `008_lely_support` | lely_sync_log таблица |
| `009_lely_config` | lely_config таблица |
| `010_lely_fixes` | Исправления Lely схемы |
| `011_missing_indexes` | Дополнительные индексы |

**Каждая миграция имеет down-migration** — полный откат на предыдущую версию.

### Ключевые таблицы

- `animals` — основная таблица (tag_number, name, breed, birth_date, status, location_id)
- `milk_yield` / `milk_visits` / `milk_quality` — учёт надоев
- `feed_day_amounts` / `feed_visits` / `feed_types` / `feed_groups` — кормление
- `inseminations` / `pregnancies` / `calvings` / `heats` / `dry_periods` — воспроизводство
- `users` / `refresh_tokens` / `token_revocation_list` — аутентификация
- `locations` / `contacts` — справочники
- `bulk_tank_records` — сборное молоко
- `lely_config` / `lely_sync_log` — интеграция с Lely
- `system_settings` / `user_preferences` — настройки

### Seed binary

Отдельный бинарник `cargo run --bin seed`:
- Автоматически применяет миграции (up)
- Генерирует mock-данные: 300 коров, 3 года истории
- Используется для разработки, тестирования, демонстрации

---

## 12. Безопасность

### Аутентификация

| Механизм | Детали |
|---|---|
| **Access token** | JWT, 15 мин TTL (настраиваемый), содержит `{ sub, role, exp }` |
| **Refresh token** | JWT, 7 дней, хранится в БД, поддерживает revocation list |
| **Cookies** | `HttpOnly` + `Secure` (production) + `SameSite=Lax` |
| **Пароли** | bcrypt (cost factor 12) |
| **Token revocation** | При logout refresh token добавляется в blacklist в БД |
| **Роли** | `user` — данные; `admin` — + users, settings, Lely, metrics |
| **AdminGuard** | Middleware проверяет `role == "admin"` для защищённых endpoints |

### Защита

| Механизм | Детали |
|---|---|
| **Rate limiting** | 100 req/60s (in-memory, настраиваемый через env) |
| **CORS** | Whitelist origins через `CORS_ORIGINS` env var |
| **Encryption** | AES-256-GCM для Lely OAuth2 credentials (client_secret) |
| **LELY_ENCRYPTION_KEY** | Обязательный env var, 32 байта, при отсутствии — panic |
| **Metrics auth** | `/metrics` endpoint доступен только admin |
| **Request ID** | Уникальный UUID для трассировки в логах |
| **SQL injection** | SQLx compile-time проверка, parameterized queries |
| **XSS** | Svelte auto-escaping, HttpOnly cookies |

---

## 13. Тестирование

### 234 теста

| Категория | Кол-во | Что тестируют |
|---|---|---|
| Auth | ~30 | login, register, refresh, logout, invalid credentials, role checks |
| Animals | ~25 | CRUD, pagination, filtering, batch deactivate, CSV import |
| Milk | ~20 | day-productions CRUD, visits, quality |
| Feed | ~20 | day-amounts, types/groups CRUD, visits |
| Reproduction | ~30 | 5 сущностей CRUD, status endpoint |
| Reports | ~33 | 17 отчётов + CSV/PDF экспорт |
| Analytics | ~8 | KPI, alerts, trends, forecasts |
| Lely | ~11 | config, status, sync, test-connection |
| Locations | ~9 | CRUD |
| Settings | ~15 | users, preferences, JWT TTL, alerts, backup, password reset |
| Unit tests | ~33 | Services, models, validators |

### Инструменты

- **cargo test** — integration + unit тесты (тестовая БД создаётся автоматически)
- **cargo clippy** — линтер Rust (0 warnings)
- **svelte-check** — TypeScript type checking Svelte компонентов
- **Vitest** — unit тесты для frontend
- **Seed binary** — воспроизводимые тестовые данные

### CI pipeline

```bash
cargo test          # 234 теста
cargo clippy        # 0 warnings
svelte-check        # TypeScript типы
```

---

## 14. Развёртывание (Docker Compose)

### Архитектура развёртывания

```
Internet → Nginx (:443 SSL) → Frontend (:3000)
                              → Backend  (:3000) → PostgreSQL (:5432)
                              → Lely Cloud (HTTPS)
```

### 3 сервиса

```yaml
services:
  postgres:       # postgres:16, 512MB, pgdata volume, healthcheck
  backend:        # Rust binary, 256MB, env vars, depends_on postgres
  frontend:       # SvelteKit Node.js, 256MB, depends_on backend
```

### Backend Dockerfile

Multi-stage build:
1. `rust:1.87-slim` — compile release binary (~2 мин)
2. `debian:bookworm-slim` — runtime (~50MB image)

### Frontend Dockerfile

Multi-stage build:
1. `node:20-slim` — `npm ci` + `npm run build`
2. `node:20-slim` — `npm run preview` (SvelteKit adapter-node)

### Production (docker-compose.prod.yml)

- `SECURE_COOKIES=true`
- `CORS_ORIGINS` = production domain
- `LELY_ENCRYPTION_KEY` = production key
- Nginx SSL termination
- Resource limits

---

## 15. Результаты

### Количественные показатели

| Метрика | Значение |
|---|---|
| Backend LOC | 19 679 |
| Frontend LOC | 12 707 |
| **Итого** | **~32 400** |
| API endpoints | ~97 |
| Handler функций | 132 |
| Service модулей | 20 |
| Middleware | 4 |
| Моделей | 17 |
| Миграций БД | 11 (up + down) |
| Frontend страниц | 15 |
| UI компонентов | 16 |
| API клиент модулей | 14 |
| Отчётов | 17 |
| Тестов | 234 |
| Docker сервисов | 3 |

### Качественные показатели

- **Производительность**: Rust backend (zero-cost abstractions, no GC pauses), SQLx prepared statements, connection pooling
- **Надёжность**: compile-time SQL verification, TypeScript типы, 234 теста
- **Безопасность**: JWT + bcrypt + AES-256-GCM + rate limiting + CORS + HttpOnly cookies
- **Расширяемость**: модульная архитектура (handlers/services/models), OpenAPI документация
- **UX**: SSR (быстрая загрузка), responsive design, тёмная тема, accessibility (ARIA)
- **Maintainability**: down-migrations, seed binary, structured logging, Prometheus metrics

---

## 16. Заключение

1. Разработана полнофункциональная система управления молочной фермой — от учёта данных до аналитики и отчётов.
2. Реализована интеграция с роботизированными доильными системами Lely Astronaut (OAuth2, периодическая синхронизация, шифрование credentials).
3. Современный стек: **Rust** (производительность, memory safety) + **SvelteKit 5** (SSR, reactive UI, TypeScript).
4. Production-ready: Docker Compose деплой, SSL, Prometheus metrics, 234 теста, OpenAPI документация.
5. Open-source, self-hosted, без лицензионных отчислений.
