# Rust Axum API Playground

This project is a personal, self-directed learning endeavor built with Rust and Axum. It’s designed as a sandbox to experiment with asynchronous programming, robust routing, and error handling—all in a type-safe and performance-oriented environment. Use this project as your playground to understand key web development concepts, build practical RESTful APIs, and progressively sharpen your Rust skills.

---

## Features

- **Asynchronous Programming:** Built using Tokio for handling concurrent operations.
- **Robust Routing:** Leveraging Axum’s routing system to design a RESTful API.
- **Type Safety:** Emphasizes Rust’s type system to catch errors at compile time.
- **Structured Error Handling:** Consistent JSON responses using an `ApiResponse` wrapper.
- **MongoDB Integration:** Uses MongoDB as the data store via the official Rust driver.

---

## Routes and Endpoints

### GET `/`

- **Description:**  
  Returns a simple "Hello World" message, serving as a health check.
- **Method:** GET
- **Response:**  
  - **Status:** 200 OK  
  - **Body:**  

    ```bash
    Hello World
    ```

---

### POST `/identity`

- **Description:**  
  Creates a new identity record. The request should include a JSON payload with `name` and `age`.
- **Method:** POST
- **Request Body Example:**

  ```json
  {
    "name": "Alice",
    "age": 30
  }
  ```

- **Response:**  
  - **Status:** 201 Created  
  - **Body:**  

    ```json
    {
      "message": "Identity created",
      "data": "INSERTED_ID_HERE"
    }
    ```

---

### GET `/identity`

- **Description:**  
  Retrieves a list of all identities stored in the database.
- **Method:** GET
- **Response:**  
  - **Status:** 200 OK  
  - **Body:**  

    ```json
    {
      "message": "Fetched all identities",
      "data": [
        {
          "_id": "60b8d6c5f1a8d23d4c8f4e1a",
          "name": "Alice",
          "age": 30
        },
        {
          "_id": "60b8d6d9f1a8d23d4c8f4e1b",
          "name": "Bob",
          "age": 25
        }
      ]
    }
    ```

---

### GET `/identity/{id}`

- **Description:**  
  Retrieves a specific identity by its unique ObjectId.
- **Method:** GET
- **URL Parameter:**  
  - `id`: The MongoDB ObjectId of the identity.
- **Response:**  
  - **Status:**  
    - **200 OK** if found  
    - **404 Not Found** if the identity does not exist  
  - **Body Example (Found):**

    ```json
    {
      "message": "Fetched",
      "data": {
        "_id": "60b8d6c5f1a8d23d4c8f4e1a",
        "name": "Alice",
        "age": 30
      }
    }
    ```

---

### PATCH `/identity/{id}`

- **Description:**  
  Updates an existing identity partially. Only the fields provided in the JSON payload will be updated. At least one field (`name` or `age`) must be provided.
- **Method:** PATCH
- **URL Parameter:**  
  - `id`: The MongoDB ObjectId of the identity.
- **Request Body Example:**

  ```json
  {
    "name": "Alice Smith"
  }
  ```

- **Response:**  
  - **Status:**  
    - **200 OK** if updated or no changes were necessary  
    - **404 Not Found** if the identity does not exist  
    - **400 Bad Request** if neither `name` nor `age` is provided  
  - **Body:**  

    ```json
    {
      "message": "Updated", // or "No changes made" or "Document not found"
      "data": null
    }
    ```

---

### DELETE `/identity/{id}`

- **Description:**  
  Deletes an identity record based on its ObjectId.
- **Method:** DELETE
- **URL Parameter:**  
  - `id`: The MongoDB ObjectId of the identity.
- **Response:**  
  - **Status:**  
    - **200 OK** if deletion was successful  
    - **404 Not Found** if the identity does not exist  
  - **Body:**  

    ```json
    {
      "message": "Deleted", // or "Document not found"
      "data": null
    }
    ```

---

## Running the Project

### Prerequisites

- **Rust Toolchain:** Install from [rustup.rs](https://rustup.rs/)
- **MongoDB:** Running instance (default URI: `mongodb://localhost:27017/`)
- **Cargo:** Package manager included with Rust

### Installation

1. **Clone the Repository:**

   ```bash
   git clone https://github.com/yourusername/rust-axum-api-playground.git
   cd rust-axum-api-playground
   ```

2. **Build and Run:**

   ```bash
   cargo run
   ```

   The server will start on port `3000`. You can test the endpoints using tools like `curl`, Postman, or your preferred REST client.

---
