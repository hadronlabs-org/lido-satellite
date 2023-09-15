const { expect } = require("chai");
const {ethers} = require("hardhat");

describe("GMP Helper", function () {
    it("Static variables", async function () {
        const gmpHelper = await ethers.deployContract("GmpHelper",
            [
                "0x0000000000000000000000000000000000000001",
                "0x0000000000000000000000000000000000000002",
                "0x0000000000000000000000000000000000000003",
            ],
        );

        expect(await gmpHelper.DESTINATION_CHAIN()).to.equal("neutron");
        expect(await gmpHelper.GAS_SERVICE()).to.equal("0x0000000000000000000000000000000000000002");
        expect(await gmpHelper.GATEWAY()).to.equal("0x0000000000000000000000000000000000000001");
        expect(await gmpHelper.LIDO_SATELLITE()).to.equal("neutron1ug740qrkquxzrk2hh29qrlx3sktkfml3je7juusc2te7xmvsscns0n2wry");
        expect(await gmpHelper.WST_ETH()).to.equal("0x0000000000000000000000000000000000000003");
        expect(await gmpHelper.WSTETH_SYMBOL()).to.equal("wstETH");
    });

    it("Function calls", async function () {
        const [owner] = await ethers.getSigners();
        const axelarGateway = await  ethers.deployContract("AxelarGatewayMock");
        const axelarGasService = await ethers.deployContract("AxelarGasServiceMock");
        const wstETH = await ethers.deployContract("wstEthMock", [axelarGateway.target]);
        const gmpHelper = await ethers.deployContract("GmpHelper",
            [
                axelarGateway.target,
                axelarGasService.target,
                wstETH.target,
            ],
        );
        await axelarGateway.setGmpHelper(gmpHelper.target);
        await axelarGasService.setGmpHelper(gmpHelper.target);
        await wstETH.setGmpHelper(gmpHelper.target);

        it("send()", async function() {
            it("with zero refund address", async function() {
                await gmpHelper.send(
                    "neutron12345",
                    "10",
                    "0x0000000000000000000000000000000000000000",
                    { value: ethers.parseUnits("100", "wei") },
                );
            });

            it("with custom refund address", async function() {
                await gmpHelper.send(
                    "neutron12345",
                    "10",
                    owner.address,
                    { value: ethers.parseUnits("100", "wei") },
                );
            });
        });

        it("sendWithPermit()", async function() {
            it("with zero refund address", async function() {
                await gmpHelper.sendWithPermit(
                    "neutron12345",
                    "10",
                    "200",
                    27,
                    "0x0001020304050607080910111213141516171819202122232425262728293031",
                    "0x0001020304050607080910111213141516171819202122232425262728293031",
                    "0x0000000000000000000000000000000000000000",
                    { value: ethers.parseUnits("100", "wei") },
                );
            });

            it("with custom refund address", async function() {
                await gmpHelper.sendWithPermit(
                    "neutron12345",
                    "10",
                    "200",
                    27,
                    "0x0001020304050607080910111213141516171819202122232425262728293031",
                    "0x0001020304050607080910111213141516171819202122232425262728293031",
                    owner.address,
                    { value: ethers.parseUnits("100", "wei") },
                );
            });
        });
    });
});
