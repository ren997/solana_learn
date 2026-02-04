import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Task3BlueshiftAnchorEscrow } from "../target/types/task_3_blueshift_anchor_escrow";

describe("task-3-blueshift-anchor-escrow", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.task3BlueshiftAnchorEscrow as Program<Task3BlueshiftAnchorEscrow>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
