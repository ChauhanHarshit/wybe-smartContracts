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
  

  it("Initializes the global configuration account!", async () => {
      const [globalConfigPDA , bump] = await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("CurveConfiguration")],
        program.programId,
      );

      await program.methods
      .initialize(new anchor.BN(2))
      .accountsPartial({
        globalConfigurationAccount: globalConfigPDA,
        admin: user1.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .signers([user1])
      .rpc();
      
      const configAccount = await program.account.curveConfiguration.fetch(globalConfigPDA);
      console.log("Fee stored : " , configAccount.fees.toNumber());

      if (configAccount.fees.toNumber() !== 2) {
        throw new Error("Fee mismatch!");
      }

  });
});
