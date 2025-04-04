# Rust Axum API Playground

This project is a personal, self-directed learning endeavor built with Rust and Axum. It serves as a sandbox to experiment with asynchronous programming, robust routing, structured error handling, and secure authentication—all within a type-safe, high-performance environment. Use this project to explore key web development concepts, build practical RESTful APIs, and sharpen your Rust skills.

---

## Features

- **Asynchronous Programming:** Built using Tokio for concurrent operations.
- **Robust Routing:** Leverages Axum's routing system to design a RESTful API.
- **Type Safety:** Utilizes Rust’s strong type system to catch errors at compile time.
- **Structured Error Handling:** Consistent JSON responses via an `ApiResponse` wrapper.
- **MongoDB Integration:** Uses MongoDB as the datastore via the official Rust driver.
- **Authentication & Authorization:** Secure endpoints with JWT and password hashing (Argon2).
- **Centralized Configuration:** Loads environment variables from a `.env` file using a dedicated configuration module.
- **Middleware Support:** Implements custom middleware for JWT authentication.

---

## Configuration

The application uses a dedicated `config` module to load environment variables via [`dotenvy`](https://crates.io/crates/dotenvy) and [`once_cell`](https://crates.io/crates/once_cell). Global configuration values are defined in `config.rs` and can be imported throughout the app.

### Environment Variables

The following environment variables must be set in your `.env` file:

- `SECRET_KEY` – The secret key used for JWT encoding.
- `HOST` – The host on which the API server will run (e.g., `0.0.0.0`).
- `PORT` – The port on which the API server will listen (e.g., `3000`).
- `MONGO_URI` – The connection URI for your MongoDB instance (e.g., `mongodb://localhost:27017/`).

Example `.env` file:

```env
SECRET_KEY=your_secret_key_here
HOST=0.0.0.0
PORT=3000
MONGO_URI=mongodb://localhost:27017/
```

The configuration is loaded at the beginning of the `main` function by calling `load_dotenv()`, making these values available globally.

---

## Routes and Endpoints

The API is split into **public endpoints** (accessible without authentication) and **protected endpoints** (which require a valid JWT token). For protected endpoints, include the token in the `Authorization` header using the format:

```sh
Authorization: Bearer <JWT_TOKEN>
```

### Public Endpoints

#### GET `/`

- **Description:**  
  Returns a simple "Hello World" message as a health check.
- **Method:** GET
- **Response:**  
  - **Status:** 200 OK  
  - **Body:**

    ```plain
    Hello World
    ```

#### POST `/signup`

- **Description:**  
  Registers a new user by accepting an email and password. The password is securely hashed using Argon2.
- **Method:** POST
- **Request Body Example:**

  ```json
  {
    "email": "user@example.com",
    "password": "your_password"
  }
  ```

- **Response:**  
  - **Status:** 201 Created  
  - **Body:**

    ```json
    {
      "message": "Auth created",
      "data": "INSERTED_ID_HERE"
    }
    ```

#### POST `/login`

- **Description:**  
  Authenticates a user with email and password. On success, returns a JWT token.
- **Method:** POST
- **Request Body Example:**

  ```json
  {
    "email": "user@example.com",
    "password": "your_password"
  }
  ```

- **Response:**  
  - **Status:** 200 OK  
  - **Body:**

    ```json
    {
      "message": "You are logged in",
      "data": "JWT_TOKEN_HERE"
    }
    ```

---

### Protected Endpoints

*These endpoints require a valid JWT token in the `Authorization` header.*

#### GET `/protected`

- **Description:**  
  A sample endpoint that greets the authenticated user.
- **Method:** GET
- **Response:**  
  - **Status:** 200 OK  
  - **Body:**

    ```json
    {
      "message": "Hello. You are logged in using user@example.com",
      "data": {}
    }
    ```

#### Identity CRUD Operations

##### POST `/identity`

- **Description:**  
  Creates a new identity record. The JSON payload must include `name` and `age`.
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

##### GET `/identity`

- **Description:**  
  Retrieves a list of all identities in the database.
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

##### GET `/identity/{id}`

- **Description:**  
  Retrieves a specific identity by its MongoDB ObjectId.
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

##### PATCH `/identity/{id}`

- **Description:**  
  Partially updates an existing identity. At least one field (`name` or `age`) must be provided.
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
    - **200 OK** if updated (or no changes were made)  
    - **404 Not Found** if the identity does not exist  
    - **400 Bad Request** if neither field is provided  
  - **Body:**

    ```json
    {
      "message": "Updated", // or "No changes made" or "Document not found"
      "data": null
    }
    ```

##### DELETE `/identity/{id}`

- **Description:**  
  Deletes an identity record based on its MongoDB ObjectId.
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
- **MongoDB:** Ensure you have a running instance (default URI: `mongodb://localhost:27017/`)
- **Cargo:** Rust’s package manager

### Installation

1. **Clone the Repository:**

   ```bash
   git clone https://github.com/yourusername/rust-axum-api-playground.git
   cd rust-axum-api-playground
   ```

2. **Set Up Environment Variables:**

   Create a `.env` file in the root directory with the following contents:

   ```env
   SECRET_KEY=your_secret_key_here
   HOST=0.0.0.0
   PORT=3000
   MONGO_URI=mongodb://localhost:27017/
   ```

3. **Build and Run:**

   ```bash
   cargo run
   ```

   The server will start on the host and port specified in your `.env` file (e.g., `0.0.0.0:3000`). Test the endpoints using tools like `curl`, Postman, or your preferred REST client.

---

## Project Structure

- **Main File:** Contains the Axum server setup, router composition, and main function.
- **Configuration:**  
  - `config.rs` loads the `.env` file and exposes global configuration values (`SECRET_KEY`, `HOST`, `PORT`, `MONGO_URI`) via lazy statics.
- **Route Handlers:** Functions for Identity CRUD operations and authentication (signup/login).
- **Middleware:** Custom `login_required` middleware to enforce JWT authentication on protected endpoints.
- **Data Models:** Structs (`Identity`, `Auth`, etc.) using Serde for serialization/deserialization.
- **Database Integration:** Uses the official MongoDB Rust driver for database operations.

---
