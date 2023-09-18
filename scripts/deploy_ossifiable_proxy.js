const hre = require("hardhat");
require('dotenv').config();

async function main() {
    const {
        IMPLEMENTATION,
        ADMIN
    } = process.env;

    const [deployer] = await ethers.getSigners();
    console.log("Deploying with account:", deployer.address);

    const ossifiableProxy = await ethers.deployContract("OssifiableProxy", [
        IMPLEMENTATION, ADMIN, "0x"
    ]);
    const ossifiableProxyAddress = await ossifiableProxy.getAddress();
    console.log("Proxy address:", ossifiableProxyAddress);

    console.log("Waiting until tx included in block + 16 blocks…");
    await ossifiableProxy.deploymentTransaction().wait(16);
    await hre.run("verify", {
        address: ossifiableProxyAddress,
        constructorArgsParams: [IMPLEMENTATION, ADMIN, "0x"],
    });
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });
