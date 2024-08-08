import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Tokenix } from "../target/types/tokenix";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress, createAssociatedTokenAccountInstruction } from "@solana/spl-token";
import { expect } from "chai";

describe("tokenix", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Tokenix as Program<Tokenix>;
  const wallet = provider.wallet as anchor.Wallet;

  let mint: Keypair;
  let tokenAccount: PublicKey;
  let pool: PublicKey;
  let poolTokenAccount: PublicKey;

  it("Creates a token", async () => {
    mint = Keypair.generate();
    const name = "Test Token";
    const symbol = "TEST";
    const uri = "https://example.com/metadata.json";
    const initialSupply = new anchor.BN(100_000_000).mul(new anchor.BN(10).pow(new anchor.BN(9)));

    tokenAccount = await getAssociatedTokenAddress(mint.publicKey, wallet.publicKey);

    try {
      const tx = await program.methods
        .createToken(name, symbol, uri, initialSupply)
        .accounts({
          authority: wallet.publicKey,
          mint: mint.publicKey,
          tokenAccount: tokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        })
        .signers([mint])
        .rpc();

      console.log("=== Token Creation ===");
      console.log("Token creation TX:", tx);
      console.log("Token address (mint):", mint.publicKey.toString());
      console.log("Initial supply:", initialSupply.toString());
      console.log("Minted to (token account):", tokenAccount.toString());

      const mintInfo = await provider.connection.getAccountInfo(mint.publicKey);
      expect(mintInfo).to.not.be.null;
    } catch (error) {
      console.error("Error creating token:", error);
      throw error;
    }
  });

  it("Creates a pool", async () => {
    const [poolPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), mint.publicKey.toBuffer()],
      program.programId
    );
    pool = poolPda;

    poolTokenAccount = await getAssociatedTokenAddress(mint.publicKey, pool, true);

    const initialPrice = new anchor.BN(10000); // 0.00001 SOL

    try {
      const tx = await program.methods
        .createPool(initialPrice)
        .accounts({
          authority: wallet.publicKey,
          pool: pool,
          mint: mint.publicKey,
          authorityTokenAccount: tokenAccount,
          poolTokenAccount: poolTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        })
        .rpc();

      console.log("=== Pool Creation ===");
      console.log("Pool creation TX:", tx);
      console.log("Pool address (bonding curve):", pool.toString());
      console.log("Pool token account:", poolTokenAccount.toString());

      const poolInfo = await program.account.pool.fetch(pool);
      console.log("Pool initial price:", poolInfo.currentPrice.toString());
      console.log("Pool total supply:", poolInfo.totalSupply.toString());

      expect(poolInfo.mint.toString()).to.equal(mint.publicKey.toString());
      expect(poolInfo.tokenAccount.toString()).to.equal(poolTokenAccount.toString());
      expect(poolInfo.currentPrice.toString()).to.equal(initialPrice.toString());
    } catch (error) {
      console.error("Error creating pool:", error);
      throw error;
    }
  });

  async function logPoolInfo() {
    const poolInfo = await program.account.pool.fetch(pool);
    console.log("Pool info:");
    console.log("  Total supply:", poolInfo.totalSupply.toString());
    console.log("  Current price:", poolInfo.currentPrice.toString());
  }

  it("Buys tokens", async () => {
    const buyer = Keypair.generate();
    const airdropAmount = 10_000_000_000_000; // 10,000 SOL
    await provider.connection.requestAirdrop(buyer.publicKey, airdropAmount);
    await provider.connection.confirmTransaction(await provider.connection.requestAirdrop(buyer.publicKey, airdropAmount));
    
    const buyerTokenAccount = await getAssociatedTokenAddress(mint.publicKey, buyer.publicKey);
    
    const amount = new anchor.BN(1); // 1 token

    try {
      console.log("=== Token Purchase ===");
      console.log("Buyer address:", buyer.publicKey.toString());
      console.log("Buyer initial balance:", await provider.connection.getBalance(buyer.publicKey));
      console.log("Amount to buy:", amount.toString());

      console.log("Pool info before purchase:");
      await logPoolInfo();

      // Initialize buyer token account
      const createAtaIx = createAssociatedTokenAccountInstruction(
        buyer.publicKey,
        buyerTokenAccount,
        buyer.publicKey,
        mint.publicKey
      );

      // Create transaction to buy token
      const buyTx = await program.methods
        .buyToken(amount)
        .accounts({
          buyer: buyer.publicKey,
          pool: pool,
          poolTokenAccount: poolTokenAccount,
          buyerTokenAccount: buyerTokenAccount,
          mint: mint.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .transaction();

      // Combine ATA creation instruction with buy transaction
      const tx = new anchor.web3.Transaction().add(createAtaIx, buyTx);

      // Send and confirm transaction
      const txSignature = await provider.sendAndConfirm(tx, [buyer]);
      console.log("Buy transaction signature:", txSignature);

      console.log("Buyer final balance:", await provider.connection.getBalance(buyer.publicKey));
      const buyerTokenBalance = await provider.connection.getTokenAccountBalance(buyerTokenAccount);
      console.log("Buyer token balance:", buyerTokenBalance.value.uiAmount);

      console.log("Pool info after purchase:");
      await logPoolInfo();
    } catch (error) {
      console.error("Error buying tokens:", error);
      if (error instanceof anchor.AnchorError) {
        console.error("Error code:", error.error.errorCode.code);
        console.error("Error message:", error.error.errorMessage);
      }
      throw error;
    }
  });
});