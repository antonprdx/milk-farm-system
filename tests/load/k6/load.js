import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
    stages: [
        { duration: '30s', target: 10 },
        { duration: '1m',  target: 10 },
        { duration: '10s', target: 0 },
    ],
    thresholds: {
        http_req_duration: ['p(95)<500'],
        http_req_failed: ['rate<0.05'],
    },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';

export function setup() {
    const loginRes = http.post(`${BASE_URL}/api/v1/auth/login`, JSON.stringify({
        username: 'admin',
        password: __ENV.ADMIN_PASSWORD || 'admin',
    }), {
        headers: { 'Content-Type': 'application/json' },
    });

    check(loginRes, {
        'login succeeded': (r) => r.status === 200,
    });

    const cookie = loginRes.headers['Set-Cookie'] || '';
    const tokenMatch = cookie.match(/token=([^;]+)/);
    const token = tokenMatch ? tokenMatch[1] : '';

    return { token };
}

export default function (data) {
    const headers = {
        'Authorization': `Bearer ${data.token}`,
        'Content-Type': 'application/json',
    };

    const scenarios = [
        () => {
            const res = http.get(`${BASE_URL}/api/v1/animals?limit=20`, { headers });
            check(res, { 'animals list 200': (r) => r.status === 200 });
        },
        () => {
            const res = http.get(`${BASE_URL}/api/v1/analytics/kpi`, { headers });
            check(res, { 'analytics kpi 200': (r) => r.status === 200 });
        },
        () => {
            const res = http.get(`${BASE_URL}/api/v1/milk/day-productions?limit=20`, { headers });
            check(res, { 'milk productions 200': (r) => r.status === 200 });
        },
        () => {
            const res = http.get(`${BASE_URL}/api/v1/reproduction/calvings?limit=20`, { headers });
            check(res, { 'reproduction calvings 200': (r) => r.status === 200 });
        },
        () => {
            const res = http.get(`${BASE_URL}/api/v1/feed/day-amounts?limit=20`, { headers });
            check(res, { 'feed amounts 200': (r) => r.status === 200 });
        },
        () => {
            const res = http.get(`${BASE_URL}/api/v1/reports/summary`, { headers });
            check(res, { 'reports summary 200': (r) => r.status === 200 });
        },
        () => {
            const res = http.get(`${BASE_URL}/api/v1/animals?limit=50`, { headers });
            check(res, { 'animals list 50 200': (r) => r.status === 200 });
        },
        () => {
            const res = http.get(`${BASE_URL}/api/v1/bulk-tank`, { headers });
            check(res, { 'bulk tank 200': (r) => r.status === 200 });
        },
    ];

    const idx = Math.floor(Math.random() * scenarios.length);
    scenarios[idx]();

    sleep(1);
}
