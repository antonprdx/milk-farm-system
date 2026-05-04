#!/usr/bin/env bash
set -e

DB_USER="milkfarm"
DB_PASS="milkfarm"
DB_NAME="milkfarm"

SEED_COWS="${SEED_COWS:-100}"
SEED_YEARS="${SEED_YEARS:-1}"
ENABLE_MONITORING=false
RUN_BENCH=false
K6_TEST=""
SKIP_SEED=false
DEMO_MODE=false

for arg in "$@"; do
    case $arg in
        --monitoring|-m) ENABLE_MONITORING=true ;;
        --bench|-b) RUN_BENCH=true ;;
        --bench-sql) RUN_BENCH=true; BENCH_FILTER="bench_sql" ;;
        --bench-pdf) RUN_BENCH=true; BENCH_FILTER="bench_pdf" ;;
        --bench-auth) RUN_BENCH=true; BENCH_FILTER="bench_auth" ;;
        --bench-rate-limit) RUN_BENCH=true; BENCH_FILTER="bench_rate_limit" ;;
        --k6-smoke) K6_TEST="smoke" ;;
        --k6-load) K6_TEST="load" ;;
        --k6-stress) K6_TEST="stress" ;;
        --no-seed) SKIP_SEED=true ;;
        --demo|-d) DEMO_MODE=true ;;
    esac
done

echo "Starting PostgreSQL..."
sudo systemctl start postgresql

echo "Ensuring database user and database exist..."
sudo -u postgres psql -tc "SELECT 1 FROM pg_roles WHERE rolname='$DB_USER'" | grep -q 1 || \
    sudo -u postgres psql -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASS';"
sudo -u postgres psql -lqt | cut -d \| -f 1 | grep -qw $DB_NAME || \
    sudo -u postgres psql -c "CREATE DATABASE $DB_NAME OWNER $DB_USER;"

if [ "$SKIP_SEED" = true ]; then
    echo "Skipping seed (--no-seed). Running migrations only..."
    (cd backend && DATABASE_URL="postgresql://$DB_USER:$DB_PASS@localhost:5432/$DB_NAME" "$HOME/.cargo/bin/sqlx" migrate run --source ./migrations)
else
    echo "Seeding mock data (${SEED_COWS} cows, ${SEED_YEARS} years)..."
    (cd backend && cargo run --bin seed -- --cows "$SEED_COWS" --years "$SEED_YEARS")
fi

ML_PORT=8001
ML_VENV="analytics-ml/.venv"

echo "Setting up ML service..."
fuser -k 8001/tcp 2>/dev/null || true
if [ ! -d "$ML_VENV" ]; then
    echo "  Creating Python venv for analytics-ml..."
    python3 -m venv "$ML_VENV"
    "$ML_VENV/bin/pip" install -q --upgrade pip
fi
if ! "$ML_VENV/bin/python" -c "import fastapi" 2>/dev/null; then
    echo "  Installing ML dependencies..."
    PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 "$ML_VENV/bin/pip" install -q --upgrade pip
    PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 "$ML_VENV/bin/pip" install -q \
        "fastapi" "uvicorn[standard]" "sqlalchemy[asyncio]" "asyncpg" \
        "pydantic" "pydantic-settings" "xgboost" "scikit-learn" \
        "pandas<3" "numpy" "joblib" "httpx" "apscheduler"
    PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 "$ML_VENV/bin/pip" install -q \
        "opentelemetry-api" "opentelemetry-sdk" \
        "opentelemetry-instrumentation-fastapi" \
        "opentelemetry-exporter-otlp-proto-grpc" \
        2>/dev/null || true
    PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 "$ML_VENV/bin/pip" install -q \
        "onnxmltools" "onnxruntime" "skl2onnx" \
        2>/dev/null || true
    echo "  Done."
fi
ML_ENV="DATABASE_URL=postgresql+asyncpg://$DB_USER:$DB_PASS@localhost:5432/$DB_NAME"
if [ "$ENABLE_MONITORING" = true ]; then
    ML_ENV="$ML_ENV OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317"
fi
env $ML_ENV "$ML_VENV/bin/uvicorn" app.main:app --host 0.0.0.0 --port $ML_PORT --app-dir analytics-ml || true &
ML_PID=$!
sleep 2
if ! kill -0 $ML_PID 2>/dev/null; then
    echo "  WARNING: ML service failed to start, continuing without it"
    ML_PID=
fi

run_benchmarks() {
    echo ""
    echo "Running Rust benchmarks..."
    export DATABASE_URL="postgresql://$DB_USER:$DB_PASS@localhost:5432/$DB_NAME"
    if [ -n "$BENCH_FILTER" ]; then
        (cd backend && cargo bench --bench "$BENCH_FILTER")
    else
        (cd backend && cargo bench --bench bench_sql --bench bench_pdf --bench bench_auth --bench bench_rate_limit)
    fi
    unset DATABASE_URL
    echo "Benchmarks complete. Results in target/criterion/"
}

run_k6() {
    local test_name="$1"
    if ! command -v k6 &>/dev/null; then
        echo "ERROR: k6 is not installed. Install: https://k6.io/docs/get-started/installation/"
        exit 1
    fi

    echo ""
    echo "Waiting for backend to be ready for k6 test ($test_name)..."
    until curl -sf http://localhost:3000/api/v1/health > /dev/null 2>&1; do
        sleep 1
    done

    echo "Running k6 $test_name test..."
    k6 run \
        -e BASE_URL=http://localhost:3000 \
        -e ADMIN_PASSWORD=admin \
        --out json=tests/load/k6/results/${test_name}-$(date +%Y%m%d-%H%M%S).json \
        "tests/load/k6/${test_name}.js" || true
    echo "k6 $test_name test complete."
}

cleanup() {
    echo ""
    echo "Stopping..."
    if [ "$ENABLE_MONITORING" = true ]; then
        docker compose -f docker-compose.monitoring.yml -f docker-compose.monitoring.dev.yml down 2>/dev/null || true
    fi
    kill $BACKEND_PID $FRONTEND_PID $ML_PID $MOCK_LELY_PID 2>/dev/null
    wait $BACKEND_PID $FRONTEND_PID 2>/dev/null
    [ -n "$ML_PID" ] && wait $ML_PID 2>/dev/null
    [ -n "$MOCK_LELY_PID" ] && wait $MOCK_LELY_PID 2>/dev/null
    exit 0
}
trap cleanup INT TERM

if [ "$ENABLE_MONITORING" = true ]; then
    echo "Starting monitoring stack..."
    docker compose -f docker-compose.monitoring.yml -f docker-compose.monitoring.dev.yml up -d
    echo "Monitoring: http://localhost:3001 (admin/admin)"
fi

MOCK_LELY_PORT=1988
echo "Starting mock Lely server on port $MOCK_LELY_PORT..."
fuser -k $MOCK_LELY_PORT/tcp 2>/dev/null || true
(cd backend && cargo run --bin mock_lely -- --port $MOCK_LELY_PORT --cows "$SEED_COWS") &
MOCK_LELY_PID=$!

echo "Starting backend..."
fuser -k 3000/tcp 2>/dev/null || true
OTEL_EXPORTER_OTLP_ENDPOINT=""
LOG_DIR="/tmp/milk-farm-logs-$(id -u)"
mkdir -p "$LOG_DIR"
if [ "$ENABLE_MONITORING" = true ]; then
    OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
fi
DEMO_ENV=""
if [ "$DEMO_MODE" = true ]; then
    DEMO_ENV="DEMO_MODE=true"
fi
LOG_FILE="$LOG_DIR/backend.json"
env \
    LELY_ENABLED=true \
    LELY_BASE_URL="http://localhost:$MOCK_LELY_PORT" \
    LELY_USERNAME=mock \
    LELY_PASSWORD=mock \
    LELY_FARM_KEY=mock-farm \
    LELY_ENCRYPTION_KEY="01234567890123456789012345678901" \
    LELY_SYNC_INTERVAL_SECS=60 \
    OTEL_EXPORTER_OTLP_ENDPOINT="$OTEL_EXPORTER_OTLP_ENDPOINT" \
    RUST_LOG="info,milk_farm_backend=debug" \
    RUST_LOG_FORMAT="json" \
    $DEMO_ENV \
    bash -c 'cd backend && cargo run --bin milk-farm-backend 2>&1 | tee "$0"' "$LOG_FILE" &
BACKEND_PID=$!

echo "Waiting for backend to be ready..."
until curl -sf http://localhost:3000/api/v1/health > /dev/null 2>&1; do
    sleep 1
done
echo "Backend is ready."

if [ "$RUN_BENCH" = true ]; then
    run_benchmarks
    if [ "$K6_TEST" = "" ]; then
        cleanup
    fi
fi

if [ -n "$K6_TEST" ]; then
    mkdir -p tests/load/k6/results
    run_k6 "$K6_TEST"
    cleanup
fi

echo "Starting frontend..."
FRONTEND_ENV=""
FRONTEND_EXTRA_ARGS=""
if [ "$DEMO_MODE" = true ]; then
    FRONTEND_ENV="VITE_DEMO_MODE=true DEMO_MODE=true"
    FRONTEND_EXTRA_ARGS="--host 0.0.0.0 --port 8080"
fi
env $FRONTEND_ENV bash -c "cd frontend && npm run dev -- $FRONTEND_EXTRA_ARGS" &
FRONTEND_PID=$!

OPEN_BROWSER=true
for arg in "$@"; do
    case $arg in
        --no-open|-n) OPEN_BROWSER=false ;;
    esac
done

FRONTEND_PORT=5173
if [ "$DEMO_MODE" = true ]; then
    FRONTEND_PORT=8080
fi

echo ""
echo "    Running!"
echo "    Frontend:   http://localhost:$FRONTEND_PORT"
echo "    Backend:    http://localhost:3000"
echo "    Swagger:    http://localhost:3000/api/v1/docs"
echo "    Mock Lely:  http://localhost:$MOCK_LELY_PORT"
echo "    ML service: http://localhost:$ML_PORT"
if [ "$DEMO_MODE" = true ]; then
echo "    Demo mode:  ON (no login required, listening on 0.0.0.0:$FRONTEND_PORT)"
else
echo "    Login: admin / admin"
fi
if [ "$ENABLE_MONITORING" = true ]; then
echo "    Grafana:    http://localhost:3001 (admin/admin)"
echo "    Prometheus: http://localhost:9090"
fi
echo ""

if [ "$OPEN_BROWSER" = true ]; then
    sleep 3
    xdg-open "http://localhost:$FRONTEND_PORT" 2>/dev/null &
    xdg-open "http://localhost:3000/api/v1/docs" 2>/dev/null &
    if [ "$ENABLE_MONITORING" = true ]; then
        xdg-open "http://localhost:3001" 2>/dev/null &
        xdg-open "http://localhost:9090" 2>/dev/null &
    fi
fi

wait
