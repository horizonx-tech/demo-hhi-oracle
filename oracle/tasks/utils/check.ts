import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import { OracleV1__factory } from "../../typechain-types";

const ADDR_V1 = "0x5d666338118763ca0cF6719F479491B76bc88131";
task("check:oraclev1", "check:oraclev1").setAction(
  async ({}, hre: HardhatRuntimeEnvironment) => {
    const { ethers, network } = hre;
    console.log(`[check:oraclev1] START - ${network.name}`);

    const contract = OracleV1__factory.connect(ADDR_V1, ethers.provider);
    console.log(`address: ${contract.address}`);

    const stateLength = await contract.getStateLength();
    console.log(`> stateLength: ${stateLength.toString()}`);
  }
);
