![panorama logo](https://i.imagesup.co/images2/302ff85b1ff055738b8c63ae2ca0137f9c3a6929.png)

# Panorama Swap Smart Contract Repository

Welcome to the Panorama Swap smart contract repository! Here, you'll find all the smart contracts that we are using in the Panorama Swap platform. We write our smart contracts using the ink! domain-specific language, which is designed for networks that run on the Substrate framework. The ink! contracts are compiled to WebAssembly, providing efficient and secure execution on the blockchain. We are proud to implement OpenBrush's PSP22 protocol and standard in our contracts.

** Note: We are continually updating and adding more smart contracts and in-depth information, so stay tuned for updates!**

## Contracts :
This repository contains the following smart contracts:

### contract_creator
contract_creator is a contract used to deploy and create trading_pair_azero, trading_pair_psp22 and multi_sig contracts. These contracts enable users to create their own liquidity pools and trading pairs on the Panorama Swap platform and multi sig wallets.

### psp22
psp22 is a contract that implements OpenBrush's PSP22 standard, allowing the creation of PSP22 tokens with metadata extensions such as token name and symbol. This contract provides the functionality to manage PSP22 tokens on the Panorama Swap platform.

### trading_pair_azero
trading_pair_azero is a contract used in deploying AZERO/PSP22 trading pairs and pools on the Panorama Swap platform. This contract provides the necessary functionality to enable trading and liquidity provision for the AZERO/PSP22 pair.

### trading_pair_psp22
trading_pair_psp22 is a contract used in deploying PSP22/PSP22 trading pairs and pools on the Panorama Swap platform. This contract provides the necessary functionality to enable trading and liquidity provision for the PSP22/PSP22 pair.

### vesting_contract
vesting_contract is a smart contract that contains all the logic for the vesting program on the Panorama Swap platform. This contract is used to manage the vesting of tokens for different stakeholders according to predefined rules and conditions.

### airdrop_contract
airdrop_contract is a smart contract that contains all the logic for the airdrop event on the Panorama Swap platform. This contract is used to distribute tokens to eligible participants in an airdrop campaign based on specific criteria and rules.

### multi_sig
multi_sig is a smart contract that contains all the logic for a multiple signatures wallet on the Panorama Swap platform. This contract provides a secure and decentralized way to manage funds by requiring multiple signatures for certain operations, ensuring increased security and accountability.

## How to build and deploy the contracts

To build, compile and deploy your smart contract on Aleph Zero, you will need to install the development tools. the following link is to a great guide from Aleph Zero team: https://docs.alephzero.org/aleph-zero/build/installing-required-tools

**Enter to any smart contract that you wish to compile and build:**

```
cargo +nightly contract build --release
```

**After you successfully compile your contracts, you'll see** ```contract_name.contract``` **, which you'll need in order to deploy the smart contract to the Aleph Zero network, here is a great guide by the Aleph Zero team on how to deploy your contracts: ** [https://docs.alephzero.org/aleph-zero/build/aleph-zero-smart-contracts-basics/deploying-your-contract-to-aleph-zero-testnet](https://docs.alephzero.org/aleph-zero/build/aleph-zero-smart-contracts-basics/deploying-your-contract-to-aleph-zero-testnet)

## Useful Links

- Panorama Swap DAPP: [https://panoramaswap.app/](https://panoramaswap.app/)
- Panorama Swap homepage: [https://panoramaswap.com/](https://panoramaswap.com/)
- Panorama Swap gitbook: [https://panoramaswap-1.gitbook.io/panorama-swaps-documentation/](https://panoramaswap-1.gitbook.io/panorama-swaps-documentation/)
- Aleph Zero homepage: [https://alephzero.org/](https://alephzero.org/)
- ink! GitHub repository: [https://github.com/paritytech/ink](https://github.com/paritytech/ink)
- OpenBrush's GitHub repository: [https://github.com/Supercolony-net/openbrush-contracts](https://github.com/Supercolony-net/openbrush-contracts)
- Substrate GitHub repository: [https://github.com/paritytech/substrate](https://github.com/paritytech/substrate)

