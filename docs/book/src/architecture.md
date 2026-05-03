# Архитектура системы

## Контекст системы (System Context)

```mermaid
graph TB
    User[Фермер / Зоотехник]
    Admin[Администратор]
    System[Milk Farm System]
    Lely[Lely Horizon API]
    
    User -->|Браузер| System
    Admin -->|Браузер| System
    System -->|REST/HTTPS| Lely
    Lely -->|Данные сенсоров| System
```

Система взаимодействует с двумя типами пользователей (через браузер) и внешней платформой Lely Horizon для получения данных с доильных роботов и датчиков.

## Контейнеры (Container Diagram)

```mermaid
graph TB
    User[Пользователь]
    
    subgraph Milk Farm
        FE[Frontend<br/>SvelteKit SPA]
        BE[Backend API<br/>Rust / Axum]
        ML[ML Service<br/>Python / FastAPI]
        DB[(PostgreSQL<br/>Основная БД)]
        CH[(ClickHouse<br/>Аналитика)]
        RD[(Redis<br/>Кэш / Rate Limit)]
    end
    
    Lely[Lely Horizon API]
    
    User -->|HTTPS| FE
    FE -->|REST API| BE
    BE -->|SQL| DB
    BE -->|SQL| CH
    BE -->|Redis Protocol| RD
    BE -->|HTTPS| Lely
    BE -->|HTTP| ML
    ML -->|SQL| DB
```

## Компоненты Backend

```mermaid
graph TB
    subgraph HTTP Layer
        REQ[HTTP Request]
    end
    
    subgraph Middleware Stack
        TRACE[TraceLayer — логирование]
        COMP[CompressionLayer — сжатие]
        CORS[CorsLayer — CORS]
        BODY[BodyLimitLayer — лимит тела]
        CSRF[CsrfLayer — защита от CSRF]
        RL[RateLimitLayer — ограничение запросов]
        RID[RequestIdLayer — идентификатор запроса]
        MET[MetricsLayer — метрики Prometheus]
    end
    
    subgraph Handlers
        AUTH_H[auth — аутентификация]
        ANIMAL_H[animals — животные]
        MILK_H[milk — надои]
        REPRO_H[reproduction — воспроизводство]
        FEED_H[feed — кормление]
        REPORT_H[reports — отчёты]
        ALERT_H[alerts — оповещения]
        SETTINGS_H[settings — настройки]
        LELY_H[lely — интеграция Lely]
    end
    
    subgraph Services
        AUTH_S[auth_service]
        LELY_S[lely::service]
        ALERT_S[alert_engine]
        REPORT_S[report_service]
        SYNC[lely::sync — планировщик]
    end
    
    subgraph Data
        POOL[SQLx PgPool]
        REDIS[Redis]
        LELY_CL[LelyClient]
    end
    
    REQ --> TRACE --> COMP --> CORS --> BODY --> CSRF --> RL --> RID --> MET
    MET --> AUTH_H & ANIMAL_H & MILK_H & REPRO_H & FEED_H & REPORT_H & ALERT_H & SETTINGS_H & LELY_H
    AUTH_H --> AUTH_S
    ANIMAL_H & MILK_H & REPRO_H & FEED_H --> POOL
    ALERT_H --> ALERT_S
    REPORT_H --> REPORT_S
    LELY_H --> LELY_S
    SYNC --> LELY_CL
    LELY_S --> POOL
    AUTH_S --> POOL & REDIS
    ALERT_S --> POOL
    REPORT_S --> POOL
```

## Компоненты Frontend

```mermaid
graph TB
    subgraph SvelteKit Application
        LAYOUT[+layout.svelte<br/>Глобальный макет]
        
        subgraph Страницы
            DASH[Dashboard]
            ANIMALS[Животные]
            MILK_P[Надои]
            REPRO_P[Воспроизводство]
            FEED_P[Кормление]
            REPORTS_P[Отчёты]
            ALERTS_P[Оповещения]
            SETTINGS_P[Настройки]
            AUTH_P[Авторизация]
        end
        
        subgraph Stores
            AUTH_STORE[auth store<br/>JWT, пользователь]
            THEME[theme store]
            NOTIFY[notifications store]
        end
        
        subgraph API Layer
            CLIENT[api/client.ts<br/>Базовый HTTP-клиент]
            ANIMALS_API[animals.ts]
            MILK_API[milk.ts]
            REPRO_API[reproduction.ts]
            FEED_API[feed.ts]
        end
    end
    
    LAYOUT --> DASH & ANIMALS & MILK_P & REPRO_P & FEED_P & REPORTS_P & ALERTS_P & SETTINGS_P & AUTH_P
    DASH --> CLIENT
    ANIMALS --> ANIMALS_API --> CLIENT
    MILK_P --> MILK_API --> CLIENT
    REPRO_P --> REPRO_API --> CLIENT
    FEED_P --> FEED_API --> CLIENT
    CLIENT -->|REST API| BACKEND[Backend API]
    AUTH_P --> AUTH_STORE
```

## Диаграмма развёртывания

```mermaid
graph TB
    subgraph Сервер
        NGINX[Nginx<br/>Reverse Proxy]
        BE_PROC[Backend Process<br/>Rust binary]
        FE_BUILD[Frontend Build<br/>Статические файлы]
        PG_INST[(PostgreSQL<br/>:5432)]
        CH_INST[(ClickHouse<br/>:8123)]
        RD_INST[(Redis<br/>:6379)]
    end
    
    subgraph ML Service
        ML_PROC[FastAPI<br/>:8000]
    end
    
    subgraph Monitoring
        PROM[Prometheus]
        GRAF[Grafana]
    end
    
    subgraph Внешние
        LELY_EXT[Lely Horizon]
    end
    
    CLIENT[Браузер] -->|HTTPS| NGINX
    NGINX -->|/| FE_BUILD
    NGINX -->|/api/v1| BE_PROC
    NGINX -->|/metrics| PROM
    BE_PROC --> PG_INST
    BE_PROC --> CH_INST
    BE_PROC --> RD_INST
    BE_PROC -->|HTTPS| LELY_EXT
    BE_PROC -->|HTTP| ML_PROC
    ML_PROC --> PG_INST
    PROM -->|scrape| BE_PROC
    GRAFANA_USER[Оператор] --> GRAF
    GRAF --> PROM
```

## Потоки данных

### Синхронизация с Lely

```mermaid
sequenceDiagram
    participant S as Scheduler
    participant DB as PostgreSQL
    participant LC as LelyClient
    participant LA as Lely API
    participant AE as Alert Engine
    
    S->>DB: Попытка получить блокировку
    DB-->>S: Lock acquired
    loop Для каждой сущности
        S->>DB: Получить дату последней синхронизации
        DB-->>S: last_synced_at
        S->>LC: Запрос данных (from, till)
        LC->>LA: HTTP GET
        LA-->>LC: JSON records
        LC-->>S: Vec<Record>
        S->>DB: UPSERT записей
    end
    S->>DB: Обновить sync_state
    S->>DB: Освободить блокировку
    AE->>DB: Проверка правил оповещений
    AE->>DB: Создать/обновить alerts
```

### Аутентификация

```mermaid
sequenceDiagram
    participant U as Пользователь
    participant FE as Frontend
    participant BE as Backend
    participant DB as PostgreSQL
    participant R as Redis
    
    U->>FE: Логин + пароль
    FE->>BE: POST /api/v1/auth/login
    BE->>DB: Проверка пароля (bcrypt)
    DB-->>BE: OK
    BE->>BE: Создать access + refresh JWT
    BE-->>FE: Set-Cookie: token, refresh_token
    FE-->>U: Перенаправление на Dashboard
    
    Note over FE,BE: Последующие запросы
    FE->>BE: GET /api/v1/animals<br/>Cookie: token=...
    BE->>BE: Проверить JWT (decode + revocation)
    BE->>DB: Проверить, не отозван ли токен
    DB-->>BE: OK
    BE-->>FE: 200 OK + данные
```

## Принятые архитектурные решения

| Решение | Обоснование |
|---------|-------------|
| Rust для backend | Производительность, безопасность типов, минимальное потребление ресурсов |
| Axum | Асинхронный web-фреймворк, хорошо интегрируется с экосистемой Tower |
| SvelteKit для frontend | Компактный рантайм, отличная производительность, SSR из коробки |
| PostgreSQL | Надёжная реляционная СУБД, поддержка JSON, полнотекстовый поиск |
| ClickHouse | Колонковая СУБД для быстрых аналитических запросов по большим объёмам данных |
| Redis | Кэширование сессий, rate limiting, in-memory счётчики |
| JWT + HttpOnly cookies | Безопасная аутентификация без уязвимостей XSS |
