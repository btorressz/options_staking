describe("options_staking", () => {
  // Test for staking an option
  it("stakes an option successfully", async () => {
    // Generate keypairs for the user account, staking pool, and option contract.
    const userAccountKp = new web3.Keypair();
    const stakingPoolKp = new web3.Keypair();
    const optionContractKp = new web3.Keypair();

    // Airdrop lamports to each account (adjust the amount as needed).
    let sig = await pg.connection.requestAirdrop(userAccountKp.publicKey, 1e9);
    await pg.connection.confirmTransaction(sig);
    sig = await pg.connection.requestAirdrop(stakingPoolKp.publicKey, 1e9);
    await pg.connection.confirmTransaction(sig);
    sig = await pg.connection.requestAirdrop(optionContractKp.publicKey, 1e9);
    await pg.connection.confirmTransaction(sig);

    // Define test parameters.
    const lockPeriod = new BN(60); // Example: 60 seconds lock period.
    const optionType = 0; // 0 represents a Call option.

    // Pass the option contract public key as the first parameter.
    const txHash = await pg.program.methods
      .stakeOptions(optionContractKp.publicKey, lockPeriod, optionType)
      .accounts({
        userAccount: userAccountKp.publicKey,
        stakingPool: stakingPoolKp.publicKey,
        optionContract: optionContractKp.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([userAccountKp, stakingPoolKp, optionContractKp])
      .rpc();
    console.log("stakeOptions transaction:", txHash);

    // Retrieve the user account and option contract account data.
    const userAccount = await pg.program.account.userAccount.fetch(userAccountKp.publicKey);
    const optionContract = await pg.program.account.optionContract.fetch(optionContractKp.publicKey);

    console.log("UserAccount on-chain data:", userAccount);
    console.log("OptionContract on-chain data:", optionContract);

    // Ensure that the option is now locked and recorded in the user's staked options.
    assert(optionContract.locked === true, "Option contract should be locked");
    assert(
      userAccount.stakedOptions.some((pub: web3.PublicKey) =>
        pub.equals(optionContractKp.publicKey)
      ),
      "User account should include the staked option"
    );
  });

  // Test for unstaking with an early unstake and claiming rewards.
  it("unstakes an option early and claims rewards", async () => {
    // Generate keypairs for the user account, staking pool, and option contract.
    const userAccountKp = new web3.Keypair();
    const stakingPoolKp = new web3.Keypair();
    const optionContractKp = new web3.Keypair();

    // Airdrop lamports for all accounts.
    let sig = await pg.connection.requestAirdrop(userAccountKp.publicKey, 1e9);
    await pg.connection.confirmTransaction(sig);
    sig = await pg.connection.requestAirdrop(stakingPoolKp.publicKey, 1e9);
    await pg.connection.confirmTransaction(sig);
    sig = await pg.connection.requestAirdrop(optionContractKp.publicKey, 1e9);
    await pg.connection.confirmTransaction(sig);

    // Stake the option first.
    const lockPeriod = new BN(60); // 60 seconds lock period.
    const optionType = 0;
    const stakeTx = await pg.program.methods
      .stakeOptions(optionContractKp.publicKey, lockPeriod, optionType)
      .accounts({
        userAccount: userAccountKp.publicKey,
        stakingPool: stakingPoolKp.publicKey,
        optionContract: optionContractKp.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([userAccountKp, stakingPoolKp, optionContractKp])
      .rpc();
    console.log("stakeOptions tx:", stakeTx);

    // Unstake the option with an early unstake flag set to true.
    const unstakeTx = await pg.program.methods
      .unstakeOptions(true)
      .accounts({
        userAccount: userAccountKp.publicKey,
        stakingPool: stakingPoolKp.publicKey,
        optionContract: optionContractKp.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([userAccountKp, stakingPoolKp, optionContractKp])
      .rpc();
    console.log("unstakeOptions tx:", unstakeTx);

    // Claim rewards without compounding.
    // Reward distribution requires reward token account, reward mint, and the pool authority PDA.
    // These accounts are assumed to be set up already.
    const rewardTokenAccount = new web3.Keypair(); // Placeholder keypair.
    const rewardMint = new web3.Keypair(); // Placeholder keypair.
    
    // The pool authority is derived via PDA using the stakingPoolKp.publicKey.
    const claimTx = await pg.program.methods
      .claimRewards(false) // `false` indicates reward tokens are minted to the user.
      .accounts({
        userAccount: userAccountKp.publicKey,
        stakingPool: stakingPoolKp.publicKey,
        rewardTokenAccount: rewardTokenAccount.publicKey,
        rewardMint: rewardMint.publicKey,
        tokenProgram: web3.PublicKey.default, // Placeholder for Solana Playground 
        poolAuthority: web3.PublicKey.default 
      })  
      .signers([userAccountKp])
      .rpc();
    console.log("claimRewards tx:", claimTx);
  });
});
