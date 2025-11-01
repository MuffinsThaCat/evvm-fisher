import { expect } from "chai";
import { ethers } from "hardhat";
import { OptimizedFisher } from "../typechain-types";
import { SignerWithAddress } from "@nomicfoundation/hardhat-ethers/signers";

describe("OptimizedFisher", function () {
  let fisher: OptimizedFisher;
  let operator: SignerWithAddress;
  let user1: SignerWithAddress;
  let user2: SignerWithAddress;
  let evvmCore: string;

  beforeEach(async function () {
    [operator, user1, user2] = await ethers.getSigners();
    
    // Mock EVVM Core address (replace with actual when available)
    evvmCore = "0x0000000000000000000000000000000000000001";
    
    const FisherFactory = await ethers.getContractFactory("OptimizedFisher");
    fisher = await FisherFactory.deploy(
      evvmCore,
      100, // 1% relayer fee
      10   // Min batch size of 10
    );
    
    await fisher.waitForDeployment();
  });

  describe("Deployment", function () {
    it("Should set the correct operator", async function () {
      expect(await fisher.operator()).to.equal(operator.address);
    });

    it("Should set the correct EVVM Core address", async function () {
      expect(await fisher.evvmCore()).to.equal(evvmCore);
    });

    it("Should set the correct relayer fee", async function () {
      expect(await fisher.relayerFeeBps()).to.equal(100);
    });

    it("Should set the correct min batch size", async function () {
      expect(await fisher.minBatchSize()).to.equal(10);
    });
  });

  describe("Chunk Size Calculation", function () {
    it("Should calculate correct chunk size for 100 operations", async function () {
      // √100 * log₂(100) ≈ 10 * 6.64 ≈ 66 (allowing for integer rounding)
      const chunkSize = await fisher.calculateChunkSize(100);
      expect(Number(chunkSize)).to.be.closeTo(66, 10);
    });

    it("Should calculate correct chunk size for 1000 operations", async function () {
      // √1000 * log₂(1000) ≈ 31.62 * 9.97 ≈ 315 (allowing for integer rounding)
      const chunkSize = await fisher.calculateChunkSize(1000);
      expect(Number(chunkSize)).to.be.closeTo(315, 40);
    });

    it("Should calculate correct chunk size for 10000 operations", async function () {
      // √10000 * log₂(10000) ≈ 100 * 13.29 ≈ 1329 (allowing for integer rounding)
      const chunkSize = await fisher.calculateChunkSize(10000);
      expect(Number(chunkSize)).to.be.closeTo(1329, 100);
    });
  });

  describe("Gas Estimation", function () {
    it("Should estimate correct gas for 100 operations", async function () {
      const [estimatedGas, estimatedSavings] = await fisher.estimateGas(100);
      
      // Optimized: 14K per operation
      expect(estimatedGas).to.equal(100 * 14_000);
      
      // Traditional: 100K per operation, savings: 86K per operation
      expect(estimatedSavings).to.equal(100 * 86_000);
    });

    it("Should estimate correct gas for 1000 operations", async function () {
      const [estimatedGas, estimatedSavings] = await fisher.estimateGas(1000);
      
      expect(estimatedGas).to.equal(1000 * 14_000);
      expect(estimatedSavings).to.equal(1000 * 86_000);
    });

    it("Should show 86% gas savings", async function () {
      const [estimatedGas, estimatedSavings] = await fisher.estimateGas(1000);
      
      const traditionalGas = 1000 * 100_000;
      const savingsPercent = (Number(estimatedSavings) / traditionalGas) * 100;
      
      expect(savingsPercent).to.equal(86);
    });
  });

  describe("Admin Functions", function () {
    it("Should allow operator to update relayer fee", async function () {
      await fisher.setRelayerFee(200); // 2%
      expect(await fisher.relayerFeeBps()).to.equal(200);
    });

    it("Should revert if non-operator tries to update fee", async function () {
      await expect(
        fisher.connect(user1).setRelayerFee(200)
      ).to.be.revertedWithCustomError(fisher, "Unauthorized");
    });

    it("Should revert if fee is too high", async function () {
      await expect(
        fisher.setRelayerFee(1001) // Over 10%
      ).to.be.revertedWithCustomError(fisher, "InvalidFee");
    });

    it("Should allow operator to toggle pause", async function () {
      expect(await fisher.paused()).to.equal(false);
      
      await fisher.togglePause();
      expect(await fisher.paused()).to.equal(true);
      
      await fisher.togglePause();
      expect(await fisher.paused()).to.equal(false);
    });

    it("Should allow operator to update min batch size", async function () {
      await fisher.setMinBatchSize(50);
      expect(await fisher.minBatchSize()).to.equal(50);
    });
  });

  describe("Mathematical Correctness", function () {
    it("Should verify Williams compression provides O(√n log n) space", async function () {
      const testSizes = [100, 1000, 10000];
      
      for (const n of testSizes) {
        const chunkSize = await fisher.calculateChunkSize(n);
        const expected = Math.floor(Math.sqrt(n) * Math.log2(n));
        
        // Allow 15% tolerance for integer math rounding
        expect(Number(chunkSize)).to.be.closeTo(expected, expected * 0.15);
      }
    });

    it("Should verify memory savings scale correctly", async function () {
      // Traditional: O(n) space
      // Williams: O(√n log n) space
      
      const n1 = 100;
      const n2 = 1000;
      
      const chunk1 = Number(await fisher.calculateChunkSize(n1));
      const chunk2 = Number(await fisher.calculateChunkSize(n2));
      
      // Space ratio should be less than n ratio (proving sub-linear growth)
      const spaceRatio = chunk2 / chunk1;
      const nRatio = n2 / n1;
      
      expect(spaceRatio).to.be.lessThan(nRatio);
    });
  });
});
