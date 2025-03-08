# Vite + React + Rust

This template gives you everything you need to build a full-stack Web3 application on the [Internet Computer](https://internetcomputer.org/).
It includes a frontend built with Vite and React and a backend written in Rust.

## Get started:

Make sure that [Node.js](https://nodejs.org/en/) `>= 21` and [`dfx`](https://internetcomputer.org/docs/current/developer-docs/build/install-upgrade-remove) `>= 0.18` are installed on your system.

Run the following commands in a new, empty project directory:

```sh
git clone [https://github.com/crnlicp/Chronolock-React-Rust.git](https://github.com/crnlicp/Chronolock-React-Rust.git) # Download this starter project
cd Chronolock-React-Rust # Navigate to the project directory
dfx start --clean --background # Run dfx in the background
npm install # Install project dependencies
npm run setup # Install packages, deploy canisters, and generate type bindings

npm start # Start the development server
```

## 🚀 Develop

The frontend will update automatically as you save changes. 
For the backend, run `dfx deploy backend` to redeploy.
To redeploy all canisters (front- and backend), run `dfx deploy`.

When ready, run `dfx deploy --network ic` to deploy your application to the ICP mainnet.

## 🛠️ Technology Stack

- [Vite](https://vitejs.dev/): high-performance tooling for front-end web development
- [React](https://reactjs.org/): a component-based UI library
- [TypeScript](https://www.typescriptlang.org/): JavaScript extended with syntax for types
- [Sass](https://sass-lang.com/): an extended syntax for CSS stylesheets
- [Prettier](https://prettier.io/): code formatting for a wide range of supported languages
- [Rust CDK](https://docs.rs/ic-cdk/): the Canister Development Kit for Rust

## 📚 Documentation

- [Internet Computer docs](https://internetcomputer.org/docs/current/developer-docs/ic-overview)
- [Internet Computer wiki](https://wiki.internetcomputer.org/)
- [Internet Computer forum](https://forum.dfinity.org/)
- [Vite developer docs](https://vitejs.dev/guide/)
- [React quick start guide](https://react.dev/learn)
- [`dfx.json` reference schema](https://internetcomputer.org/docs/current/references/dfx-json-reference/)
- [Rust developer docs](https://internetcomputer.org/docs/current/developer-docs/backend/rust/)
- [Developer Experience Feedback Board](https://dx.internetcomputer.org/)


## 💡 Tips and Tricks

- If the links printed by dfx do not work in Codespaces, run `./canister_urls.py` and click the links shown there.
- If you get an error `The wasm of 7hfb6-caaaa-aaaar-qadga-cai in pulled cache ...` run `rm -rf ~/.cache/dfinity/pulled/7hfb6-caaaa-aaaar-qadga-cai`
- Customize your project's code style by editing the `.prettierrc` file and then running `npm run format`.
- Split your frontend and backend console output by running `npm run frontend` and `npm run backend` in separate terminals.
