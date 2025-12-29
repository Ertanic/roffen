POST {{base}}/admin/login
[Form]
login: admin
password: admin
HTTP 200
[Captures]
jwt: cookie "{{AUTH_COOKIE_NAME}}"

GET {{base}}/admin/logout
[Cookies]
auth: {{jwt}}
HTTP 200
[Asserts]
cookie "{{AUTH_COOKIE_NAME}}" == "_"
cookie "{{AUTH_COOKIE_NAME}}[Max-Age]" == 0

GET {{base}}/admin/logout?next=/admin/login
[Cookies]
auth: {{jwt}}
HTTP 303
Location: /admin/login
[Asserts]
cookie "{{AUTH_COOKIE_NAME}}" == "_"
cookie "{{AUTH_COOKIE_NAME}}[Max-Age]" == 0