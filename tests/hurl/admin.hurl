GET {{base}}/admin
HTTP 303
Location: /admin/login?next=/admin

POST {{base}}/admin/login
[Form]
login: admin
password: admin
HTTP 200
[Captures]
jwt: cookie "{{AUTH_COOKIE_NAME}}"

GET {{base}}/admin
[Cookies]
auth: {{jwt}}
HTTP 200