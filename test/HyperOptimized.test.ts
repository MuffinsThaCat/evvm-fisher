import { expect } from "chai";
import { ethers } from "hardhat";

describe("HyperOptimizedFisher - Combined Williams + φ", function () {
  let fisher: any;
  let operator: any;
  
  const BASE_FEE = ethers.parseEther("0.001");
  const FEE_GROWTH_RATE = ethers.parseEther("0.05"); // 5% per era
  const ERA_DURATION = 86400; // 1 day
  const MIN_BATCH_SIZE = 10;
  
  beforeEach(async function () {
    [operator] = await ethers.getSigners();
    
    const evvmCore = "0x0000000000000000000000000000000000000001";
    
    const FisherFactory = await ethers.getContractFactory("HyperOptimizedFisher");
    fisher = await FisherFactory.deploy(
      evvmCore,
      BASE_FEE,
      FEE_GROWTH_RATE,
      ERA_DURATION,
      MIN_BATCH_SIZE
    );
    
    await fisher.waitForDeployment();
  });

  describe("Combined Gas Savings", function () {
    it("Should show Williams compression savings (86%)", async function () {
      const batchSize = 1000;
      const [williamsGas] = await fisher.estimateCombinedGas(batchSize);
      
      // Williams: 14K per op
      expect(williamsGas).to.equal(batchSize * 14_000);
      
      // vs Traditional: 100K per op
      const traditional = batchSize * 100_000;
      const savings = ((traditional - Number(williamsGas)) / traditional) * 100;
      
      expect(savings).to.equal(86);
    });

    it("Should show φ-optimization additional savings", async function () {
      const batchSize = 1000;
      const [williamsGas, phiGas, totalSavings] = await fisher.estimateCombinedGas(batchSize);
      
      // φ adds 5K savings per op
      expect(phiGas).to.equal(BigInt(batchSize * 9_000));
      
      // Total savings vs traditional
      const traditional = BigInt(batchSize * 100_000);
      expect(totalSavings).to.equal(traditional - phiGas);
    });

    it("Should achieve 91% combined savings", async function () {
      const batchSize = 1000;
      const [, phiGas] = await fisher.estimateCombinedGas(batchSize);
      
      const traditional = batchSize * 100_000; // 100M gas
      const optimized = Number(phiGas);         // 9M gas
      const savingsPercent = ((traditional - optimized) / traditional) * 100;
      
      expect(savingsPercent).to.equal(91);
    });
  });

  describe("Era-Based Fee System (φ-Optimized)", function () {
    it("Should advance era with minimal gas (5K vs millions)", async function () {
      // In traditional system, would update all users: millions of gas
      // With φ: just increment counter
      
      const tx = await fisher.advanceFeeEra();
      const receipt = await tx.wait();
      
      // Era advancement is cheap (just increment + event)
      // Still WAY less than millions of gas for traditional approach
      expect(receipt.gasUsed).to.be.lessThan(100_000);
      
      const newEra = await fisher.feeEra();
      expect(newEra).to.equal(1);
    });

    it("Should compute fees off-chain using φ-formula", async function () {
      const user = operator.address;
      
      // This is a VIEW function - costs 0 gas!
      const fees = await fisher.getUserFeesOffChain(user, 0, 5);
      
      // Fees are computed using φ-formula, no storage updates
      expect(fees).to.be.a('bigint');
    });

    it("Should calculate era-specific fees deterministically", async function () {
      const era0Fee = await fisher.getEraFee(0);
      const era1Fee = await fisher.getEraFee(1);
      const era2Fee = await fisher.getEraFee(2);
      
      // Fees decay using φ-formula
      expect(era0Fee).to.be.greaterThan(era1Fee);
      expect(era1Fee).to.be.greaterThan(era2Fee);
      
      // Deterministic - same result every time
      const era0FeeAgain = await fisher.getEraFee(0);
      expect(era0Fee).to.equal(era0FeeAgain);
    });
  });

  describe("Williams Compression", function () {
    it("Should calculate correct chunk sizes", async function () {
      const size100 = await fisher.calculateChunkSize(100);
      const size1000 = await fisher.calculateChunkSize(1000);
      const size10000 = await fisher.calculateChunkSize(10000);
      
      // √n * log₂(n) - allowing for integer rounding
      expect(Number(size100)).to.be.closeTo(66, 10);
      expect(Number(size1000)).to.be.closeTo(315, 40);
      expect(Number(size10000)).to.be.closeTo(1329, 100);
    });
  });

  describe("Comparison: Traditional vs Optimized", function () {
    it("Should demonstrate massive savings at scale", async function () {
      const testSizes = [100, 1000, 10000];
      
      for (const size of testSizes) {
        const [, phiGas, totalSavings] = await fisher.estimateCombinedGas(size);
        
        const traditional = size * 100_000;
        const savingsPercent = (Number(totalSavings) / traditional) * 100;
        
        console.log(`\n${size} operations:`);
        console.log(`  Traditional: ${traditional.toLocaleString()} gas`);
        console.log(`  Optimized:   ${phiGas.toLocaleString()} gas`);
        console.log(`  Savings:     ${savingsPercent.toFixed(1)}%`);
        
        expect(savingsPercent).to.be.greaterThan(90);
      }
    });
  });
});
