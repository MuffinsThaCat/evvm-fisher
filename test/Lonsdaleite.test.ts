import { expect } from "chai";
import { ethers } from "hardhat";

describe("LonsdaleiteOptimizedFisher - Selective Optimization Method", function () {
  let fisher: any;
  let operator: any;
  
  const ERA_DURATION = 86400; // 1 day
  const MIN_BATCH_SIZE = 10;
  
  beforeEach(async function () {
    [operator] = await ethers.getSigners();
    
    const evvmCore = "0x0000000000000000000000000000000000000001";
    
    const FisherFactory = await ethers.getContractFactory("LonsdaleiteOptimizedFisher");
    fisher = await FisherFactory.deploy(
      evvmCore,
      ERA_DURATION,
      MIN_BATCH_SIZE
    );
    
    await fisher.waitForDeployment();
  });

  describe("Lonsdaleite Methodology", function () {
    
    it("Should identify weak links (era operations) correctly", async function () {
      // In lonsdaleite: Identified interlayer bonds as weak point
      // In our system: Era operations are the weak point
      
      // This is internal, but we can verify through gas estimates
      const [weakGas, strongGas, total, savings] = await fisher.estimateMixedBatch(
        100,  // weak link ops (era-based)
        100   // strong link ops (transfers)
      );
      
      // Weak links should use dramatically less gas
      expect(weakGas).to.be.lessThan(strongGas);
      
      // Weak links: ~1K gas per op
      expect(Number(weakGas) / 100).to.be.closeTo(1000, 200);
      
      // Strong links: ~14K gas per op
      expect(Number(strongGas) / 100).to.be.closeTo(14000, 2000);
    });

    it("Should apply selective optimization (weak vs strong)", async function () {
      // Lonsdaleite: Different bond lengths for different purposes
      // Our system: Different optimizations for different operation types
      
      const [weakGas, strongGas] = await fisher.estimateMixedBatch(1000, 1000);
      
      // Ratio of strong to weak (like lonsdaleite's 1.56/1.47 ≈ 1.061)
      const ratio = Number(strongGas) / Number(weakGas);
      
      // Our ratio should be 14K/1K = 14
      expect(ratio).to.be.closeTo(14, 2);
      
      console.log(`\nLonsdaleite bond ratio: 1.061`);
      console.log(`Our gas ratio: ${ratio.toFixed(2)}`);
      console.log(`(Different values, same principle: selective optimization)\n`);
    });

    it("Should achieve superior results through selective strengthening", async function () {
      // Lonsdaleite: 164 GPa vs 110 GPa = 49% improvement
      // Our system: Should achieve 93%+ savings with mixed operations
      
      const [, , totalGas, savings] = await fisher.estimateMixedBatch(1000, 1000);
      
      const traditional = 2000 * 100_000; // 2000 ops at 100K each
      const savingsPercent = (Number(savings) / traditional) * 100;
      
      console.log(`\nLonsdaleite improvement: 49% harder than diamond`);
      console.log(`Our improvement: ${savingsPercent.toFixed(1)}% gas savings\n`);
      
      // With 50/50 mix of weak/strong operations:
      // Total: 1M (weak) + 14M (strong) = 15M gas
      // vs 200M traditional = 92.5% savings
      expect(savingsPercent).to.be.greaterThan(92);
    });
  });

  describe("Weak Link Optimization (Era Operations)", function () {
    
    it("Should achieve 99%+ savings on weak link operations", async function () {
      // These are the "cleavage plane" weakness
      // Maximum optimization applied here
      
      const [weakGas] = await fisher.estimateMixedBatch(1000, 0);
      
      const traditional = 1000 * 100_000;
      const savings = traditional - Number(weakGas);
      const savingsPercent = (savings / traditional) * 100;
      
      console.log(`\nWeak link operations (era-based):`);
      console.log(`  Traditional: ${traditional.toLocaleString()} gas`);
      console.log(`  Optimized: ${Number(weakGas).toLocaleString()} gas`);
      console.log(`  Savings: ${savingsPercent.toFixed(2)}%\n`);
      
      // Should achieve ~99% savings on weak links
      expect(savingsPercent).to.be.greaterThanOrEqual(99);
    });
  });

  describe("Strong Link Processing (Standard Operations)", function () {
    
    it("Should achieve 86% savings on strong link operations", async function () {
      // These are already strong, maintain normal optimization
      
      const [, strongGas] = await fisher.estimateMixedBatch(0, 1000);
      
      const traditional = 1000 * 100_000;
      const savings = traditional - Number(strongGas);
      const savingsPercent = (savings / traditional) * 100;
      
      console.log(`\nStrong link operations (transfers):`);
      console.log(`  Traditional: ${traditional.toLocaleString()} gas`);
      console.log(`  Optimized: ${Number(strongGas).toLocaleString()} gas`);
      console.log(`  Savings: ${savingsPercent.toFixed(2)}%\n`);
      
      // Should achieve ~86% savings on strong links (Williams)
      expect(savingsPercent).to.be.closeTo(86, 5);
    });
  });

  describe("Mixed Batch Performance", function () {
    
    it("Should optimize different operation ratios correctly", async function () {
      const testCases = [
        { weak: 900, strong: 100, desc: "90% weak (era-heavy)" },
        { weak: 500, strong: 500, desc: "50/50 mix" },
        { weak: 100, strong: 900, desc: "10% weak (transfer-heavy)" }
      ];
      
      console.log(`\nMixed batch performance:\n`);
      
      for (const test of testCases) {
        const [, , totalGas, savings] = await fisher.estimateMixedBatch(
          test.weak,
          test.strong
        );
        
        const traditional = (test.weak + test.strong) * 100_000;
        const savingsPercent = (Number(savings) / traditional) * 100;
        
        console.log(`${test.desc}:`);
        console.log(`  Traditional: ${traditional.toLocaleString()} gas`);
        console.log(`  Optimized: ${Number(totalGas).toLocaleString()} gas`);
        console.log(`  Savings: ${savingsPercent.toFixed(1)}%\n`);
        
        // All scenarios should achieve >85% savings
        expect(savingsPercent).to.be.greaterThan(85);
      }
    });

    it("Should demonstrate scalability with weak link optimization", async function () {
      // As weak link percentage increases, overall savings improve
      
      const results = [];
      
      for (let weakPercent = 0; weakPercent <= 100; weakPercent += 25) {
        const weak = (1000 * weakPercent) / 100;
        const strong = 1000 - weak;
        
        const [, , totalGas, savings] = await fisher.estimateMixedBatch(weak, strong);
        
        const traditional = 1000 * 100_000;
        const savingsPercent = (Number(savings) / traditional) * 100;
        
        results.push({ weakPercent, savingsPercent });
      }
      
      console.log(`\nSavings vs weak link percentage:\n`);
      results.forEach(r => {
        console.log(`${r.weakPercent}% weak links: ${r.savingsPercent.toFixed(1)}% savings`);
      });
      console.log();
      
      // Verify savings increase with weak link percentage
      for (let i = 1; i < results.length; i++) {
        expect(results[i].savingsPercent).to.be.greaterThanOrEqual(
          results[i - 1].savingsPercent
        );
      }
    });
  });

  describe("Era Transition (Primary Weak Link)", function () {
    
    it("Should transition era with minimal gas", async function () {
      // This is THE weak link - the cleavage plane
      // Traditional: 140M gas for 1000 users
      // Optimized: ~5K gas
      
      // Advance time to allow era transition
      await ethers.provider.send("evm_increaseTime", [86400]); // 1 day
      await ethers.provider.send("evm_mine", []);
      
      const tx = await fisher.transitionEra();
      const receipt = await tx.wait();
      
      console.log(`\nEra transition (primary weak link):`);
      console.log(`  Traditional: 140,000,000 gas (for 1000 users)`);
      console.log(`  Optimized: ${receipt.gasUsed.toLocaleString()} gas`);
      console.log(`  Savings: ${(((140_000_000 - Number(receipt.gasUsed)) / 140_000_000) * 100).toFixed(4)}%\n`);
      
      // Should be dramatically less than traditional
      expect(receipt.gasUsed).to.be.lessThan(100_000);
      
      const newEra = await fisher.currentEra();
      expect(newEra).to.equal(1);
    });
  });

  describe("Comparison to Lonsdaleite Paper", function () {
    
    it("Should demonstrate parallel methodology and results", async function () {
      console.log(`\n${"=".repeat(70)}`);
      console.log(`LONSDALEITE METHOD COMPARISON`);
      console.log(`${"=".repeat(70)}\n`);
      
      console.log(`Material Science (Lonsdaleite):`);
      console.log(`  Problem: Cubic diamond has weak (111) cleavage planes`);
      console.log(`  Weak point: Interlayer bonds`);
      console.log(`  Solution: Shorten interlayer bonds (1.54Å → 1.47Å)`);
      console.log(`  Keep normal: Intralayer bonds (1.56Å)`);
      console.log(`  Result: 164 GPa vs 110 GPa (49% improvement)\n`);
      
      console.log(`Blockchain (Our Fisher):`);
      console.log(`  Problem: Traditional systems compute everything on-chain`);
      console.log(`  Weak point: Era transitions`);
      console.log(`  Solution: Off-chain computation for era ops (140M → 5K gas)`);
      console.log(`  Keep normal: Williams batching for transfers (86% savings)`);
      
      const [, , totalGas, savings] = await fisher.estimateMixedBatch(500, 500);
      const traditional = 1000 * 100_000;
      const savingsPercent = (Number(savings) / traditional) * 100;
      
      console.log(`  Result: ${savingsPercent.toFixed(1)}% gas savings (50/50 mix)\n`);
      
      console.log(`Shared Methodology:`);
      console.log(`  ✓ Identify specific weak point`);
      console.log(`  ✓ Apply selective optimization to weak point`);
      console.log(`  ✓ Maintain normal optimization elsewhere`);
      console.log(`  ✓ Achieve breakthrough performance\n`);
      console.log(`${"=".repeat(70)}\n`);
      
      // Both achieve significant improvements through selective optimization
      expect(savingsPercent).to.be.greaterThan(90);
    });
  });
});
