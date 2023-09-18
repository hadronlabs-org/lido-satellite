const hre = require("hardhat");
require('dotenv').config();

async function main() {
    const {
        AXELAR_GATEWAY,
        AXELAR_GAS_SERVICE,
        WST_ETH
    } = process.env;

    const [deployer] = await ethers.getSigners();
    console.log("Deploying with account:", deployer.address);

    const gmpHelper = await ethers.deployContract("GmpHelper", [
        AXELAR_GATEWAY, AXELAR_GAS_SERVICE, WST_ETH
    ]);
    const gmpHelperAddress = await gmpHelper.getAddress();
    console.log("GMP Helper address:", gmpHelperAddress);

    console.log("Waiting until tx included in block + 16 blocks…");
    await gmpHelper.deploymentTransaction().wait(16);
    await hre.run("verify", {
        address: gmpHelperAddress,
        constructorArgsParams: [AXELAR_GATEWAY, AXELAR_GAS_SERVICE, WST_ETH],
    });
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });
