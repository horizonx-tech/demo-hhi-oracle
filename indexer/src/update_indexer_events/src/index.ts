import path from 'path';
import * as indexer from '../../declarations/indexer_mainnet';
import * as mapping from '../../declarations/mapping_mainnet';
import {
  _SERVICE,
  TransferEvent,
} from '../../declarations/indexer_mainnet/indexer_mainnet.did';
import fetch from 'node-fetch';
import { JsonRpcProvider, Interface, Log } from 'ethers';
import { Erc20__factory } from '../types/ethers-contracts/factories/Erc20__factory';
import { Erc20 } from '../types/ethers-contracts';
import { ActorSubclass } from '@dfinity/agent';
import * as fs from 'fs';
const nodeFetch: any = fetch;
global.Headers = nodeFetch.Headers;
const DAI_ADDRESS = '0x6B175474E89094C44Da98b954EedeAC495271d0F';
const DAI_ADDRESS_OPTIMISM = '0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1';
const HOST = 'https://icp0.io';
const ENV = 'mainnet';
const GET_LOGS_MAX_RETRY = 5;
const SAVE_LOGS_MAX_RETRY = 5;
const canisterIds = () => {
  return require(path.resolve(
    __dirname,
    //    `../../../.dfx/${ENV}/canister_ids.json`
    `../../../canister_ids.json`
  ));
};
/**
 * @notice When exchanging numbers with a contract, the BigInt type must be used.
 * However, JSON can't use BigInt (happen "Do not know how to serialize a BigInt" error).
 * Therefore, when converting to JSON, convert BigInt to String, and parse String to BigInt on client.
 */
// @ts-ignore
// tslint:disable-next-line:typedef
BigInt.prototype.toJSON = function () {
  return this.toString();
};
const createIndexer = (canisterId: string) =>
  indexer.createActor(canisterId, {
    agentOptions: {
      fetch: require('node-fetch'),
      host: HOST,
    },
  });
//onst createHHISnapshot = (canisterId: string) =>
// hhiSnap.createActor(canisterId, {
//   agentOptions: {
//     fetch: require('node-fetch'),
//     host: HOST,
//   },
// });

const createMapping = (canisterIds: string) =>
  mapping.createActor(canisterIds, {
    agentOptions: {
      fetch: require('node-fetch'),
      host: HOST,
    },
  });

const canisterMainnet = () =>
  createIndexer(canisterIds().indexer_mainnet.mainnet as string);

const canisterOptmism = () =>
  createIndexer(canisterIds().indexer_optimism.mainnet as string);

//const hhiSnapshot = () =>
//  createHHISnapshot(canisterIds().hhi_snapshot.mainnet as string);

const mappingMainnet = () => {
  console.log(canisterIds().mapping_optimism.mainnet);

  return createMapping(canisterIds().mapping_mainnet.mainnet as string);
};
const mappingOptimism = () => {
  return createMapping(canisterIds().mapping_optimism.mainnet as string);
};

const _saveEvents = async (
  events: TransferEvent[],
  canister: ActorSubclass<_SERVICE>
) => {
  if (events.length == 0) {
    console.log('no events found');
    return;
  }
  const map = events.reduce((map, obj) => {
    const key = obj.block_number;
    if (!map.has(key)) {
      map.set(key, []);
    }
    map.get(key)!.push(obj);
    return map;
  }, new Map<bigint, TransferEvent[]>());
  let txCount = 0;
  Array.from(map.values()).forEach((e) => (txCount += e.length));
  console.log('saving events', txCount);
  await canister.update_events([...map]);
};

const saveEvents = async (
  events: TransferEvent[],
  canister: ActorSubclass<_SERVICE>
) => {
  await saveEventsWithRetry(events, 0, canister);
};

const saveEventsWithRetry = async (
  events: TransferEvent[],
  retryCount: number,
  canister: ActorSubclass<_SERVICE>
) => {
  try {
    await _saveEvents(events, canister);
  } catch (e) {
    if (retryCount > SAVE_LOGS_MAX_RETRY) {
      console.log('error', e);
      throw e;
    }
    const len = events.length;
    const mid = events.length / 2;
    await saveEventsWithRetry(events.slice(0, mid), retryCount + 1, canister);
    await saveEventsWithRetry(
      events.slice(mid + 1, len),
      retryCount + 1,
      canister
    );
  }
};
const _getLogs = async (
  contract: Erc20,
  from: number,
  to: number,
  provider: JsonRpcProvider
) => {
  return await provider.getLogs({
    address: await contract.getAddress(),
    toBlock: to,
    fromBlock: from,
    topics: [contract.interface.getEvent('Transfer').topicHash],
  });
};
const getLogs = async (
  contract: Erc20,
  from: number,
  to: number,
  provider: JsonRpcProvider
) => {
  return await getLogsWithRetry(contract, from, to, 0, provider);
};

const getLogsWithRetry = async (
  contract: Erc20,
  from: number,
  to: number,
  retryCount: number,
  provider: JsonRpcProvider
): Promise<Log[]> => {
  try {
    return await _getLogs(contract, from, to, provider);
  } catch (e) {
    if (retryCount > GET_LOGS_MAX_RETRY) {
      console.log('error', e);
      throw e;
    }
    const mid = from + (to - from) / 2;
    return (
      await getLogsWithRetry(contract, from, mid, retryCount + 1, provider)
    ).concat(
      await getLogsWithRetry(contract, mid + 1, to, retryCount + 1, provider)
    );
  }
};
const events_from_to = async (
  from: number,
  to: number,
  provider: JsonRpcProvider,
  address: string,
  batchSize?: number
): Promise<TransferEvent[]> => {
  let logs = [];
  let startFrom = from;
  const execBatchSize = batchSize ? batchSize : 1000;
  while (true) {
    let batchTo =
      startFrom + execBatchSize > to ? to : startFrom + execBatchSize;
    console.log('getting logs fromTo', startFrom, batchTo);
    const batchLogs = await getLogs(
      erc20Contract(provider, address),
      startFrom,
      batchTo,
      provider
    );
    logs.push(...batchLogs);
    if (batchTo == to) {
      break;
    }
    startFrom = batchTo + 1;
  }
  const erc20ContractIface = new Interface(Erc20__factory.abi);

  return logs
    .map((l) => {
      return {
        log: erc20ContractIface.parseLog({
          data: l.data,
          topics: l.topics.map((t) => t.toString()),
        })!,
        metadata: {
          hash: l.blockHash,
          blockNumber: l.blockNumber,
          at: l.blockNumber,
        },
      };
    })
    .map((t) => {
      return {
        block_number: BigInt(t.metadata.blockNumber),
        from: t.log.args[0],
        hash: t.metadata.hash,
        to: t.log.args[1],
        value: BigInt(t.log.args[2]),
        recipient: t.log.args[1],
        at: BigInt(t.metadata.at),
      };
    });
};

const providerMainnet = () => {
  return new JsonRpcProvider('https://mainnet.infura.io/v3/YOUR_KEY');
};
const providerOptimism = () => {
  return new JsonRpcProvider('https://optimism-mainnet.infura.io/v3/YOUR_KEY');
};
const erc20Contract = (provider: JsonRpcProvider, address: string) => {
  return Erc20__factory.connect(address, provider);
};

const updateLogsMainnet = async () => {
  await updateLogs(
    canisterMainnet(),
    providerMainnet(),
    DAI_ADDRESS,
    1000,
    3000
  );
};

const updateLogsOptimism = async () => {
  await updateLogs(
    canisterOptmism(),
    providerOptimism(),
    DAI_ADDRESS_OPTIMISM,
    5000,
    50000
  );
};

const saveLogsMainnet = async () => {
  await saveLogsInfoFile(
    canisterMainnet(),
    providerMainnet(),
    DAI_ADDRESS,
    1000,
    3000
  );
};
const saveLogsOptimism = async () => {
  await saveLogsInfoFile(
    canisterOptmism(),
    providerOptimism(),
    DAI_ADDRESS_OPTIMISM,
    5000,
    50000
  );
};

const latestBlockNumberFromFile = async (address: string) => {
  const path = `./logs/${address}/`;
  if (!fs.existsSync(path)) {
    fs.mkdirSync(path);
  }
  return fs
    .readdirSync(path)
    .map((f) => f.split('.')[0])
    .map((blockNumber) => Number(blockNumber))
    .reduce((a, b) => Math.max(a, b), 0);
};

const saveLogsInfoFile = async (
  instance: ActorSubclass<_SERVICE>,
  provider: JsonRpcProvider,
  address: string,
  getLogsBatchSize: number,
  batchSize: number
) => {
  const bn_at_deploy = await instance.block_number_at_deploy();
  const startFrom = Math.max(
    Number(await instance.latest_block_number()),
    await latestBlockNumberFromFile(address)
  );

  let processed = startFrom;

  while (true) {
    let to =
      processed + batchSize > bn_at_deploy
        ? Number(bn_at_deploy)
        : processed + batchSize;
    const events = await events_from_to(
      processed,
      to,
      provider,
      address,
      getLogsBatchSize
    );
    const filePath = `./logs/${address}/${processed}.json`;
    if (!fs.existsSync(`./logs/${address}`)) {
      fs.mkdirSync(`./logs/${address}`);
    }
    if (!fs.existsSync(filePath)) {
      fs.writeFileSync(filePath, JSON.stringify(events));
    }

    processed = to;
    if (processed >= bn_at_deploy) {
      break;
    }
  }
};

const updateLogs = async (
  instance: ActorSubclass<_SERVICE>,
  provider: JsonRpcProvider,
  address: string,
  getLogsBatchSize: number,
  batchSize: number
) => {
  const bn_at_deploy = await instance.block_number_at_deploy();
  console.log(bn_at_deploy);
  const startFrom = Number(await instance.latest_block_number());
  let processed = startFrom;
  while (true) {
    let to =
      processed + batchSize > bn_at_deploy
        ? Number(bn_at_deploy)
        : processed + batchSize;
    console.log('processing sync event fromTo', processed, to);
    const events = await events_from_to(
      processed,
      to,
      provider,
      address,
      getLogsBatchSize
    );
    await saveEvents(events, instance);
    processed = to;
    if (processed >= bn_at_deploy) {
      break;
    }
  }
};

const main = async () => {
  await mappingMainnet().subscribe(
    canisterIds().indexer_mainnet.mainnet as string
  );
  await mappingOptimism().subscribe(
    canisterIds().indexer_optimism.mainnet as string
  );
  //  await hhiSnapshot().setup(
  //    canisterIds().hhi.mainnet as string,
  //    canisterIds().mapping_mainnet.mainnet as string
  //  );

  await Promise.all([saveLogsMainnet(), saveLogsOptimism()]).then((values) =>
    console.log(values)
  );
};
main();
