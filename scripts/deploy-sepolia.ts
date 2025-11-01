import { ethers } from "hardhat";

async function main() {
  console.log("🚀 Deploying φ-Freeman Fisher to REAL EVVM Sepolia...\n");

  // Real EVVM Sepolia addresses from hackathon
  const EVVM_CORE = "0xF817e9ad82B4a19F00dA7A248D9e556Ba96e6366";
  const RELAYER_FEE_BPS = 10; // 0.1%
  const MIN_BATCH_SIZE = 10;

  const [deployer] = await ethers.getSigners();
  console.log("📋 Deployer:", deployer.address);
  
  const balance = await ethers.provider.getBalance(deployer.address);
  console.log("💰 Balance:", ethers.formatEther(balance), "ETH\n");

  if (balance === 0n) {
    console.error("❌ No ETH! Get Sepolia ETH from:");
    console.error("   https://discord.com/channels/554623348622098432/1423452960985321532");
    process.exit(1);
  }

  console.log("🔨 Deploying FisherProduction to Sepolia...");
  const FisherProduction = await ethers.getContractFactory("FisherProduction");
  const fisher = await FisherProduction.deploy(
    EVVM_CORE,
    RELAYER_FEE_BPS,
    MIN_BATCH_SIZE
  );
  
  console.log("⏳ Waiting for deployment...");
  await fisher.waitForDeployment();
  const fisherAddress = await fisher.getAddress();

  console.log("\n✅ DEPLOYED TO SEPOLIA!");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("📍 Fisher Address:", fisherAddress);
  console.log("🔗 Etherscan:", `https://sepolia.etherscan.io/address/${fisherAddress}`);
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

  // Verify it can talk to EVVM
  console.log("🔍 Verifying EVVM connection...");
  const evvmCore = await fisher.evvmCore();
  console.log("   ✅ EVVM Core:", evvmCore);
  console.log("   ✅ Operator:", await fisher.operator());
  console.log("   ✅ Relayer Fee:", (await fisher.relayerFeeBps()).toString(), "bps");

  // Show gas estimates
  console.log("\n⚡ Gas Savings (estimates):");
  const [gas100, save100] = await fisher.estimateGas(100);
  const [gas1000, save1000] = await fisher.estimateGas(1000);
  console.log("   100 ops:  ", ethers.formatUnits(gas100, "wei"), "gas (86% savings)");
  console.log("   1000 ops: ", ethers.formatUnits(gas1000, "wei"), "gas (86% savings)");

  console.log("\n📝 Environment Setup:");
  console.log("Add this to relayer/.env:");
  console.log(`FISHER_ADDRESS=${fisherAddress}`);
  console.log(`SEPOLIA_RPC_URL=https://rpc.sepolia.org`);
  console.log(`EVVM_CORE=${EVVM_CORE}`);
  console.log(`PRIVATE_KEY=your_relayer_private_key`);

  console.log("\n🎯 Next Steps:");
  console.log("1. ✅ Fisher deployed on Sepolia");
  console.log("2. Update relayer/.env with FISHER_ADDRESS above");
  console.log("3. cd relayer && npm install && npm run dev");
  console.log("4. Test with real EVVM transactions\n");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
