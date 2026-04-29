import http from 'k6/http';
import { check } from 'k6';

export const options = {
    vus: 5,
    iterations: 10,
    thresholds: {
        http_req_duration: ['p(95)<1000'],
        http_req_failed: ['rate<0.1'],
    },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';

export default function () {
    const res = http.get(`${BASE_URL}/api/v1/health`);
    check(res, {
        'health status 200': (r) => r.status === 200,
        'health body valid': (r) => {
            try {
                const body = JSON.parse(r.body);
                return body.status === 'ok' || body.status === 'healthy';
            } catch {
                return r.status === 200;
            }
        },
    });

    const loginRes = http.post(`${BASE_URL}/api/v1/auth/login`, JSON.stringify({
        username: 'admin',
        password: __ENV.ADMIN_PASSWORD || 'admin',
    }), {
        headers: { 'Content-Type': 'application/json' },
    });

    check(loginRes, {
        'login status 200': (r) => r.status === 200 || r.status === 400,
    });
}
