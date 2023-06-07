# indexer_oracle

```bash
yarn
SKIP_LOAD=true yarn compile
yarn test

# deploy
yarn hardhat --network xxx deploy:oraclev1
yarn hardhat --network xxx verify --contract contracts/OracleV1.sol:OracleV1 0x...
```
