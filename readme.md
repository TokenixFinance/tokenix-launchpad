# Tokenix Smart Contract

This repository contains the smart contract for Tokenix, a launchpad project currently under development at https://tokenix.finance.

## Overview

Tokenix is a decentralized launchpad platform built on the Solana blockchain. It aims to provide a seamless and efficient way for projects to launch their tokens and for users to participate in token sales.

## Features (In Development)

- Token Creation: Easily create new tokens with customizable parameters.
- Pool Creation: Set up bonding curve pools for token distribution.
- Token Purchase: Allow users to buy tokens directly from the pool.
- Dynamic Pricing: Implement a bonding curve mechanism for price discovery.

Please note that this project is currently in the development phase and features are subject to change.

## Installation

To set up the project locally, follow these steps:

1. Clone the repository:
   ```
   git clone https://github.com/tokenix/tokenix-launchpad.git
   cd tokenix
   ```

2. Install dependencies:
   ```
   yarn install
   ```

3. Build the project:
   ```
   anchor build
   ```

## Running Tests

To run the tests, use the following command:

```
anchor test --skip-local-validator
```

## Test Results

Here are the latest test results:

```
=== Token Creation ===
Token creation TX: 4znrmXdBLokfd3kAAHzs8R55eMXFJCe31eLoFtPUrncHU1Uo1UQUSXRyPRRqm4dsJ1YxGugDroKSFSfZuDvEGBZJ
Token address (mint): GzsMKTqH3PuZGXL1B4A7CVeLsbcV8tsmQdNPiiuo96hN
Initial supply: 100000000000000000
Minted to (token account): C7XgYTq8YqAnijcSXQwGaqSRDQj5dfNKXp6VwpkqieRP
    ✔ Creates a token (457ms)

=== Pool Creation ===
Pool creation TX: 3FEgKD36F8RV57EJz6am1bDrgKHbhiTdGchwjHzxnLZBfaBp8Rckccx625V8cqA3VtZYHcyXUYknHSEwYqDaigh5
Pool address (bonding curve): 2h9W9v9rG82V7Et5LK59F9JoTcFVemsoAVNZ9cQHv3bj
Pool token account: Fd9NLRxz4zKK2N65ssJMYyzeoB6gsE3n2RgxXaRppTm5
Pool initial price: 10000
Pool total supply: 100000000000000000
    ✔ Creates a pool (405ms)

=== Token Purchase ===
Buyer address: GEDbafFFyNsc1WnUVUXfUTx91bzb32CuTVji1P6sPDM1
Buyer initial balance: 10000000000000
Amount to buy: 1
Pool info before purchase:
Pool info:
  Total supply: 100000000000000000
  Current price: 10000
Buy transaction signature: 2TBr3BUknNFz89dPovwEgKs4JpoxRbCjAey6HZiKzJVKwWN9SUsphrCqemFxtn4ojYFLsWFX8NVB8tV2cDmSY8xp
Buyer final balance: 8999997950720
Buyer token balance: 1e-9
Pool info after purchase:
Pool info:
  Total supply: 99999999999999999
  Current price: 1000000009999
    ✔ Buys tokens (809ms)

3 passing (2s)
```

## Disclaimer

This project is in active development. The code and features are subject to change. Do not use in production environments.

## Contributing

We welcome contributions to the Tokenix project. Please feel free to submit issues and pull requests.

## License

[MIT License](LICENSE)

## Contact

For any inquiries or support, please contact us at:

- Website: [tokenix.finance](https://tokenix.finance)
- Email: dev@tokenix.finance
- Twitter: [@TokenixFinance](https://twitter.com/TokenixFinance)


## Security

We take the security of our smart contracts seriously. If you discover any security issues, please report them to dev@tokenix.finance.

## Acknowledgements

We would like to thank the Solana and Anchor teams for their excellent documentation and tools, which have been instrumental in the development of this project.

## Development Setup

For developers looking to contribute or modify the smart contract:

1. Install Rust and Cargo:
   ```
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Install Solana CLI tools:
   ```
   sh -c "$(curl -sSfL https://release.solana.com/v1.14.11/install)"
   ```

3. Install Anchor:
   ```
   cargo install --git https://github.com/project-serum/anchor avm --locked --force
   avm install latest
   avm use latest
   ```

4. Set up a local Solana validator:
   ```
   solana-test-validator
   ```

## Smart Contract Structure

The main components of the Tokenix smart contract are:

- `lib.rs`: Contains the core logic for token creation, pool management, and token purchases.
- `state.rs`: Defines the state structures used in the contract.
- `errors.rs`: Custom error types for better error handling.

## Testing

We use Anchor's testing framework for our unit and integration tests. To add new tests:

1. Navigate to the `tests` directory.
2. Add your test cases to the existing test files or create new ones.
3. Run the tests using the command mentioned in the "Running Tests" section.

## Deployment

To deploy the smart contract to the Solana devnet:

1. Build the project:
   ```
   anchor build
   ```

2. Get the program ID:
   ```
   solana address -k target/deploy/tokenix-keypair.json
   ```

3. Update the program ID in `Anchor.toml` and `lib.rs`.

4. Deploy the program:
   ```
   anchor deploy --provider.cluster devnet
   ```

Remember to thoroughly test on devnet before considering mainnet deployment.
