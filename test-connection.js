const { ethers } = require("hardhat");

async function main() {
  console.log("Testing connection...");
  
  try {
    const [signer] = await ethers.getSigners();
    console.log("Signer address:", signer.address);
    
    const balance = await ethers.provider.getBalance(signer.address);
    console.log("Balance:", ethers.formatEther(balance), "ETH");
    
    const network = await ethers.provider.getNetwork();
    console.log("Network:", network.name, "Chain ID:", network.chainId);
    
  } catch (error) {
    console.error("Error:", error.message);
  }
}

main();
