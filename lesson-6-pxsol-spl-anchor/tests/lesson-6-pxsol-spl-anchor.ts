import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Lesson6PxsolSplAnchor } from "../target/types/lesson_6_pxsol_spl_anchor";

describe("lesson-6-pxsol-spl-anchor", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.lesson6PxsolSplAnchor as Program<Lesson6PxsolSplAnchor>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
