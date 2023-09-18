require("@nomicfoundation/hardhat-toolbox");
require('dotenv').config();

const { ETHERSCAN_API_KEY, GOERLI_PRIVATE_KEY, ETHEREUM_PRIVATE_KEY } = process.env;

/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  solidity: {
    version: "0.8.19",
    settings: {
      optimizer: {
        enabled: true,
        runs: 5000000
      }
    }
  },
  networks: {
    ethereum: {
      url: "https://ethereum.publicnode.com",
      accounts: [ETHEREUM_PRIVATE_KEY]
    },
    goerli: {
      url: "https://ethereum-goerli.publicnode.com",
      accounts: [GOERLI_PRIVATE_KEY]
    }
  },
  etherscan: {
    apiKey: ETHERSCAN_API_KEY
  }
};
