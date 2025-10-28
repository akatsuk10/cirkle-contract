import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CirkleContract } from "../target/types/cirkle_contract";
import { PublicKey } from "@solana/web3.js";
import * as assert from "assert";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAccount,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { associatedAddress } from "@coral-xyz/anchor/dist/cjs/utils/token";

describe("buy_token tests (SOL-based) - FIXED", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.CirkleContract as Program<CirkleContract>;

  let user = provider.wallet;
  let admin = anchor.web3.Keypair.generate();
  let vaultPda: PublicKey;
  let vaultBump: number;
  const cityName = "TestCity";

  // Price of SOL in USD (for test calculations)
  const solPriceUsd = new anchor.BN(200); // 1 SOL = $200

  before(async () => {
    // Derive vault PDA
    [vaultPda, vaultBump] = await PublicKey.findProgramAddress(
      [Buffer.from("protocol_admin"), admin.publicKey.toBuffer()],
      program.programId
    );
    console.log("Vault PDA:", vaultPda.toBase58(), "Bump:", vaultBump);

    // Fund admin
    const airdropSig = await provider.connection.requestAirdrop(
      admin.publicKey,
      10_000_000_000 // 10 SOL to ensure integer math doesn't truncate
    );
    await provider.connection.confirmTransaction(airdropSig, "confirmed");
    console.log("Admin funded:", admin.publicKey.toBase58());

    // Initialize vault
    await program.methods
      .vaultInitialize()
      .accountsPartial({
        admin: admin.publicKey,
        adminVault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([admin])
      .rpc();
    console.log("Vault initialized for admin:", admin.publicKey.toBase58());
  });

  it("should successfully buy tokens using SOL units", async () => {
    const lamports = new anchor.BN(1_000_000_000); // 1 SOL
    const circleRate = new anchor.BN(10); // $10 per city token to get >0 tokens

    const [cityConfigPda] = await PublicKey.findProgramAddress(
      [Buffer.from("city-config"), Buffer.from(cityName)],
      program.programId
    );
    const [cityMintPda] = await PublicKey.findProgramAddress(
      [Buffer.from("city-mint"), Buffer.from(cityName)],
      program.programId
    );
    const userAta = await associatedAddress({ mint: cityMintPda, owner: user.publicKey });

    const txSig = await program.methods
      .buy(cityName, circleRate, lamports, solPriceUsd)
      .accountsPartial({
        user: user.publicKey,
        admin: admin.publicKey,
        vault: vaultPda,
        cityConfig: cityConfigPda,
        cityMint: cityMintPda,
        userAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();
    console.log("Buy transaction signature:", txSig);

    const vaultAccount = await program.account.vault.fetch(vaultPda);
    assert.ok(vaultAccount.balance.gte(lamports));

    const cityConfig = await program.account.cityConfig.fetch(cityConfigPda);
    assert.equal(cityConfig.cityName, cityName);
    assert.ok(cityConfig.mint.equals(cityMintPda));

    const tokenAccount = await getAccount(provider.connection, userAta);

    // ✅ Correct integer calculation using BN
    const expectedAmount = lamports
      .div(new anchor.BN(1_000_000_000)) // lamports → SOL
      .mul(solPriceUsd)                  // SOL → USD
      .div(circleRate)                   // USD → token units
      .mul(new anchor.BN(1_000_000));   // decimals

    assert.equal(tokenAccount.amount.toString(), expectedAmount.toString());
  });

  it("should fail if circle_rate is 0", async () => {
    const lamports = new anchor.BN(1_000_000_000);
    const circleRate = new anchor.BN(0);

    const [cityConfigPda] = await PublicKey.findProgramAddress(
      [Buffer.from("city-config"), Buffer.from(cityName)],
      program.programId
    );
    const [cityMintPda] = await PublicKey.findProgramAddress(
      [Buffer.from("city-mint"), Buffer.from(cityName)],
      program.programId
    );
    const userAta = await associatedAddress({ mint: cityMintPda, owner: user.publicKey });

    try {
      await program.methods
        .buy(cityName, circleRate, lamports, solPriceUsd)
        .accountsPartial({
          user: user.publicKey,
          admin: admin.publicKey,
          vault: vaultPda,
          cityConfig: cityConfigPda,
          cityMint: cityMintPda,
          userAta,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .rpc();
      assert.fail("Transaction should have failed due to zero rate");
    } catch (err: any) {
      const errorString = err.toString();
      const logs = err.logs?.join("\n") || "";
      assert.ok(
        errorString.includes("RateNotValid") || 
        logs.includes("RateNotValid") ||
        err.error?.errorCode?.code === "RateNotValid"
      );
    }
  });

  it("should mint to existing ATA if already exists", async () => {
    const lamports = new anchor.BN(1_000_000_000);
    const circleRate = new anchor.BN(5); // $5 per token

    const [cityConfigPda] = await PublicKey.findProgramAddress(
      [Buffer.from("city-config"), Buffer.from(cityName)],
      program.programId
    );
    const [cityMintPda] = await PublicKey.findProgramAddress(
      [Buffer.from("city-mint"), Buffer.from(cityName)],
      program.programId
    );
    const userAta = await associatedAddress({ mint: cityMintPda, owner: user.publicKey });

    let balanceBefore = new anchor.BN(0);
    try {
      const accountBefore = await getAccount(provider.connection, userAta);
      balanceBefore = new anchor.BN(accountBefore.amount.toString());
    } catch {}

    await program.methods
      .buy(cityName, circleRate, lamports, solPriceUsd)
      .accountsPartial({
        user: user.publicKey,
        admin: admin.publicKey,
        vault: vaultPda,
        cityConfig: cityConfigPda,
        cityMint: cityMintPda,
        userAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const tokenAccount = await getAccount(provider.connection, userAta);

    const expectedIncrease = lamports
      .div(new anchor.BN(1_000_000_000))
      .mul(solPriceUsd)
      .div(circleRate)
      .mul(new anchor.BN(1_000_000));

    const expectedTotal = balanceBefore.add(expectedIncrease);

    assert.equal(tokenAccount.amount.toString(), expectedTotal.toString());
  });
});
