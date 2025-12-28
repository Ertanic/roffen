POST {{base}}/admin/login
HTTP 400

POST {{base}}/admin/login
Content-Type: application/json
{
    "login": "admin",
    "password": "admin"
}
HTTP 200
[Asserts]
header "Set-Cookie" startsWith "auth="