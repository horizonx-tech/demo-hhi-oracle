import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";

task(
  "utils:accounts",
  "Prints the list of accounts",
  async (_, hre: HardhatRuntimeEnvironment) => {
    const { ethers, network } = hre;
    console.log(`[utils:accounts] network: ${network.name}`);
    const accounts = await ethers.getSigners();

    for await (const account of accounts) {
      const balance = await ethers.provider.getBalance(account.address);
      console.log(`${account.address}: ${ethers.utils.formatEther(balance)}`);
    }
  }
);

task("utils:send-native-token", "utils:send-native-token").setAction(
  async (_, hre: HardhatRuntimeEnvironment) => {
    const { ethers } = hre;
    const signer = (await ethers.getSigners())[0];
    const tx = {
      to: signer.address,
      // Convert currency unit from ether to wei
      value: ethers.utils.parseEther("0.01"),
      gasLimit: ethers.BigNumber.from(21000),
      maxPriorityFeePerGas: ethers.BigNumber.from(10000000000000), // 10,000 gwei
      maxFeePerGas: ethers.BigNumber.from(10000000000000), // 10,000 gwei
      // nonce: ethers.BigNumber.from(3010),
    };
    const res = await signer.sendTransaction(tx);
    console.log(`tx.hash: ${res.hash}`);
    await res.wait();
  }
);
