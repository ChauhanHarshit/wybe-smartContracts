import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { WybeSmartContracts } from "../target/types/wybe_smart_contracts";

describe("wybe-smart-contracts", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(anchor.AnchorProvider.env());

  const user1 = anchor.web3.Keypair.generate();
  const user2 = anchor.web3.Keypair.generate();

  const program = anchor.workspace
    .wybeSmartContracts as Program<WybeSmartContracts>;

  async function airdrop(pubkey: anchor.web3.PublicKey) {
    const sig = await provider.connection.requestAirdrop(
      pubkey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig);
  }

  before(async () => {
    await airdrop(user1.publicKey);
    await airdrop(user2.publicKey);
  })
  

  // it("Initializes the global configuration account!", async () => {
  //     const [globalConfigPDA , bump] = await anchor.web3.PublicKey.findProgramAddress(
  //       [Buffer.from("CurveConfiguration")],
  //       program.programId,
  //     );

  //     await program.methods
  //     .initialize(new anchor.BN(2))
  //     .accountsPartial({
  //       globalConfigurationAccount: globalConfigPDA,
  //       admin: user1.publicKey,
  //       rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //       systemProgram: anchor.web3.SystemProgram.programId
  //     })
  //     .signers([user1])
  //     .rpc();
      
  //     const configAccount = await program.account.curveConfiguration.fetch(globalConfigPDA);
  //     console.log("Fee stored : " , configAccount.fees.toNumber());

  //     if (configAccount.fees.toNumber() !== 2) {
  //       throw new Error("Fee mismatch!");
  //     }
  // });

      // Token launch test
  it("Create a liquidity pool"  , async() =>{
    const TOKEN_DECIMALS = 9;
    const TOTAL_TOKENS = new anchor.BN("1000000000000000000"); // 10^18
    const tokenMint = anchor.web3.Keypair.generate();

    const [poolPDA , bump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("liquidity_pool") , tokenMint.publicKey.toBuffer()],
      program.programId
    );



    const [treasuryPDA] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("treasury")],
      program.programId
    );

    const poolTokenAccount = await anchor.utils.token.associatedAddress({
      mint : tokenMint.publicKey,
      owner : poolPDA,
    });

    const treasuryTokenAccount = await anchor.utils.token.associatedAddress({
      mint : tokenMint.publicKey, 
      owner : treasuryPDA
    })

    // Send a bit more than 0.0057 SOL to cover rent etc.
    const lamports = 6_000_000
    
    const txSig = await program.methods
    .createPool()
    .accountsPartial({
      pool : poolPDA,
      tokenMint : tokenMint.publicKey,
      poolTokenAccount,
      treasury : treasuryPDA,
      treasuryTokenAccount,
      payer : user1.publicKey,
      tokenProgram : anchor.utils.token.TOKEN_PROGRAM_ID,
      associatedTokenProgram : anchor.utils.token.ASSOCIATED_PROGRAM_ID,
      rent : anchor.web3.SYSVAR_RENT_PUBKEY,
      systemProgram : anchor.web3.SystemProgram.programId
    })
    .signers([user1 , tokenMint])
    .rpc();

    console.log("TX", txSig);

    // Fetch and verify
    const poolAccount = await program.account.liquidityPool.fetch(poolPDA);
    const treasuryAtaInfo = await provider.connection.getTokenAccountBalance(treasuryTokenAccount);

    console.log("Total supply in pool account:", poolAccount.totalSupply.toString());
    console.log("Treasury ATA balance:", treasuryAtaInfo.value.amount);

    // Assertion

    const expectedPoolSupply = TOTAL_TOKENS.muln(99).divn(100);
    console.log("Expected:", expectedPoolSupply.toString());
    console.log("Actual:", poolAccount.totalSupply.toString());
    
    if (!poolAccount.totalSupply.eq(expectedPoolSupply)) {
      throw new Error("Token minting failed or incorrect token supply");
    }
    
  });

});
