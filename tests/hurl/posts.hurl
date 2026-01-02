POST {{base}}/api/posts
HTTP 401

DELETE {{base}}/api/posts
HTTP 401

PATCH {{base}}/api/posts
HTTP 401

POST {{base}}/admin/login
[Form]
login: admin
password: admin
HTTP 200
[Captures]
jwt: cookie "{{AUTH_COOKIE_NAME}}"

POST {{base}}/api/posts
```json
{
    "title": "Test post",
    "status": "draft",
    "content": [
        {
            "name": "Text"
        }
    ]
}
```
HTTP 201
[Captures]
post-id-1: jsonpath "$['post_id']"

POST {{base}}/api/posts
```json
{
    "title": "Test post",
    "status": "draft",
    "content": [
        {
            "name": "Text"
        }
    ]
}
```
HTTP 201
[Captures]
post-id-2: jsonpath "$['post_id']"

GET {{base}}/api/posts?id={{post-id-1}}
HTTP 200
[Asserts]
jsonpath "$.author" == "admin"
jsonpath "$.title" == "Test post"
jsonpath "$.draft" == true

PATCH {{base}}/api/posts
```json
{
    "post_id": "{{post-id-1}}",
    "new_status": "public",
    "new_title": "Updated title"
}
```
HTTP 204

GET {{base}}/api/posts?id={{post-id-1}}
HTTP 200
[Asserts]
jsonpath "$.title" == "Updated title"
jsonpath "$.draft" == false

DELETE {{base}}/api/posts
HTTP 400

DELETE {{base}}/api/posts?id={{post-id-1}}
HTTP 204

DELETE {{base}}/api/posts?id={{post-id-2}}
HTTP 204