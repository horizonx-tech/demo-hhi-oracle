import { ethers, upgrades } from "hardhat";
import {
  OracleV1,
  OracleV1__factory,
  UpgradedOracleV1__factory,
} from "../typechain-types";
import { expect } from "chai";

describe("OracleV1", () => {
  const setup = async () => {
    const [deployer, user] = await ethers.getSigners();

    const oracle = (await upgrades.deployProxy(
      new OracleV1__factory(deployer)
    )) as OracleV1;
    await oracle.deployTransaction.wait();

    return { deployer, user, oracle };
  };
  it(".version", async () => {
    const { oracle } = await setup();
    expect((await oracle.version()).toString()).eq("1");
  });
  it(".updateState", async () => {
    const { deployer, user, oracle } = await setup();
    await oracle.connect(deployer).updateState(100);
    await oracle.connect(user).updateState(200);
    expect((await oracle.state(deployer.address)).toString()).eq("100");
    expect((await oracle.state(user.address)).toString()).eq("200");

    await oracle.connect(user).updateState(1000);
    await oracle.connect(deployer).updateState(20);
    expect((await oracle.state(deployer.address)).toString()).eq("20");
    expect((await oracle.state(user.address)).toString()).eq("1000");
  });
  it("upgradable", async () => {
    const { deployer, oracle } = await setup();
    const upgraded = await upgrades.upgradeProxy(
      oracle,
      new UpgradedOracleV1__factory(deployer)
    );
    expect((await upgraded.version()).toString()).eq("2");
  });
});
