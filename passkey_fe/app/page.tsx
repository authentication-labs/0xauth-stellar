"use client";
import { getPublicKeys } from "@/util/webauthn";
import base64url from "base64url";
import { useCallback } from "react";

export default function Home() {
  const register = useCallback(() => {
    (async () => {
      const options: CredentialCreationOptions = {
        publicKey: {
          rp: {
            name: "SL",
          },
          user: {
            id: Buffer.from(base64url("User")),
            displayName: "User",
            name: "User",
          },
          authenticatorSelection: {
            requireResidentKey: false,
            residentKey: "discouraged",
            userVerification: "discouraged",
          },
          challenge: Buffer.from(base64url("challenge")),
          pubKeyCredParams: [{ type: "public-key", alg: -7 }],
          attestation: "none",
        },
      };

      let res = await navigator.credentials.create(options);

      if (!res) {
        console.log("Failed to register ");
        throw new Error("Failed to create cred");
      }

      let pRes = res as PublicKeyCredential;
      console.log("Extracting pkey");
      const { contractSalt, publicKey } = await getPublicKeys(pRes);

      console.log({ publicKey, contractSalt });
    })().catch((er) => console.warn(er));
  }, []);

  return <button onClick={register}>Register</button>;
}
