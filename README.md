# Rust API Template

A generic Rust API template with a database connection.

---

🔑 Authentication

All endpoints require a valid API key via the Authorization header.

| Key                  | Location | Type    | Required | Description                         |
|----------------------|----------|---------|----------|-------------------------------------|
| Authorization        | Header   | string  | Yes      | `Bearer <API_KEY>`                  |
| Content-Type         | Header   | string  | Yes      | `application/json`                  |

---
🌐 Base URL

http://localhost:8081/


---

📦 Data Models

Item
```json
{
  "id": "string",
  "name": "string",
  "created_at": "ISO8601 timestamp",
  "updated_at": "ISO8601 timestamp"
}
```
CreateItem
```json
{
  "name": "string"
}
```
UpdateItem

All fields optional; provide only the properties you want to modify.
```json
{
  "name": "string?"
}
```

---

📋 Endpoints

1. Create Item
	•	Method: POST
	•	Path: /item/create
	•	Permission Level: Ring1

2. Update Item
	•	Method: POST
	•	Path: /item/update/{id}
	•	Permission Level: Ring1

3. Delete Item
	•	Method: DELETE
	•	Path: /item/{id}
	•	Permission Level: Ring1

4. List All Items
	•	Method: GET
	•	Path: /item
	•	Permission Level: Ring2

5. Get Item by ID
	•	Method: GET
	•	Path: /item/{id}
	•	Permission Level: Ring2

---

💬 Standard Response Format

| Field    | Type        | Description                                 |
|----------|-------------|---------------------------------------------|
| success  | boolean     | Indicates if the request succeeded          |
| data     | object/null | The response payload (or null)              |
| error    | string/null | Error code if `success` is false            |
| message  | string      | Human-readable status message               |


---