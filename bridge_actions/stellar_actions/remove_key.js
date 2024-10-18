exports.handler = async function (payload) {
    const {
        Keypair,
        Contract,
        SorobanRpc,
        TransactionBuilder,
        Networks,
        BASE_FEE,
        xdr,
        StrKey
    } = require("@stellar/stellar-sdk");
    const axios = require('axios');

    try {
        const conditionRequest = payload.request?.body;
        let evmWalletAddress;
        let user;
        let removedKeyEVM;
        let key_purpose;

        // Check if conditionRequest exists and is valid
        if (conditionRequest) {
            for (const key in conditionRequest) {
                if (conditionRequest.hasOwnProperty(key)) {
                    if (key === 'matchReasons' && Array.isArray(conditionRequest[key])) {
                        conditionRequest[key].forEach((reason) => {
                            if (reason.args) {
                                console.log(`Match Reason Args: ${JSON.stringify(reason.args, null, 2)}`);

                                user = reason.args[0];
                                removedKeyEVM = reason.args[1];
                                key_purpose = reason.args[2];

                                console.log(`User (Address): ${user}`);
                                console.log(`key: ${removedKeyEVM}`);
                                console.log(`key_purpose: ${key_purpose}`);

                                evmWalletAddress = user;
                            }
                        });
                    }
                }
            }
        } else {
            console.warn('No condition request found in the payload.');
            return { status: 'error', message: 'No condition request found.' };
        }

        if (!evmWalletAddress) {
            console.error('No wallet address found in matchReasons.');
            return { status: 'error', message: 'No wallet address found.' };
        }

        const sourceSecret = 'SCPRTX6CYO4P224AISRDOO3TCR7I5X7PBEHD2PUZGVS355WIPV33HT6Z';
        const sorobanRpcUrl = "https://soroban-testnet.stellar.org:443";
        const contractAddress = 'CCBBUNYF265KODZCM2M4A5JP6XWMQIK3ZU75PUZ7TKJ5XH3YSTRADX64';
    
        const url = `https://services-dev.0xauth.co/wallet/wallet-sdk/get-user-wallets?evm_address=${evmWalletAddress}`;

        const sourceKeypair = Keypair.fromSecret(sourceSecret);
        const server = new SorobanRpc.Server(sorobanRpcUrl);
        const contract = new Contract(contractAddress);

        const sourceAccount = await server.getAccount(sourceKeypair.publicKey());
        const response = await axios.get(url);
        const data = response.data;
        const walletAddress = data.data.stellar_address;

        console.log('Stellar Address:', walletAddress);

        const walletPublicKey = StrKey.decodeEd25519PublicKey(walletAddress);
        const walletAddressParam = xdr.ScAddress.scAddressTypeAccount(new xdr.PublicKey.publicKeyTypeEd25519(walletPublicKey));
        const walletParam = xdr.ScVal.scvAddress(walletAddressParam);

        const walletPublicKeySourcekey = StrKey.decodeEd25519PublicKey(sourceAccount.accountId());
        const walletAddressParamSource = xdr.ScAddress.scAddressTypeAccount(new xdr.PublicKey.publicKeyTypeEd25519(walletPublicKeySourcekey));
        const walletParamSourceKey = xdr.ScVal.scvAddress(walletAddressParamSource);

        let getIdentity = new TransactionBuilder(sourceAccount, {
            fee: BASE_FEE,
            networkPassphrase: Networks.TESTNET,
        })
            .addOperation(
                contract.call("get_identity", walletParam)
            )
            .setTimeout(30)
            .build();

        console.log(`builtTransactiongetIdentity=${getIdentity.toXDR()}`);
        console.log('Transaction builtgetIdentity:', getIdentity.toXDR());

        
        let prepareGetID = await server.prepareTransaction(getIdentity);
        prepareGetID.sign(sourceKeypair);
        let sendGetID = await server.sendTransaction(prepareGetID);

        let decodedAddress;
        if (sendGetID.status === "PENDING") {
            let getResponse;
            do {
                console.log("Waiting for transaction confirmation...");
                await new Promise((resolve) => setTimeout(resolve, 1000));
                getResponse = await server.getTransaction(sendGetID.hash);
            } while (getResponse.status === "NOT_FOUND");

            if (getResponse.status === "SUCCESS") {
                if (!getResponse.resultMetaXdr) {
                    throw new Error("Empty resultMetaXDR in getTransaction response");
                }
                let transactionMeta = getResponse.resultMetaXdr;
                let returnValue = transactionMeta.v3().sorobanMeta().returnValue();

                const buffer = returnValue.value()._value;
                decodedAddress = StrKey.encodeContract(buffer);

                console.log('Returned address (decoded):', decodedAddress);
            } else {
                throw new Error(`Transaction failed: ${getResponse.resultXdr}`);
            }
        } else {
            throw new Error(sendGetID.errorResultXdr);
        }

        console.log('Sending RemoveKey transaction...');
        const identityAddress = decodedAddress;
        const identityAddressContract = new Contract(identityAddress);

        // Convert to numbers
        const keyPurposeNumber = Number(key_purpose);

        const purpose = xdr.ScVal.scvU32(keyPurposeNumber);

        let removeKey = new TransactionBuilder(sourceAccount, {
            fee: BASE_FEE,
            networkPassphrase: Networks.TESTNET,
        })
            .addOperation(
                identityAddressContract.call("remove_key", walletParamSourceKey, walletParamSourceKey, purpose)
            )
            .setTimeout(30)
            .build();

        console.log(`builtTransaction removeKey=${removeKey.toXDR()}`);
        console.log('Transaction removeKey:', removeKey.toXDR());

        let prepareRemoveKey = await server.prepareTransaction(removeKey);
        console.log('Transaction prepared:', prepareRemoveKey);

        prepareRemoveKey.sign(sourceKeypair);

        let signedXDR = prepareRemoveKey.toEnvelope().toXDR("base64");
        console.log('Signed prepared transaction XDR:', signedXDR);

        let sendRemoveKey = await server.sendTransaction(prepareRemoveKey);
        console.log('Transaction sent:', JSON.stringify(sendRemoveKey));

        if (sendRemoveKey.status === "PENDING") {
            let getResponse;
            do {
                console.log("Waiting for transaction confirmation...");
                await new Promise((resolve) => setTimeout(resolve, 1000));
                getResponse = await server.getTransaction(sendRemoveKey.hash);
            } while (getResponse.status === "NOT_FOUND");

            console.log('Transaction response:', JSON.stringify(getResponse));
            if (getResponse.status === "SUCCESS") {
                if (!getResponse.resultMetaXdr) {
                    throw new Error("Empty resultMetaXDR in getTransaction response");
                }
                let transactionMeta = getResponse.resultMetaXdr;
                let returnValue = transactionMeta.v3().sorobanMeta().returnValue();
                console.log('Transaction result:', returnValue.value());
            } else {
                throw new Error(`Transaction failed: ${getResponse.resultXdr}`);
            }
        } else {
            throw new Error(sendRemoveKey.errorResultXdr);
        }
    } catch (err) {
        console.error("An error occurred:", err);
        return { status: 'error', message: err.message };
    }
};