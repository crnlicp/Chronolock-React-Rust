# Chronolock: Vite + React + Rust

Chronolock is a full-stack Web3 application built on the [Internet Computer](https://internetcomputer.org/). It features a frontend powered by Vite and React and a backend written in Rust.

## 🚀 Get Started

### Prerequisites

Ensure the following tools are installed on your system:

- [Node.js](https://nodejs.org/en/) `>= 21`
- [dfx](https://internetcomputer.org/docs/current/developer-docs/build/install-upgrade-remove) `>= 0.18`
- [Rust](https://www.rust-lang.org/tools/install)

### Setup Instructions

Run the following command to set up the project:

```sh
npm run setup
```

This command will automatically install dependencies, build and deploy all the the backend canisters, and prepare the project for development.

### Development Workflow

- **Start the development server**:
  ```sh
  npm start
  ```

- **Deploy to the Internet Computer mainnet**:
  ```sh
  dfx deploy --network ic
  ```

## 🛠️ Technology Stack

- [Vite](https://vitejs.dev/): High-performance tooling for front-end web development
- [React](https://reactjs.org/): A component-based UI library
- [TypeScript](https://www.typescriptlang.org/): JavaScript extended with syntax for types
- [Rust CDK](https://docs.rs/ic-cdk/): The Canister Development Kit for Rust

## 📚 Documentation

- [Internet Computer docs](https://internetcomputer.org/docs/current/developer-docs/ic-overview)
- [Vite developer docs](https://vitejs.dev/guide/)
- [React quick start guide](https://react.dev/learn)
- [Rust developer docs](https://internetcomputer.org/docs/current/developer-docs/backend/rust/)

## 🧪 Testing

- **Run frontend and backend tests**:
  ```sh
  npm run test
  ```

- **Run frontend only tests**:
  ```sh
  npm run test:frontend
  ```

- **Run backend only tests**:
  ```sh
  npm run test:backend
  ```
  ```sh
  cargo test --all
  ```

- **Run specific canister's tests**:
    ```sh
  cargo test --package [package_name]
  ```

## 🛡️ License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
