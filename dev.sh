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
    esac
done

echo "Starting PostgreSQL..."
sudo systemctl start postgresql

echo "Ensuring database user and database exist..."
sudo -u postgres psql -tc "SELECT 1 FROM pg_roles WHERE rolname='$DB_USER'" | grep -q 1 || \
    sudo -u postgres psql -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASS';"
sudo -u postgres psql -lqt | cut -d \| -f 1 | grep -qw $DB_NAME || \
    sudo -u postgres psql -c "CREATE DATABASE $DB_NAME OWNER $DB_USER;"

echo "Seeding mock data (${SEED_COWS} cows, ${SEED_YEARS} years)..."
(cd backend && cargo run --bin seed -- --cows "$SEED_COWS" --years "$SEED_YEARS")

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
DATABASE_URL="postgresql+asyncpg://$DB_USER:$DB_PASS@localhost:5432/$DB_NAME" \
    "$ML_VENV/bin/uvicorn" app.main:app --host 0.0.0.0 --port $ML_PORT --app-dir analytics-ml || true &
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
    kill $BACKEND_PID $FRONTEND_PID $ML_PID 2>/dev/null
    wait $BACKEND_PID $FRONTEND_PID 2>/dev/null
    [ -n "$ML_PID" ] && wait $ML_PID 2>/dev/null
    exit 0
}
trap cleanup INT TERM

if [ "$ENABLE_MONITORING" = true ]; then
    echo "Starting monitoring stack..."
    docker compose -f docker-compose.monitoring.yml -f docker-compose.monitoring.dev.yml up -d
    echo "Monitoring: http://localhost:3001 (admin/admin)"
fi

echo "Starting backend..."
fuser -k 3000/tcp 2>/dev/null || true
(cd backend && cargo run --bin milk-farm-backend) &
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
(cd frontend && npm run dev) &
FRONTEND_PID=$!

echo ""
echo "    Running!"
echo "    Frontend:   http://localhost:5173"
echo "    Backend:    http://localhost:3000"
echo "    ML service: http://localhost:$ML_PORT"
echo "    Login: admin / admin"
if [ "$ENABLE_MONITORING" = true ]; then
echo "    Grafana:    http://localhost:3001 (admin/admin)"
echo "    Prometheus: http://localhost:9090"
fi
echo ""

wait
