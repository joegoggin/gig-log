# Payment Route Curl Tests

Base URL: `http://localhost:8000`

## Setup

### Login

Note: The committed command/request body redact the password value for safety.

Curl command:

```bash
curl -s -X POST "http://localhost:8000/auth/log-in" -H "Content-Type: application/json" -c "/tmp/giglog-payment-cookies.txt" -o "/tmp/payment-login-response.json" -w "%{http_code}" -d '{"email":"joegoggindev@gmail.com","password":"<REDACTED>","remember_me":false}'
```

Request body:

```json
{
  "email": "joegoggindev@gmail.com",
  "password": "<REDACTED>",
  "remember_me": false
}
```

Response body (HTTP 200):

```json
{
  "message": "Logged in successfully.",
  "user_id": "9fedfba9-f4a1-4f85-ab1a-8e9ab629fe45"
}
```

### Fetch company ID used by create/update tests

Curl command:

```bash
curl -s -X GET "http://localhost:8000/companies" -b "/tmp/giglog-payment-cookies.txt" -o "/tmp/payment-companies-response.json" -w "%{http_code}"
```

Request body:

`None (GET request)`

Response body (HTTP 200):

```json
{
  "companies": [
    {
      "id": "f1798457-8760-4916-9af0-e1b37c3ef2f0",
      "user_id": "9fedfba9-f4a1-4f85-ab1a-8e9ab629fe45",
      "name": "DataAnnotation",
      "requires_tax_withholdings": false,
      "tax_withholding_rate": null,
      "created_at": "2026-02-16T01:41:06.140200Z",
      "updated_at": "2026-02-16T01:41:06.140200Z"
    }
  ]
}
```

## Route Tests

### 1) List payments (`GET /payments`)

Curl command:

```bash
curl -s -X GET "http://localhost:8000/payments" -b "/tmp/giglog-payment-cookies.txt" -o "/tmp/payment-list-before-response.json" -w "%{http_code}"
```

Request body:

`None (GET request)`

Response body (HTTP 200):

```json
{
  "payments": [
    {
      "id": "ec7d3471-3355-407f-858d-217267c2efe9",
      "user_id": "9fedfba9-f4a1-4f85-ab1a-8e9ab629fe45",
      "company_id": "f1798457-8760-4916-9af0-e1b37c3ef2f0",
      "total": "300.00",
      "payout_type": "venmo",
      "expected_payout_date": null,
      "expected_transfer_date": null,
      "transfer_initiated": false,
      "payment_received": false,
      "transfer_received": false,
      "tax_withholdings_covered": false,
      "created_at": "2026-02-16T01:48:59.302071Z",
      "updated_at": "2026-02-16T01:48:59.302071Z"
    }
  ]
}
```

### 2) Create payment (`POST /payments`)

Curl command:

```bash
curl -s -X POST "http://localhost:8000/payments" -H "Content-Type: application/json" -b "/tmp/giglog-payment-cookies.txt" -o "/tmp/payment-create-response.json" -w "%{http_code}" -d '{"company_id":"f1798457-8760-4916-9af0-e1b37c3ef2f0","total":"123.45","payout_type":"paypal","expected_payout_date":"2026-02-20","expected_transfer_date":"2026-02-22","transfer_initiated":true,"payment_received":true,"transfer_received":false,"tax_withholdings_covered":true}'
```

Request body:

```json
{
  "company_id": "f1798457-8760-4916-9af0-e1b37c3ef2f0",
  "total": "123.45",
  "payout_type": "paypal",
  "expected_payout_date": "2026-02-20",
  "expected_transfer_date": "2026-02-22",
  "transfer_initiated": true,
  "payment_received": true,
  "transfer_received": false,
  "tax_withholdings_covered": true
}
```

Response body (HTTP 201):

```json
{
  "payment": {
    "id": "254ae593-c7f7-46bb-b791-db9cf02010dc",
    "user_id": "9fedfba9-f4a1-4f85-ab1a-8e9ab629fe45",
    "company_id": "f1798457-8760-4916-9af0-e1b37c3ef2f0",
    "total": "123.45",
    "payout_type": "paypal",
    "expected_payout_date": "2026-02-20",
    "expected_transfer_date": "2026-02-22",
    "transfer_initiated": true,
    "payment_received": true,
    "transfer_received": false,
    "tax_withholdings_covered": true,
    "created_at": "2026-02-19T06:07:34.939060Z",
    "updated_at": "2026-02-19T06:07:34.939060Z"
  }
}
```

### 3) Get payment by id (`GET /payments/{payment_id}`)

Curl command:

```bash
curl -s -X GET "http://localhost:8000/payments/254ae593-c7f7-46bb-b791-db9cf02010dc" -b "/tmp/giglog-payment-cookies.txt" -o "/tmp/payment-get-by-id-response.json" -w "%{http_code}"
```

Request body:

`None (GET request)`

Response body (HTTP 200):

```json
{
  "payment": {
    "id": "254ae593-c7f7-46bb-b791-db9cf02010dc",
    "user_id": "9fedfba9-f4a1-4f85-ab1a-8e9ab629fe45",
    "company_id": "f1798457-8760-4916-9af0-e1b37c3ef2f0",
    "total": "123.45",
    "payout_type": "paypal",
    "expected_payout_date": "2026-02-20",
    "expected_transfer_date": "2026-02-22",
    "transfer_initiated": true,
    "payment_received": true,
    "transfer_received": false,
    "tax_withholdings_covered": true,
    "created_at": "2026-02-19T06:07:34.939060Z",
    "updated_at": "2026-02-19T06:07:34.939060Z"
  }
}
```

### 4) Update payment (`PUT /payments/{payment_id}`)

Curl command:

```bash
curl -s -X PUT "http://localhost:8000/payments/254ae593-c7f7-46bb-b791-db9cf02010dc" -H "Content-Type: application/json" -b "/tmp/giglog-payment-cookies.txt" -o "/tmp/payment-update-response.json" -w "%{http_code}" -d '{"company_id":"f1798457-8760-4916-9af0-e1b37c3ef2f0","total":"150.00","payout_type":"cash","expected_payout_date":"2026-02-23","expected_transfer_date":null,"transfer_initiated":false,"payment_received":true,"transfer_received":false,"tax_withholdings_covered":false}'
```

Request body:

```json
{
  "company_id": "f1798457-8760-4916-9af0-e1b37c3ef2f0",
  "total": "150.00",
  "payout_type": "cash",
  "expected_payout_date": "2026-02-23",
  "expected_transfer_date": null,
  "transfer_initiated": false,
  "payment_received": true,
  "transfer_received": false,
  "tax_withholdings_covered": false
}
```

Response body (HTTP 200):

```json
{
  "payment": {
    "id": "254ae593-c7f7-46bb-b791-db9cf02010dc",
    "user_id": "9fedfba9-f4a1-4f85-ab1a-8e9ab629fe45",
    "company_id": "f1798457-8760-4916-9af0-e1b37c3ef2f0",
    "total": "150.00",
    "payout_type": "cash",
    "expected_payout_date": "2026-02-23",
    "expected_transfer_date": null,
    "transfer_initiated": false,
    "payment_received": true,
    "transfer_received": false,
    "tax_withholdings_covered": false,
    "created_at": "2026-02-19T06:07:34.939060Z",
    "updated_at": "2026-02-19T06:07:58.056047Z"
  }
}
```

### 5) Delete payment (`DELETE /payments/{payment_id}`)

Curl command:

```bash
curl -s -X DELETE "http://localhost:8000/payments/254ae593-c7f7-46bb-b791-db9cf02010dc" -b "/tmp/giglog-payment-cookies.txt" -o "/tmp/payment-delete-response.json" -w "%{http_code}"
```

Request body:

`None (DELETE request)`

Response body (HTTP 200):

```json
{
  "message": "Payment deleted successfully."
}
```
