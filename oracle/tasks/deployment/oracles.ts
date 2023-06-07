import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import { OracleV1, OracleV1__factory } from "../../typechain-types";

task("deploy:oraclev1", "deploy:oraclev1")
  .addOptionalParam("deployer", "deployer")
  .setAction(
    async (
      { deployer }: { deployer: string },
      hre: HardhatRuntimeEnvironment
    ) => {
      const { ethers, network, upgrades } = hre;
      console.log(`[deploy:oraclev1] START - ${network.name}`);

      const _deployer = deployer
        ? await ethers.getSigner(deployer)
        : (await ethers.getSigners())[0];

      // Deployment
      const contract = (await upgrades.deployProxy(
        new OracleV1__factory(_deployer)
      )) as OracleV1;
      console.log(`deployed tx: ${contract.deployTransaction.hash}`);
      await contract.deployTransaction.wait();
      console.log(`deployed! address: ${contract.address}`);

      // Check after deploying
      console.log(`Check phase`);
      console.log(`> version: ${(await contract.version()).toString()}`);

      // Verification
      if (network.name !== "hardhat") {
        await hre.run("verify:verify", {
          address: contract.address,
          constructorArguments: [],
        });
      }

      console.log(`[deploy:oraclev1] END`);
    }
  );
