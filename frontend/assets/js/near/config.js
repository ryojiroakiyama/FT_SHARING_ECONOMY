const BIKE_CONTRACT_NAME =
  process.env.CONTRACT_NAME || "sub.bike_share.testnet";
// my_ftに関しては簡単のために固定で用意
const FT_CONTRACT_NAME = "my_ft.testnet";

// 変更点メモ
// 1. bikeContractNameをftContractNameとbikeContractNameの二つに変更

function getConfig(env) {
  switch (env) {
    case "production":
    case "mainnet":
      return {
        networkId: "mainnet",
        nodeUrl: "https://rpc.mainnet.near.org",
        bikeContractName: BIKE_CONTRACT_NAME,
        ftContractName: FT_CONTRACT_NAME,
        walletUrl: "https://wallet.near.org",
        helperUrl: "https://helper.mainnet.near.org",
        explorerUrl: "https://explorer.mainnet.near.org",
      };
    case "development":
    case "testnet":
      return {
        networkId: "testnet",
        nodeUrl: "https://rpc.testnet.near.org",
        bikeContractName: BIKE_CONTRACT_NAME,
        ftContractName: FT_CONTRACT_NAME,
        walletUrl: "https://wallet.testnet.near.org",
        helperUrl: "https://helper.testnet.near.org",
        explorerUrl: "https://explorer.testnet.near.org",
      };
    case "betanet":
      return {
        networkId: "betanet",
        nodeUrl: "https://rpc.betanet.near.org",
        bikeContractName: BIKE_CONTRACT_NAME,
        ftContractName: FT_CONTRACT_NAME,
        walletUrl: "https://wallet.betanet.near.org",
        helperUrl: "https://helper.betanet.near.org",
        explorerUrl: "https://explorer.betanet.near.org",
      };
    case "local":
      return {
        networkId: "local",
        nodeUrl: "http://localhost:3030",
        keyPath: `${process.env.HOME}/.near/validator_key.json`,
        walletUrl: "http://localhost:4000/wallet",
        bikeContractName: BIKE_CONTRACT_NAME,
        ftContractName: FT_CONTRACT_NAME,
      };
    case "test":
    case "ci":
      return {
        networkId: "shared-test",
        nodeUrl: "https://rpc.ci-testnet.near.org",
        bikeContractName: BIKE_CONTRACT_NAME,
        ftContractName: FT_CONTRACT_NAME,
        masterAccount: "test.near",
      };
    case "ci-betanet":
      return {
        networkId: "shared-test-staging",
        nodeUrl: "https://rpc.ci-betanet.near.org",
        bikeContractName: BIKE_CONTRACT_NAME,
        ftContractName: FT_CONTRACT_NAME,
        masterAccount: "test.near",
      };
    default:
      throw Error(
        `Unconfigured environment '${env}'. Can be configured in src/config.js.`
      );
  }
}

module.exports = getConfig;
