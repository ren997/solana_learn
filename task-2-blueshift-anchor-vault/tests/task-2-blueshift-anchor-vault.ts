import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Task2BlueshiftAnchorVault } from "../target/types/task_2_blueshift_anchor_vault";

describe("task-2-blueshift-anchor-vault", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.task2BlueshiftAnchorVault as Program<Task2BlueshiftAnchorVault>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
