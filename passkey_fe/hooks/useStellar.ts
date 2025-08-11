import {
  Contract,
  rpc,
  Networks,
  nativeToScVal,
  Address,
  Keypair,
} from "@stellar/stellar-sdk";

import * as factory from "../lib/factory-client";
import { randomBytes } from "crypto";
import { useCallback } from "react";
import { mnemonicToSeedSync } from "bip39";
import axios from "axios";

const RPC = "https://soroban-testnet.stellar.org:443";
const SOROBAN_SERVER = new rpc.Server(RPC);

const factoryClient = new factory.Client({
  ...factory.networks.testnet,
  rpcUrl: RPC,
});

export function useStellar() {
  const deployIdentity = useCallback(
    async (pubKey: Buffer<ArrayBufferLike>, salt: Buffer<ArrayBufferLike>) => {
      const res = await axios.post(
        "http://localhost:4001/identity/create-stellar",
        {
          publicKey: pubKey.toString("base64"),
          salt: salt.toString("base64"),
        },
        {
          headers: {
            Authorization: "Bearer localsecretstring",
          },
        }
      );

      return res.data;
    },
    []
  );

  return {
    deployIdentity,
  };
}
