import * as anchor from "@coral-xyz/anchor";
import type { Program } from "@coral-xyz/anchor";
import type NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createMint,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import type { Fundraiser } from "../target/types/fundraiser";
import * as dotenv from "dotenv";
import { clusterApiUrl } from "@solana/web3.js";
import {
  SystemProgram,
  Transaction,
  Keypair,
  sendAndConfirmTransaction,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";

describe("fundraiser", () => {
  dotenv.config({ path: "./.env" });
  //const provider = anchor.AnchorProvider.local(clusterApiUrl("devnet"));
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);

  const program = anchor.workspace.Fundraiser as Program<Fundraiser>;

  const maker = anchor.web3.Keypair.generate();

  let mint: anchor.web3.PublicKey;

  let contributorATA: anchor.web3.PublicKey;

  let makerATA: anchor.web3.PublicKey;

  const wallet = provider.wallet as NodeWallet;

  const fundraiser = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("fundraiser"), maker.publicKey.toBuffer()],
    program.programId
  )[0];

  const contributor = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("contributor"),
      fundraiser.toBuffer(),
      provider.publicKey.toBuffer(),
    ],
    program.programId
  )[0];

  const confirm = async (signature: string): Promise<string> => {
    const block = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      signature,
      ...block,
    });
    return signature;
  };

  it("Test Preparation", async () => {
    const tx = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: maker.publicKey, // create a random receiver
        lamports: 1 * anchor.web3.LAMPORTS_PER_SOL,
      })
    );
    console.log(
      `txhash: ${await sendAndConfirmTransaction(provider.connection, tx, [
        wallet.payer,
      ])}`
    );

    mint = await createMint(
      provider.connection,
      wallet.payer,
      provider.publicKey,
      provider.publicKey,
      6
    );
    console.log("Mint created", mint.toBase58());

    contributorATA = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        wallet.payer,
        mint,
        wallet.publicKey
      )
    ).address;

    makerATA = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        wallet.payer,
        mint,
        maker.publicKey
      )
    ).address;

    const mintTx = await mintTo(
      provider.connection,
      wallet.payer,
      mint,
      contributorATA,
      provider.publicKey,
      1_000_000_0
    );
    console.log("Minted 10 tokens to contributor", mintTx);
  });

  it("Initialize Fundaraiser", async () => {
    //第三个参数:允许所有者帐户是 PDA（程序派生地址）.因为fundraiser是PDA所以这里是true
    const vault = getAssociatedTokenAddressSync(mint, fundraiser, true);

    const tx = await program.methods
      .initialize(new anchor.BN(30000000), 0)
      .accounts({
        maker: maker.publicKey,
        fundraiser,
        mintToRaise: mint,
        vault,
      })
      .signers([maker])
      .rpc();
    // .then(confirm);

    console.log("\nInitialized fundraiser Account");
    console.log("Your transaction signature", tx);
  });

  it("Contribute to Fundraiser", async () => {
    const vault = getAssociatedTokenAddressSync(mint, fundraiser, true);

    const tx = await program.methods
      .contribute(new anchor.BN(1000000))
      .accounts({
        contributor: provider.publicKey,
        mintToRaise: mint,
        fundraiser,
        contributorAccount: contributor,
        contributorAta: contributorATA,
        vault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    //.then(confirm);

    console.log("\nContributed to fundraiser", tx);
    console.log("Your transaction signature", tx);

    await sleep(30);
    console.log(
      "Vault balance",
      (await provider.connection.getTokenAccountBalance(vault)).value.amount
    );

    //Account<'info, Contributor>,//存储特定贡献者迄今为止贡献的总金额
    const contributorAccount = await program.account.contributor.fetch(
      contributor
    );
    console.log("Contributor balance", contributorAccount.amount.toString());
  });
  it("Contribute to Fundraiser", async () => {
    const vault = getAssociatedTokenAddressSync(mint, fundraiser, true);

    const tx = await program.methods
      .contribute(new anchor.BN(1000000))
      .accounts({
        contributor: provider.publicKey,
        mintToRaise: mint,
        fundraiser,
        contributorAccount: contributor,
        contributorAta: contributorATA,
        vault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    //.then(confirm);

    console.log("\nContributed to fundraiser", tx);
    console.log("Your transaction signature", tx);

    await sleep(30);
    console.log(
      "Vault balance",
      (await provider.connection.getTokenAccountBalance(vault)).value.amount
    );

    const contributorAccount = await program.account.contributor.fetch(
      contributor
    );
    console.log("Contributor balance", contributorAccount.amount.toString());
  });

  //超过总目标3000W的1/10将失败
  it("Contribute to Fundraiser - Robustness Test", async () => {
    try {
      const vault = getAssociatedTokenAddressSync(mint, fundraiser, true);

      const tx = await program.methods
        .contribute(new anchor.BN(2000000))
        .accounts({
          contributor: provider.publicKey,
          mintToRaise: mint,
          fundraiser,
          contributorAccount: contributor,
          contributorAta: contributorATA,
          vault,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();
      //.then(confirm);

      console.log("\nContributed to fundraiser", tx);
      console.log("Your transaction signature", tx);

      await sleep(30);
      console.log(
        "Vault balance",
        (await provider.connection.getTokenAccountBalance(vault)).value.amount
      );

      const contributorAccount = await program.account.contributor.fetch(
        contributor
      );
      console.log("Contributor balance", contributorAccount.amount.toString());
    } catch (error) {
      console.log("\nError contributing to fundraiser");
      //console.log(error);
    }
  });

  //检查因为没达到筹款目标会报错：The amount to raise has not been met
  it("Check contributions - Robustness Test", async () => {
    try {
      const vault = getAssociatedTokenAddressSync(mint, fundraiser, true);

      const tx = await program.methods
        .checkContributions()
        .accounts({
          maker: maker.publicKey,
          mintToRaise: mint,
          fundraiser,
          makerAta: makerATA,
          vault,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([maker])
        .rpc();

      console.log("\nChecked contributions");
      console.log("Your transaction signature", tx);

      await sleep(30);
      console.log(
        "Vault balance",
        (await provider.connection.getTokenAccountBalance(vault)).value.amount
      );
    } catch (error) {
      console.log("\nError checking contributions");
      console.log(error);
    }
  });

  it("Refund Contributions", async () => {
    const vault = getAssociatedTokenAddressSync(mint, fundraiser, true);

    const contributorAccount = await program.account.contributor.fetch(
      contributor
    );
    console.log("\nContributor balance", contributorAccount.amount.toString());

    const tx = await program.methods
      .refund()
      .accounts({
        contributor: provider.publicKey,
        maker: maker.publicKey,
        mintToRaise: mint,
        fundraiser,
        contributorAccount: contributor,
        contributorAta: contributorATA,
        vault,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc()
      .then(confirm);

    console.log("\nRefunded contributions", tx);
    console.log("Your transaction signature", tx);
    console.log(
      "Vault balance",
      (await provider.connection.getTokenAccountBalance(vault)).value.amount
    );
  });
});

async function sleep(number) {
  return new Promise((resolve) => {
    setTimeout(resolve, number);
  });
}
