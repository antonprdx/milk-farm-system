import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
    stages: [
        { duration: '30s',  target: 20 },
        { duration: '1m',   target: 50 },
        { duration: '30s',  target: 100 },
        { duration: '1m',   target: 100 },
        { duration: '30s',  target: 200 },
        { duration: '1m',   target: 200 },
        { duration: '30s',  target: 50 },
        { duration: '10s',  target: 0 },
    ],
    thresholds: {
        http_req_duration: ['p(95)<1000', 'p(99)<3000'],
        http_req_failed: ['rate<0.1'],
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

    const endpoints = [
        { name: 'animals_list',     url: '/api/v1/animals?limit=50',        method: 'GET' },
        { name: 'analytics_kpi',    url: '/api/v1/analytics/kpi',           method: 'GET' },
        { name: 'analytics_trends', url: '/api/v1/analytics/trends?metric=milk&period=30d', method: 'GET' },
        { name: 'reports_summary',  url: '/api/v1/reports/summary',         method: 'GET' },
        { name: 'milk_productions', url: '/api/v1/milk/day-productions?limit=50', method: 'GET' },
        { name: 'feed_amounts',     url: '/api/v1/feed/day-amounts?limit=50', method: 'GET' },
        { name: 'reproduction',     url: '/api/v1/reproduction/calvings?limit=50', method: 'GET' },
        { name: 'bulk_tank',        url: '/api/v1/bulk-tank',               method: 'GET' },
        { name: 'settings',         url: '/api/v1/settings/users',          method: 'GET' },
        { name: 'contacts',         url: '/api/v1/contacts',                method: 'GET' },
    ];

    const idx = Math.floor(Math.random() * endpoints.length);
    const ep = endpoints[idx];

    const res = http.request(ep.method, `${BASE_URL}${ep.url}`, null, { headers });
    check(res, {
        [`${ep.name} status ok`]: (r) => r.status === 200 || r.status === 404,
    });

    sleep(Math.random() * 0.5);
}

export function teardown(data) {
    http.post(`${BASE_URL}/api/v1/auth/logout`, null, {
        headers: {
            'Authorization': `Bearer ${data.token}`,
        },
    });
}
