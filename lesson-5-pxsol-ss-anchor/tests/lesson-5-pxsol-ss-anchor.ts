import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Lesson5PxsolSsAnchor } from "../target/types/lesson_5_pxsol_ss_anchor";

describe("lesson-5-pxsol-ss-anchor", () => {
  // ============ 测试环境配置 ============
  // 配置 Anchor 客户端连接到本地测试网络
  anchor.setProvider(anchor.AnchorProvider.env());
  
  // 获取程序实例（从 Anchor 工作区加载，类型是 IDL 生成的）
  const program = anchor.workspace.lesson5PxsolSsAnchor as Program<Lesson5PxsolSsAnchor>;
  
  // 获取 Provider（包含 RPC 连接和钱包信息）
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  
  // 获取测试钱包（用于签名和支付费用）
  const wallet = provider.wallet as anchor.Wallet;
  
  // 推导 PDA 地址（用户数据存储账户的地址）
  // 种子：["data" + 用户公钥]
  const walletPda = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("data"), wallet.publicKey.toBuffer()],
    program.programId
  )[0];

  // ============================================================
  // 用户友好的封装函数：智能写入数据（存在就更新，不存在就创建）
  // ============================================================
  /**
   * 智能写入数据到 PDA 账户
   * - 如果账户不存在：自动创建 + 写入数据
   * - 如果账户已存在：直接更新数据（自动处理扩容/缩小）
   * 
   * @param data - 要写入的数据
   */
  async function setData(data: Buffer): Promise<void> {
    try {
      // 1. 查询账户是否存在
      const accountInfo = await provider.connection.getAccountInfo(walletPda);
      
      if (accountInfo === null) {
        // 情况1：账户不存在 → 先创建账户
        console.log("  🆕 账户不存在，正在创建...");
        await program.methods
          .init()
          .accounts({
            user: wallet.publicKey,
            userPda: walletPda,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([wallet.payer])
          .rpc();
        console.log("  ✅ 账户已创建");
      }
      
      // 2. 写入或更新数据（无论账户是新创建还是已存在）
      console.log("  💾 正在保存数据...");
      await program.methods
        .update(data)
        .accounts({
          user: wallet.publicKey,
          userPda: walletPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([wallet.payer])
        .rpc();
      console.log("  ✅ 数据已保存");
      
    } catch (error) {
      console.error("  ❌ 操作失败:", error.message);
      throw error;
    }
  }

  /**
   * 读取 PDA 账户中的数据
   */
  async function getData(): Promise<Buffer> {
    try {
      const accountData = await program.account.data.fetch(walletPda);
      return Buffer.from(accountData.data);
    } catch (error) {
      console.error("  ❌ 读取失败:", error.message);
      throw error;
    }
  }

  /**
   * 清理账户（用于测试环境重置）
   */
  async function cleanup(): Promise<void> {
    try {
      const accountInfo = await provider.connection.getAccountInfo(walletPda);
      if (accountInfo) {
        console.log("  🗑️  正在清理账户...");
        await program.methods
          .close()
          .accounts({
            user: wallet.publicKey,
            userPda: walletPda,
          })
          .signers([wallet.payer])
          .rpc();
        console.log("  ✅ 账户已清理");
        // 等待一下确保账户已关闭
        await new Promise(resolve => setTimeout(resolve, 500));
      }
    } catch (error) {
      // 账户不存在或其他错误，忽略
    }
  }

  // ============================================================
  // 测试 1：用户友好的 API（推荐使用）
  // ============================================================
  it("🚀 用户友好测试：无需关心账户是否存在，直接使用 setData()", async () => {
    console.log("\n📋 测试场景：用户不知道账户是否存在，直接保存数据\n");
    
    // 准备测试环境：空投 SOL
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        wallet.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      ),
      "confirmed"
    );

    // 清理旧账户（测试前重置）
    await cleanup();

    // 准备测试数据
    const data1 = Buffer.from("第一次保存：这是我的第一条数据");
    const data2 = Buffer.from("第二次保存：更新数据内容，账户会自动扩容");
    const data3 = Buffer.from("第三次保存：短数据");
    
    // ============================================================
    // 💡 用户体验：完全不需要关心账户是否存在！
    // ============================================================
    
    console.log("📝 步骤 1: 第一次保存数据（账户不存在）");
    await setData(data1);  // 自动创建账户 + 写入数据
    let result = await getData();
    if (!result.equals(data1)) throw new Error("数据不匹配");
    console.log(`  📄 当前数据: "${result.toString()}"\n`);
    
    console.log("📝 步骤 2: 第二次保存数据（账户已存在，需要扩容）");
    await setData(data2);  // 自动更新 + 自动扩容
    result = await getData();
    if (!result.equals(data2)) throw new Error("数据不匹配");
    console.log(`  📄 当前数据: "${result.toString()}"\n`);
    
    console.log("📝 步骤 3: 第三次保存数据（账户已存在，需要缩小）");
    await setData(data3);  // 自动更新 + 自动缩小 + 退还租金
    result = await getData();
    if (!result.equals(data3)) throw new Error("数据不匹配");
    console.log(`  📄 当前数据: "${result.toString()}"\n`);
    
    console.log("🎉 测试完成：用户完全不需要关心账户状态，体验极佳！\n");
  });

  // ============================================================
  // 测试 2：传统 API（底层操作，需要手动管理账户）
  // ============================================================
  it("⚙️  传统测试：手动管理账户（init + update）", async () => {
    console.log("\n📋 测试场景：手动管理账户创建和更新\n");
    
    // 准备测试环境
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        wallet.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      ),
      "confirmed"
    );

    // 清理旧账户
    await cleanup();

    // 准备测试数据
    const poemInitial = Buffer.from("");
    const poemEnglish = Buffer.from("The quick brown fox jumps over the lazy dog");
    const poemChinese = Buffer.from("片云天共远, 永夜月同孤.");
    
    // 辅助函数：读取数据
    const walletPdaData = async (): Promise<Buffer> => {
      let data = await program.account.data.fetch(walletPda);
      return Buffer.from(data.data);
    }

    console.log("📝 步骤 1: 手动初始化账户");
    await program.methods
      .init()
      .accounts({
        user: wallet.publicKey,
        userPda: walletPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([wallet.payer])
      .rpc();
    
    if (!(await walletPdaData()).equals(poemInitial)) throw new Error("mismatch");
    console.log("  ✅ 初始化成功，数据为空\n");

    console.log("📝 步骤 2: 手动更新（扩容）");
    await program.methods
      .update(poemEnglish)
      .accounts({
        user: wallet.publicKey,
        userPda: walletPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([wallet.payer])
      .rpc();
    
    if (!(await walletPdaData()).equals(poemEnglish)) throw new Error("mismatch");
    console.log(`  ✅ 更新成功: "${poemEnglish.toString()}"\n`);

    console.log("📝 步骤 3: 手动更新（缩小）");
    await program.methods
      .update(poemChinese)
      .accounts({
        user: wallet.publicKey,
        userPda: walletPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([wallet.payer])
      .rpc();
    
    if (!(await walletPdaData()).equals(poemChinese)) throw new Error("mismatch");
    console.log(`  ✅ 更新成功: "${poemChinese.toString()}"\n`);
  });
});
