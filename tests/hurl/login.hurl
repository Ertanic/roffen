POST {{base}}/admin/login
HTTP 400

POST {{base}}/admin/login
[Form]
login: admin
password: admin
HTTP 200
[Asserts]
header "Set-Cookie" startsWith "auth="

POST {{base}}/admin/login
[Query]
next: /admin
[Form]
login: admin
password: admin
HTTP 303
Location: /admin
[Asserts]
header "Set-Cookie" startsWith "auth="