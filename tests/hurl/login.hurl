POST {{base}}/admin/login
HTTP 400

POST {{base}}/admin/login
[Form]
login: admin
password: admin
HTTP 200
[Asserts]
cookie "{{AUTH_COOKIE_NAME}}" exists

POST {{base}}/admin/login
[Query]
next: /admin
[Form]
login: admin
password: admin
HTTP 303
Location: /admin
[Asserts]
cookie "{{AUTH_COOKIE_NAME}}" exists