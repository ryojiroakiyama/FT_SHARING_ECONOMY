# First steps
- create [testnet account](https://wallet.testnet.near.org/)
- near-cli
```
npm install -g near-cli

// confirm
near

// create a full-access key on hard dirve (~/.near-credetials)
// generate a key pair:
//  private key -> tucked away in a JSON file
//  public key -> send as a URL parameter to NEAR Wallet, add a full access key to the account
near lgin
```
- Rust
```
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh

// Smart contracts compile to WebAssembly (Wasm) so we'll add the toolchain for Rust
rustup target add wasm32-unknown-unknown
```
- make project
```
// build command ref
RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release

// make project
npx create-near-app myproject --frontend react --contract rust
```

# near-cli
```
export ID=account.testnet

// create sub account
near create-account sub.$ID --masterAccount $ID --initialBalance 50

// compile
cargo build --all --target wasm32-unknown-unknown --release

// check account state
// Note the code_hash here is all ones. This indicates that there is no contract deployed to this account.
near state

// deploy
near deploy sub.$ID --wasmFile [wasm file path]

// call view func
near view sub.$ID [view func name]

// call mut func
// because of mut func, have to signin, so specify --accountID to use for signin
// cli use the credentials files
near call sub.$ID [func name] '{"arg name": "arg value"}' --accountId $ID

// call mut func on windows
near call sub.$ID [func name] "{\"arg name\": \"arg value\"}" --accountId $ID

// delete account
near delete sub.$ID $ID

// curl to RPC endpoint
curl -d '{"jsonrpc": "2.0", "method": "query", "id": "see-state", "params": {"request_type": "view_state", "finality": "final", "account_id": "sub.$ID", "prefix_base64": ""}}' -H 'Content-Type: application/json' https://rpc.testnet.near.org

// Batch Action
// deploy and init
near deploy sub.$ID --wasmFile [wasm file path] --initFunction 'new(func name)'  --initArgs '{"arg_name": "arg_value"}'
```

# Debug log
## check list
- import
- cargo.toml

near-blank-project
==================

This [React] app was initialized with [create-near-app]


Quick Start
===========

To run this project locally:

1. Prerequisites: Make sure you've installed [Node.js] ??? 12
2. Install dependencies: `npm install`
3. Run the local development server: `npm run dev` (see `package.json` for a
   full list of `scripts` you can run with `npm`)

Now you'll have a local development environment backed by the NEAR TestNet!

Go ahead and play with the app and the code. As you make code changes, the app will automatically reload.


Exploring The Code
==================

1. The "backend" code lives in the `/contract` folder. See the README there for
   more info.
2. The frontend code lives in the `/frontend` folder. `/frontend/index.html` is a great
   place to start exploring. Note that it loads in `/frontend/assets/js/index.js`, where you
   can learn how the frontend connects to the NEAR blockchain.
3. Tests: there are different kinds of tests for the frontend and the smart
   contract. See `contract/README` for info about how it's tested. The frontend
   code gets tested with [jest]. You can run both of these at once with `npm
   run test`.


Deploy
======

Every smart contract in NEAR has its [own associated account][NEAR accounts]. When you run `npm run dev`, your smart contract gets deployed to the live NEAR TestNet with a throwaway account. When you're ready to make it permanent, here's how.


Step 0: Install near-cli (optional)
-------------------------------------

[near-cli] is a command line interface (CLI) for interacting with the NEAR blockchain. It was installed to the local `node_modules` folder when you ran `npm install`, but for best ergonomics you may want to install it globally:

    npm install --global near-cli

Or, if you'd rather use the locally-installed version, you can prefix all `near` commands with `npx`

Ensure that it's installed with `near --version` (or `npx near --version`)


Step 1: Create an account for the contract
------------------------------------------

Each account on NEAR can have at most one contract deployed to it. If you've already created an account such as `your-name.testnet`, you can deploy your contract to `near-blank-project.your-name.testnet`. Assuming you've already created an account on [NEAR Wallet], here's how to create `near-blank-project.your-name.testnet`:

1. Authorize NEAR CLI, following the commands it gives you:

      near login

2. Create a subaccount (replace `YOUR-NAME` below with your actual account name):

      near create-account near-blank-project.YOUR-NAME.testnet --masterAccount YOUR-NAME.testnet


Step 2: set contract name in code
---------------------------------

Modify the line in `src/config.js` that sets the account name of the contract. Set it to the account id you used above.

    const CONTRACT_NAME = process.env.CONTRACT_NAME || 'near-blank-project.YOUR-NAME.testnet'


Step 3: deploy!
---------------

One command:

    npm run deploy

As you can see in `package.json`, this does two things:

1. builds & deploys smart contract to NEAR TestNet
2. builds & deploys frontend code to GitHub using [gh-pages]. This will only work if the project already has a repository set up on GitHub. Feel free to modify the `deploy` script in `package.json` to deploy elsewhere.


Troubleshooting
===============

On Windows, if you're seeing an error containing `EPERM` it may be related to spaces in your path. Please see [this issue](https://github.com/zkat/npx/issues/209) for more details.


  [React]: https://reactjs.org/
  [create-near-app]: https://github.com/near/create-near-app
  [Node.js]: https://nodejs.org/en/download/package-manager/
  [jest]: https://jestjs.io/
  [NEAR accounts]: https://docs.near.org/docs/concepts/account
  [NEAR Wallet]: https://wallet.testnet.near.org/
  [near-cli]: https://github.com/near/near-cli
  [gh-pages]: https://github.com/tschaub/gh-pages
